use crate::log::{ftlog, red4};
use red4ext_rs::{
    NativeRepr, RttiSystem, SdkEnv, VoidPtr, addr_hashes,
    types::{
        CALL_INSTR_SIZE, CName, Function, IScriptable, InvokeStatic, OPCODE_SIZE, RedString,
        ScriptRef, StackFrame, Type,
    },
};
use std::sync::OnceLock;

pub(super) type Native =
    unsafe extern "C" fn(i: *mut IScriptable, f: *mut StackFrame, a3: VoidPtr, a4: VoidPtr);

/// address hash of the native `FTLog` in RED4ext's address database
pub(super) const FTLOG_HASH: u32 = 0xCDF52009; // 0x140F60FE4 (2.31)
const NAME: &str = "FTLog";

/// opcode wrapping every `script_ref<T>` argument, followed by a `CBaseRTTIType*` (T) then the expression,
/// e.g. `66 <String type ptr> 18 <local>` or `66 <String type ptr> 24 <InvokeStatic>` (2.31)
pub(super) const REF_OPCODE: u8 = 0x66;
pub(super) const REF_INSTR_SIZE: usize = OPCODE_SIZE as usize + size_of::<usize>();

/// cached, since [`RttiSystem::get`] takes the RTTI write lock
pub(super) fn string_type() -> usize {
    static STRING: OnceLock<usize> = OnceLock::new();
    *STRING.get_or_init(|| {
        RttiSystem::get()
            .get_type(CName::new(RedString::NAME))
            .map_or(0, |t| t as *const Type as usize)
    })
}

/// `CStackFrame::code`, its first field, kept private by red4ext-rs:
/// the next instruction, i.e. the first argument when a native starts
pub(super) unsafe fn code(frame: *const StackFrame) -> *const u8 {
    unsafe { *frame.cast::<*const u8>() }
}

/// whether the next argument is a `script_ref<String>`, i.e. `REF_OPCODE` then the `String` type
pub(super) unsafe fn is_string_ref(frame: *const StackFrame) -> bool {
    unsafe {
        let code = code(frame);
        *code == REF_OPCODE
            && code
                .add(OPCODE_SIZE as usize)
                .cast::<usize>()
                .read_unaligned()
                == string_type()
    }
}

/// `None` when empty, invalid UTF-8 or with control characters: such lines are dropped
pub(super) fn message(msg: &RedString) -> Option<&str> {
    msg.to_str()
        .ok()
        .filter(|msg| !msg.is_empty() && !msg.bytes().any(|b| b.is_ascii_control() && b != b'\t'))
}

fn log(caller: Option<&Function>, msg: &RedString) {
    let Some(msg) = message(msg) else {
        return;
    };
    ftlog::pipe!("{}{msg}", crate::config::caller(caller));
}

unsafe extern "C" fn detour(
    i: *mut IScriptable,
    f: *mut StackFrame,
    a3: VoidPtr,
    a4: VoidPtr,
    cb: Native,
) {
    unsafe {
        let frame = &mut *f;
        let callee = frame
            .instr_at::<InvokeStatic>(-CALL_INSTR_SIZE - OPCODE_SIZE)
            .and_then(|call| { call.func }.as_ref());
        let other_native = callee.is_some_and(|c| c.name() != CName::new(NAME));

        if frame.has_code() && !other_native {
            let state = frame.args_state();

            if is_string_ref(f) {
                // script caller, as a pointer so that its borrow of the frame ends before stepping
                let caller = std::ptr::from_ref(frame.func());
                // evaluate the expression by value, rather than referencing a temporary
                // which the engine destroys as soon as the argument has been read
                for _ in 0..REF_INSTR_SIZE {
                    frame.step();
                }
                log(caller.as_ref(), &StackFrame::get_arg::<RedString>(frame));
            } else if callee.is_none() {
                // not called from scripts (e.g. from another native plugin)
                if let Some(msg) = StackFrame::get_arg::<ScriptRef<RedString>>(frame).value() {
                    log(None, msg);
                }
            }

            frame.restore_args(state);
        }
        cb(i, f, a3, a4)
    }
}

::red4ext_rs::hooks! {
    static HOOK: fn(i: *mut IScriptable, f: *mut StackFrame, a3: VoidPtr, a4: VoidPtr) -> ();
}

/// `None` when the hash is missing from the address database, e.g. after a game patch
fn target() -> Option<Native> {
    let addr = addr_hashes::resolve(FTLOG_HASH);
    (addr != 0).then(|| unsafe { std::mem::transmute::<usize, Native>(addr) })
}

pub fn attach_hook(env: &SdkEnv) {
    let Some(target) = target() else {
        return red4::warn!("unable to resolve {NAME} ({FTLOG_HASH:#X}), not hooked");
    };
    unsafe { env.attach_hook(HOOK, target, detour) };
    red4::info!("attached native internal hook for {NAME}");
}

pub fn detach_hook(env: &SdkEnv) {
    let Some(target) = target() else {
        return;
    };
    unsafe { env.detach_hook(target) };
    red4::info!("detached native internal hook for {NAME}");
}
