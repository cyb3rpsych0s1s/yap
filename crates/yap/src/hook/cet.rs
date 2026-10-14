use super::pe::{module_base, section};
use crate::log::{cet, ftlog, red4};
use red4ext_rs::{SdkEnv, VoidPtr};
use std::{cell::Cell, ffi::c_char, ops::Range, sync::OnceLock};

/// Cyber Engine Tweaks, loaded by the ASI loader long before RED4ext loads its plugins
const MODULE: &str = "cyber_engine_tweaks.asi";
const NAME: &str = "CET spdlog::logger::sink_it_";
/// MSVC RTTI `TypeDescriptor::name` of `spdlog::logger`, which CET links statically (spdlog 1.11.0)
const LOGGER_RTTI: &[u8] = b".?AVlogger@spdlog@@\0";
/// `TypeDescriptor::name`, after `pVFTable` and `spare`
const TYPE_DESCRIPTOR_NAME: usize = 2 * size_of::<usize>();
/// `RTTICompleteObjectLocator::signature` on x64, where its pointers are RVAs
const COL_SIGNATURE: u32 = 1;
/// `spdlog::logger` virtuals in declaration order (spdlog 1.11.0):
/// `~logger` (MSVC scalar deleting destructor), `clone`, `sink_it_`, `flush_`
const SINK_IT_SLOT: usize = 2;
const FLUSH_SLOT: usize = 3;
/// the same tag for every logger, which the logger name then tells apart
const TAG: &str = "CET";

/// spdlog `level::level_enum`, in the same order as [`crate::config::TRACE`]..[`crate::config::CRITICAL`]
const TRACE: i32 = 0;
const DEBUG: i32 = 1;
const INFO: i32 = 2;
const WARN: i32 = 3;

type SinkIt = unsafe extern "C" fn(this: VoidPtr, msg: *const LogMsg);

/// `spdlog::string_view_t`, i.e. `fmt::basic_string_view<char>`
#[repr(C)]
struct StrView {
    data: *const u8,
    size: usize,
}

impl StrView {
    fn as_bytes(&self) -> &[u8] {
        if self.data.is_null() {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(self.data, self.size) }
    }
}

/// `spdlog::source_loc`
#[repr(C)]
#[allow(dead_code, reason = "layout only")]
struct SourceLoc {
    filename: *const c_char,
    line: i32,
    funcname: *const c_char,
}

/// `spdlog::details::log_msg` (spdlog 1.11.0)
#[repr(C)]
#[allow(dead_code, reason = "layout only")]
struct LogMsg {
    logger_name: StrView,
    level: i32,
    time: i64,
    thread_id: usize,
    color_range_start: usize,
    color_range_end: usize,
    source: SourceLoc,
    payload: StrView,
}

// `sink_it_` reads `level` at 0x10 (CET 1.37.1)
const _: () = assert!(std::mem::offset_of!(LogMsg, level) == 0x10);
const _: () = assert!(std::mem::offset_of!(LogMsg, payload) == 0x50);

/// the hooked `sink_it_`, once attached
static TARGET: OnceLock<usize> = OnceLock::new();

thread_local! {
    static FORWARDING: Cell<bool> = const { Cell::new(false) };
}

/// restores [`FORWARDING`] even if the forwarded call unwinds
struct Forwarding(bool);

impl Drop for Forwarding {
    fn drop(&mut self) {
        FORWARDING.set(self.0);
    }
}

/// runs a native yap already logged: CET hooks the same natives (`FTLog`, `LogChannel*`)
/// and writes their lines again to its `gamelog` (and `scripting`), which mustn't get mirrored twice
pub(super) fn forwarding<R>(f: impl FnOnce() -> R) -> R {
    let _restore = Forwarding(FORWARDING.replace(true));
    f()
}

/// `None` when empty
fn message(payload: &[u8]) -> Option<String> {
    let msg = String::from_utf8_lossy(payload);
    let msg = msg.trim_end();
    (!msg.is_empty()).then(|| msg.to_owned())
}

/// `[CET:<logger>] msg`
fn log(msg: &LogMsg) {
    let Some(payload) = message(msg.payload.as_bytes()) else {
        return;
    };
    let logger = String::from_utf8_lossy(msg.logger_name.as_bytes());
    let (grey, blue, reset) = (ftlog::color(), cet::color(), cet::reset());
    match msg.level {
        TRACE => cet::trace!("[{grey}{TAG}{reset}:{blue}{logger}{reset}] {payload}"),
        DEBUG => cet::debug!("[{grey}{TAG}{reset}:{blue}{logger}{reset}] {payload}"),
        INFO => cet::pipe!("[{grey}{TAG}{reset}:{blue}{logger}{reset}] {payload}"),
        WARN => cet::warn!("[{grey}{TAG}{reset}:{blue}{logger}{reset}] {payload}"),
        // `err`, `critical`
        _ => cet::error!("[{grey}{TAG}{reset}:{blue}{logger}{reset}] {payload}"),
    }
}

