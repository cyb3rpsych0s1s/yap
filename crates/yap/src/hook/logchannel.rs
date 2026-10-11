use super::ftlog::{Native, REF_INSTR_SIZE, is_string_ref, message};
use crate::config;
use crate::log::{chan, red4};
use red4ext_rs::{
    NativeRepr, RttiSystem, SdkEnv, VoidPtr, addr_hashes,
    types::{
        CALL_INSTR_SIZE, CName, Function, IScriptable, InvokeStatic, OPCODE_SIZE, RedString,
        ScriptRef, StackFrame,
    },
};
use std::sync::OnceLock;

#[derive(Clone, Copy)]
enum Level {
    Info,
    Warn,
    Error,
}

/// a global `LogChannel*(channel: CName, text: script_ref<String>)` native,
/// which reads its arguments but logs nothing in retail builds
struct Channel {
    name: &'static str,
    level: Level,
}

const LOG_CHANNEL: Channel = Channel {
    name: "LogChannel",
    level: Level::Info,
};
const LOG_CHANNEL_WARNING: Channel = Channel {
    name: "LogChannelWarning",
    level: Level::Warn,
};
const LOG_CHANNEL_ERROR: Channel = Channel {
    name: "LogChannelError",
    level: Level::Error,
};

/// `LogChannel` and `LogChannelError` share a single native (identical code folding),
/// which can only be hooked once: one detour for both, told apart by the callee's name
const SHARED: &[Channel] = &[LOG_CHANNEL, LOG_CHANNEL_ERROR];
const WARNING: &[Channel] = &[LOG_CHANNEL_WARNING];

/// address hashes in RED4ext's address database, any of which resolves to the native
const SHARED_HASHES: &[u32] = &[0xB7C91CD0, 0x63201EDA]; // 0x140D279D8 (2.31)
const WARNING_HASHES: &[u32] = &[0xA9621FA6]; // 0x14205573C (2.31)

/// whether the declared `text` parameter is a `script_ref<String>` rather than a `String`,
/// `None` when neither: native callers pass it as declared, so reading the wrong type would corrupt memory.
/// Cached, since [`RttiSystem::get`] takes the RTTI write lock. The three natives share the same shape.
fn text_is_ref() -> Option<bool> {
    static TEXT: OnceLock<Option<bool>> = OnceLock::new();
    *TEXT.get_or_init(|| {
        let rtti = RttiSystem::get();
        let func = rtti.get_function(CName::new(LOG_CHANNEL.name))?;
        let text = func.params().get(1)?.type_().name();
        if text == CName::new(RedString::NAME) {
            Some(false)
        } else if text == CName::new("script_ref:String") {
            Some(true)
        } else {
            None
        }
    })
}

fn log(level: Level, channel: CName, caller: Option<&Function>, msg: &RedString) {
    let Some(msg) = message(msg) else {
        return;
    };
    let (color, reset) = (chan::color(), chan::reset());
    let caller = crate::config::caller(caller);
    let channel = channel.as_str();
    match level {
        Level::Info => chan::pipe!("[{color}{channel}{reset}] {caller}{msg}"),
        Level::Warn => chan::warn!("[{color}{channel}{reset}] {caller}{msg}"),
        Level::Error => chan::error!("[{color}{channel}{reset}] {caller}{msg}"),
    }
}