// never log from the detour through `red4::*`, like the `Logger` detours
unsafe extern "C" fn detour(this: VoidPtr, msg: *const LogMsg, cb: SinkIt) {
    unsafe {
        cb(this, msg);
        if !FORWARDING.get()
            && let Some(msg) = msg.as_ref()
        {
            log(msg);
        }
    }
}

::red4ext_rs::hooks! {
    static HOOK: fn(this: VoidPtr, msg: *const LogMsg) -> ();
}

fn bytes(range: &Range<usize>) -> &'static [u8] {
    unsafe { std::slice::from_raw_parts(range.start as *const u8, range.len()) }
}

/// address of `spdlog::logger::sink_it_`, found through the class' MSVC RTTI:
/// `TypeDescriptor` (`.data`) → `CompleteObjectLocator` (`.rdata`) → vtable (`.rdata`)
fn find_sink_it(base: usize) -> Result<(usize, usize), &'static str> {
    let text = section(base, b".text").ok_or(".text section not found")?;
    let rdata = section(base, b".rdata").ok_or(".rdata section not found")?;
    let data = section(base, b".data").ok_or(".data section not found")?;

    let name = bytes(&data)
        .windows(LOGGER_RTTI.len())
        .position(|w| w == LOGGER_RTTI)
        .ok_or("spdlog::logger RTTI not found")?;
    let td_rva = (data.start + name - TYPE_DESCRIPTOR_NAME - base) as u32;

    let rdata_bytes = bytes(&rdata);
    let u32_at = |offset: usize| {
        u32::from_le_bytes(
            rdata_bytes[offset..offset + 4]
                .try_into()
                .unwrap_or_default(),
        )
    };
    // `signature`, `offset`, `cdOffset`, `pTypeDescriptor`, `pClassDescriptor`, `pSelf`
    let col = (0..rdata_bytes.len().saturating_sub(6 * 4))
        .step_by(4)
        .find(|&o| {
            u32_at(o) == COL_SIGNATURE
                && u32_at(o + 4) == 0
                && u32_at(o + 12) == td_rva
                && u32_at(o + 20) as usize == rdata.start + o - base
        })
        .map(|o| rdata.start + o)
        .ok_or("spdlog::logger CompleteObjectLocator not found")?;

    // the vtable is preceded by a pointer to its locator
    let meta = rdata_bytes
        .as_chunks::<{ size_of::<usize>() }>()
        .0
        .iter()
        .position(|&w| usize::from_le_bytes(w) == col)
        .ok_or("spdlog::logger vtable not found")?;
    let vtable = rdata.start + (meta + 1) * size_of::<usize>();
    let slot = |idx: usize| unsafe { (vtable as *const usize).add(idx).read() };

    let sink_it = slot(SINK_IT_SLOT);
    (text.contains(&sink_it) && text.contains(&slot(FLUSH_SLOT)))
        .then_some((vtable, sink_it))
        .ok_or("spdlog::logger vtable entries outside .text")
}

pub fn attach_hook(env: &SdkEnv) {
    let Ok(base) = module_base(MODULE) else {
        return red4::info!("{MODULE} not loaded, {NAME} not hooked");
    };
    let (vtable, target) = match find_sink_it(base) {
        Ok(found) => found,
        Err(e) => return red4::warn!("unable to locate {NAME} in {MODULE}: {e}, not hooked"),
    };
    red4::trace!("{MODULE} at {base:#X}, spdlog::logger vtable {vtable:#X}, sink_it_ {target:#X}");
    if TARGET.set(target).is_err() {
        return;
    }
    unsafe { env.attach_hook(HOOK, std::mem::transmute::<usize, SinkIt>(target), detour) };
    red4::info!("attached native internal hook for {NAME}");
}

pub fn detach_hook(env: &SdkEnv) {
    let Some(&target) = TARGET.get() else {
        return;
    };
    unsafe { env.detach_hook(std::mem::transmute::<usize, SinkIt>(target)) };
    red4::info!("detached native internal hook for {NAME}");
}