/// logs the call's `(channel, text)` arguments, then leaves them for the original native
unsafe fn capture(f: *mut StackFrame, natives: &[Channel]) {
    unsafe {
        let frame = &mut *f;
        if !frame.has_code() {
            return;
        }
        let named = |func: &Function| {
            natives
                .iter()
                .find(|native| func.short_name() == CName::new(native.name))
        };
        let callee = frame
            .instr_at::<InvokeStatic>(-CALL_INSTR_SIZE - OPCODE_SIZE)
            .and_then(|call| { call.func }.as_ref());
        // script caller, as a pointer so that its borrow of the frame ends before stepping
        let (native, caller) = match callee {
            Some(callee) => match named(callee) {
                Some(native) => (native, config::func(frame).map(std::ptr::from_ref)),
                None => return, // another native sharing the address
            },
            // not called from scripts (e.g. from another native plugin, like Codeware's `Red::Log`):
            // `Function::Execute` leaves the frame's function null, so `LogChannel` and `LogChannelError`
            // can't be told apart there, Codeware only calls `LogChannel`
            None => (
                config::func(frame).and_then(named).unwrap_or(&natives[0]),
                None,
            ),
        };

        let state = frame.args_state();
        let channel = StackFrame::get_arg::<CName>(frame);
        let msg = if caller.is_some() {
            // evaluate the expression by value, rather than referencing a temporary
            // which the engine destroys as soon as the argument has been read
            if is_string_ref(f) {
                for _ in 0..REF_INSTR_SIZE {
                    frame.step();
                }
            }
            Some(StackFrame::get_arg::<RedString>(frame))
        } else {
            match text_is_ref() {
                Some(true) => StackFrame::get_arg::<ScriptRef<RedString>>(frame)
                    .value()
                    .cloned(),
                Some(false) => Some(StackFrame::get_arg::<RedString>(frame)),
                None => None,
            }
        };
        frame.restore_args(state);

        if let Some(msg) = msg {
            log(native.level, channel, caller.and_then(|c| c.as_ref()), &msg);
        }
    }
}

unsafe extern "C" fn detour_shared(
    i: *mut IScriptable,
    f: *mut StackFrame,
    a3: VoidPtr,
    a4: VoidPtr,
    cb: Native,
) {
    unsafe {
        capture(f, SHARED);
        cb(i, f, a3, a4)
    }
}

unsafe extern "C" fn detour_warning(
    i: *mut IScriptable,
    f: *mut StackFrame,
    a3: VoidPtr,
    a4: VoidPtr,
    cb: Native,
) {
    unsafe {
        capture(f, WARNING);
        cb(i, f, a3, a4)
    }
}

::red4ext_rs::hooks! {
    static HOOK_SHARED: fn(i: *mut IScriptable, f: *mut StackFrame, a3: VoidPtr, a4: VoidPtr) -> ();
    static HOOK_WARNING: fn(i: *mut IScriptable, f: *mut StackFrame, a3: VoidPtr, a4: VoidPtr) -> ();
}

/// `None` when every hash is missing from the address database, e.g. after a game patch
fn target(hashes: &[u32]) -> Option<Native> {
    hashes
        .iter()
        .map(|&hash| addr_hashes::resolve(hash))
        .find(|&addr| addr != 0)
        .map(|addr| unsafe { std::mem::transmute::<usize, Native>(addr) })
}

/// e.g. `LogChannel/LogChannelError`
fn names(natives: &[Channel]) -> String {
    natives
        .iter()
        .map(|native| native.name)
        .collect::<Vec<_>>()
        .join("/")
}

pub fn attach_hook(env: &SdkEnv) {
    unsafe {
        attach(env, SHARED, SHARED_HASHES, HOOK_SHARED, detour_shared);
        attach(env, WARNING, WARNING_HASHES, HOOK_WARNING, detour_warning);
    }
}

fn attach(
    env: &SdkEnv,
    natives: &[Channel],
    hashes: &[u32],
    hook: *mut ::red4ext_rs::Hook<
        unsafe extern "C" fn(*mut IScriptable, *mut StackFrame, VoidPtr, VoidPtr),
        unsafe extern "C" fn(*mut IScriptable, *mut StackFrame, VoidPtr, VoidPtr, Native),
    >,
    detour: unsafe extern "C" fn(*mut IScriptable, *mut StackFrame, VoidPtr, VoidPtr, Native),
) {
    let Some(target) = target(hashes) else {
        return red4::warn!(
            "unable to resolve {} ({hashes:#X?}), not hooked",
            names(natives)
        );
    };
    unsafe { env.attach_hook(hook, target, detour) };
    red4::info!("attached native internal hook for {}", names(natives));
}

pub fn detach_hook(env: &SdkEnv) {
    for (natives, hashes) in [(WARNING, WARNING_HASHES), (SHARED, SHARED_HASHES)] {
        let Some(target) = target(hashes) else {
            continue;
        };
        unsafe { env.detach_hook(target) };
        red4::info!("detached native internal hook for {}", names(natives));
    }
}
