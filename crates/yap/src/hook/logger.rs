use super::ftlog::FTLOG_HASH;
use crate::config::{self, enabled};
use crate::log::{
    ext::{color, reset},
    red4,
};
use libloading::os::windows::Library;
use red4ext_rs::{
    Hook, SdkEnv, U16CStr, VoidPtr, addr_hashes,
    internal::{PluginInfo, Sdk},
    wcstr,
};
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    ffi::{CStr, CString, OsString, VaList, c_char},
    mem::ManuallyDrop,
    ops::Range,
    os::windows::ffi::OsStringExt,
    path::PathBuf,
    ptr::null_mut,
    sync::{Arc, LazyLock, OnceLock, RwLock},
};

/// `Trace`, `Debug`, ..: `(PluginHandle, const char*)`
type Narrow = unsafe extern "C" fn(handle: VoidPtr, msg: *const c_char);
/// `TraceW`, `DebugW`, ..: `(PluginHandle, const wchar_t*)`
type Wide = unsafe extern "C" fn(handle: VoidPtr, msg: *const u16);
/// `TraceF`, `DebugF`, ..: `(PluginHandle, const char*, ...)`
type NarrowF = unsafe extern "C" fn(handle: VoidPtr, fmt: *const c_char, ...);
/// `TraceWF`, `DebugWF`, ..: `(PluginHandle, const wchar_t*, ...)`
type WideF = unsafe extern "C" fn(handle: VoidPtr, fmt: *const u16, ...);

type ResolveAddress = unsafe extern "C" fn(hash: u32) -> usize;
type Query = unsafe extern "C" fn(info: *mut PluginInfo);
type GetModuleFileNameW =
    unsafe extern "system" fn(module: VoidPtr, filename: *mut u16, size: u32) -> u32;
type VsPrintf = unsafe extern "C" fn(
    options: u64,
    buf: *mut c_char,
    count: usize,
    fmt: *const c_char,
    locale: VoidPtr,
    args: VaList<'_>,
) -> i32;
type VswPrintf = unsafe extern "C" fn(
    options: u64,
    buf: *mut u16,
    count: usize,
    fmt: *const u16,
    locale: VoidPtr,
    args: VaList<'_>,
) -> i32;

const MODULE: &str = "RED4ext.dll";
const OWN: &str = concat!(env!("CARGO_PKG_NAME"), ".dll");
const CRT: &str = "ucrtbase.dll";
const KERNEL: &str = "kernel32.dll";
/// UTF-16 units for a module path, long enough for paths past the legacy `MAX_PATH` (260)
const PATH_BUF_LEN: usize = 1024;
/// `Trace`, `Debug`, `Info`, `Warn`, `Error`, `Critical`
const LOGGER_LEVELS: usize = config::CRITICAL + 1;
/// variants of each level, in `Logger` table order
const VARIANTS: usize = 4;
/// `(PluginHandle, const char*)`
const PLAIN: usize = 0;
/// `(PluginHandle, const char* fmt, ...)`
const FORMAT: usize = 1;
/// `(PluginHandle, const wchar_t*)`
const WIDE: usize = 2;
/// `(PluginHandle, const wchar_t* fmt, ...)`
const WIDE_FORMAT: usize = 3;
/// function pointers in RED4ext's `v1::Logger`
const ENTRIES_LEN: usize = LOGGER_LEVELS * VARIANTS;

/// `_CRT_INTERNAL_PRINTF_STANDARD_SNPRINTF_BEHAVIOR`: always NUL-terminate and return the full length
const STANDARD_SNPRINTF_BEHAVIOR: u64 = 2;
/// `_CRT_INTERNAL_PRINTF_LEGACY_WIDE_SPECIFIERS`: `%s` in a wide format is a wide string
const LEGACY_WIDE_SPECIFIERS: u64 = 4;
/// what MSVC's `vsnprintf` / `_vsnwprintf` pass by default
const PRINTF_OPTIONS: u64 = STANDARD_SNPRINTF_BEHAVIOR | LEGACY_WIDE_SPECIFIERS;
const WIDE_STRING_FMT: &U16CStr = wcstr!("%ls");

/// `IMAGE_DOS_HEADER::e_lfanew`: offset of the `IMAGE_NT_HEADERS64`
const IMAGE_DOS_E_LFANEW: usize = 0x3C;
/// `IMAGE_NT_HEADERS64::OptionalHeader`, after `Signature` (4) and `IMAGE_FILE_HEADER` (0x14)
const IMAGE_NT_OPTIONAL_HEADER: usize = 0x18;
/// `IMAGE_OPTIONAL_HEADER64::SizeOfImage`
const IMAGE_OPTIONAL_SIZE_OF_IMAGE: usize = 0x38;

/// `(entry index, address)` of every detoured [`ENTRIES`]
static ATTACHED: OnceLock<Vec<(usize, usize)>> = OnceLock::new();
static PRINTF: OnceLock<Option<Printf>> = OnceLock::new();
/// this plugin's own `PluginHandle`, the one other plugins' lines get mirrored with
static OWN_HANDLE: OnceLock<usize> = OnceLock::new();
/// plugin name for every `PluginHandle` seen so far
static ORIGINS: LazyLock<RwLock<HashMap<usize, Arc<str>>>> = LazyLock::new(Default::default);

thread_local! {
    static REENTRANT: Cell<bool> = const { Cell::new(false) };
}

/// set while a detour runs, so that e.g. `InfoF` internally calling `Info` doesn't get mirrored twice
struct Guard;

impl Guard {
    fn enter() -> Option<Self> {
        (!REENTRANT.replace(true)).then_some(Self)
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        REENTRANT.set(false);
    }
}

struct Printf {
    narrow: VsPrintf,
    wide: VswPrintf,
}

/// the CRT is already loaded in the process, only grab its real exports
/// (`vsnprintf` and friends are header-only inline wrappers around them)
fn printf() -> Option<&'static Printf> {
    PRINTF
        .get_or_init(|| {
            let lib = Library::open_already_loaded(CRT).ok()?;
            let narrow = *unsafe { lib.get::<VsPrintf>(b"__stdio_common_vsprintf\0") }.ok()?;
            let wide = *unsafe { lib.get::<VswPrintf>(b"__stdio_common_vswprintf\0") }.ok()?;
            // keep the reference: both pointers must outlive the hooks
            let _ = lib.into_raw();
            Some(Printf { narrow, wide })
        })
        .as_ref()
}

fn format(fmt: *const c_char, args: VaList<'_>) -> Option<Vec<u8>> {
    let printf = printf()?;
    let len =
        unsafe { (printf.narrow)(PRINTF_OPTIONS, null_mut(), 0, fmt, null_mut(), args.clone()) };
    let len = usize::try_from(len).ok()?;
    let mut buf = vec![0u8; len + 1];
    let written = unsafe {
        (printf.narrow)(
            PRINTF_OPTIONS,
            buf.as_mut_ptr().cast(),
            buf.len(),
            fmt,
            null_mut(),
            args,
        )
    };
    (written >= 0).then(|| {
        buf.truncate(len);
        buf
    })
}

fn format_w(fmt: *const u16, args: VaList<'_>) -> Option<Vec<u16>> {
    let printf = printf()?;
    let len =
        unsafe { (printf.wide)(PRINTF_OPTIONS, null_mut(), 0, fmt, null_mut(), args.clone()) };
    let len = usize::try_from(len).ok()?;
    let mut buf = vec![0u16; len + 1];
    let written = unsafe {
        (printf.wide)(
            PRINTF_OPTIONS,
            buf.as_mut_ptr(),
            buf.len(),
            fmt,
            null_mut(),
            args,
        )
    };
    (written >= 0).then(|| {
        buf.truncate(len);
        buf
    })
}

fn prefix(origin: Option<&str>) -> String {
    origin.map_or_else(String::new, |origin| {
        format!("[{}{origin}{}] ", color(), reset())
    })
}

/// `[<origin>] <msg>` (or just `<msg>`), cut at the first NUL if any
fn line(origin: Option<&str>, msg: &[u8]) -> CString {
    let msg = msg.split(|&b| b == 0).next().unwrap_or_default();
    let mut out = prefix(origin).into_bytes();
    out.extend_from_slice(msg);
    // cannot fail: cut at the first NUL above
    CString::new(out).unwrap_or_default()
}

/// `[<origin>] <msg>` (or just `<msg>`), NUL-terminated, cut at the first NUL if any
fn line_w(origin: Option<&str>, msg: &[u16]) -> Vec<u16> {
    let msg = msg.split(|&c| c == 0).next().unwrap_or_default();
    prefix(origin)
        .encode_utf16()
        .chain(msg.iter().copied())
        .chain(Some(0))
        .collect()
}

/// the name the plugin gives itself to RED4ext, through its mandatory `Query` export
fn plugin_name(handle: VoidPtr) -> Option<String> {
    // not ours to free
    let lib = ManuallyDrop::new(unsafe { Library::from_raw(handle as usize as _) });
    let query = unsafe { lib.get::<Query>(b"Query\0") }.ok()?;
    let mut info = PluginInfo::default();
    unsafe { query(&mut info) };
    (!info.name.is_null()).then(|| unsafe { U16CStr::from_ptr_str(info.name) }.to_string_lossy())
}

/// base address (`HMODULE`) of a module already loaded in the process, e.g. `RED4ext.dll`
fn module_base(name: &str) -> Result<usize, libloading::Error> {
    let base = Library::open_already_loaded(name)?.into_raw() as usize;
    // balance the refcount taken by open_already_loaded
    drop(unsafe { Library::from_raw(base as _) });
    Ok(base)
}

fn module_path(handle: VoidPtr) -> Option<PathBuf> {
    let lib = Library::open_already_loaded(KERNEL).ok()?;
    let get = unsafe { lib.get::<GetModuleFileNameW>(b"GetModuleFileNameW\0") }.ok()?;
    let mut buf = [0u16; PATH_BUF_LEN];
    let len = unsafe { get(handle, buf.as_mut_ptr(), buf.len() as u32) } as usize;
    (len != 0).then(|| PathBuf::from(OsString::from_wide(&buf[..len])))
}

fn module_stem(handle: VoidPtr) -> Option<String> {
    Some(
        module_path(handle)?
            .file_stem()?
            .to_string_lossy()
            .into_owned(),
    )
}

/// name of the plugin behind `handle`, unless it's this plugin
fn origin(handle: VoidPtr) -> Option<Arc<str>> {
    let own = *OWN_HANDLE.get()?;
    let key = handle as usize;
    if key == own {
        return None;
    }
    if let Some(name) = ORIGINS.read().ok()?.get(&key) {
        return Some(name.clone());
    }
    let name: Arc<str> = plugin_name(handle)
        .or_else(|| module_stem(handle))
        .unwrap_or_else(|| format!("{handle:p}"))
        .into();
    Some(ORIGINS.write().ok()?.entry(key).or_insert(name).clone())
}

/// this plugin's handle, and the origin to tag another plugin's line with
fn mirror_to(handle: VoidPtr) -> Option<(VoidPtr, Arc<str>)> {
    Some((*OWN_HANDLE.get()? as VoidPtr, origin(handle)?))
}

/// the line of another plugin, to be written again with this plugin's handle
fn mirrored(handle: VoidPtr, msg: &[u8]) -> Option<(VoidPtr, CString)> {
    let (own, origin) = mirror_to(handle)?;
    Some((own, line(Some(&origin), msg)))
}

/// the line of another plugin, to be written again with this plugin's handle
fn mirrored_w(handle: VoidPtr, msg: &[u16]) -> Option<(VoidPtr, Vec<u16>)> {
    let (own, origin) = mirror_to(handle)?;
    Some((own, line_w(Some(&origin), msg)))
}

#[derive(Clone, Copy)]
struct Entry {
    name: &'static str,
    attach: fn(&SdkEnv, usize) -> bool,
    detach: fn(&SdkEnv, usize) -> bool,
}

// never log from a detour: every `red4::*` macro goes through the Logger again
macro_rules! tap {
    (narrow $module:ident, $name:expr, $level:expr) => {
        mod $module {
            use super::*;

            const NAME: &str = $name;
            const LEVEL: usize = $level;
            pub const ENTRY: Entry = Entry {
                name: NAME,
                attach,
                detach,
            };

            unsafe extern "C" fn detour(handle: VoidPtr, msg: *const c_char, cb: Narrow) {
                // filtered out by RED4ext anyway: don't mirror
                let Some(_guard) = Guard::enter().filter(|_| !msg.is_null() && enabled(LEVEL))
                else {
                    return unsafe { cb(handle, msg) };
                };
                unsafe { cb(handle, msg) };
                let msg = unsafe { CStr::from_ptr(msg) }.to_bytes();
                if let Some((own, line)) = mirrored(handle, msg) {
                    unsafe { cb(own, line.as_ptr()) };
                }
            }

            ::red4ext_rs::hooks! {
                static HOOK: fn(handle: VoidPtr, msg: *const c_char) -> ();
            }

            fn attach(env: &SdkEnv, addr: usize) -> bool {
                unsafe { env.attach_hook(HOOK, std::mem::transmute::<usize, Narrow>(addr), detour) }
            }
            fn detach(env: &SdkEnv, addr: usize) -> bool {
                unsafe { env.detach_hook(std::mem::transmute::<usize, Narrow>(addr)) }
            }
        }
    };
    (wide $module:ident, $name:expr, $level:expr) => {
        mod $module {
            use super::*;

            const NAME: &str = $name;
            const LEVEL: usize = $level;
            pub const ENTRY: Entry = Entry {
                name: NAME,
                attach,
                detach,
            };

            unsafe extern "C" fn detour(handle: VoidPtr, msg: *const u16, cb: Wide) {
                // filtered out by RED4ext anyway: don't mirror
                let Some(_guard) = Guard::enter().filter(|_| !msg.is_null() && enabled(LEVEL))
                else {
                    return unsafe { cb(handle, msg) };
                };
                unsafe { cb(handle, msg) };
                let msg = unsafe { U16CStr::from_ptr_str(msg) }.as_slice();
                if let Some((own, line)) = mirrored_w(handle, msg) {
                    unsafe { cb(own, line.as_ptr()) };
                }
            }

            ::red4ext_rs::hooks! {
                static HOOK: fn(handle: VoidPtr, msg: *const u16) -> ();
            }

            fn attach(env: &SdkEnv, addr: usize) -> bool {
                unsafe { env.attach_hook(HOOK, std::mem::transmute::<usize, Wide>(addr), detour) }
            }
            fn detach(env: &SdkEnv, addr: usize) -> bool {
                unsafe { env.detach_hook(std::mem::transmute::<usize, Wide>(addr)) }
            }
        }
    };
    // `hooks!` installs a non-variadic trampoline which would drop the `...` arguments,
    // so install a variadic one through [`Hook`] directly, typed as [`Narrow`] for `attach_hook` only
    (narrow_f $module:ident, $name:expr, $level:expr) => {
        mod $module {
            use super::*;

            const NAME: &str = $name;
            const LEVEL: usize = $level;
            pub const ENTRY: Entry = Entry {
                name: NAME,
                attach,
                detach,
            };

            static mut TARGET: Option<Narrow> = None;
            static mut DETOUR: Option<Narrow> = None;
            static mut HOOK: Hook<Narrow, Narrow> = Hook::new(
                unsafe { std::mem::transmute::<NarrowF, Narrow>(installed) },
                &raw mut TARGET,
                &raw mut DETOUR,
            );

            unsafe extern "C" fn installed(handle: VoidPtr, fmt: *const c_char, args: ...) {
                let cb = unsafe { *&raw const TARGET }.expect("target function should be set");
                let cb = unsafe { std::mem::transmute::<Narrow, NarrowF>(cb) };
                let Some(_guard) = Guard::enter().filter(|_| !fmt.is_null() && enabled(LEVEL))
                else {
                    // the arguments cannot be forwarded as-is, print the format verbatim:
                    // when filtered out by RED4ext, the line gets dropped without ever being formatted
                    return unsafe { cb(handle, c"%s".as_ptr(), fmt) };
                };
                let formatted = format(fmt, args);
                let msg = formatted
                    .as_deref()
                    .unwrap_or_else(|| unsafe { CStr::from_ptr(fmt) }.to_bytes());
                // the arguments cannot be forwarded as-is, pass the formatted message instead
                unsafe { cb(handle, c"%s".as_ptr(), line(None, msg).as_ptr()) };
                if let Some((own, line)) = mirrored(handle, msg) {
                    unsafe { cb(own, c"%s".as_ptr(), line.as_ptr()) };
                }
            }

            fn attach(env: &SdkEnv, addr: usize) -> bool {
                unsafe {
                    env.attach_hook(
                        &raw mut HOOK,
                        std::mem::transmute::<usize, Narrow>(addr),
                        std::mem::transmute::<NarrowF, Narrow>(installed),
                    )
                }
            }
            fn detach(env: &SdkEnv, addr: usize) -> bool {
                unsafe { env.detach_hook(std::mem::transmute::<usize, Narrow>(addr)) }
            }
        }
    };
    (wide_f $module:ident, $name:expr, $level:expr) => {
        mod $module {
            use super::*;

            const NAME: &str = $name;
            const LEVEL: usize = $level;
            pub const ENTRY: Entry = Entry {
                name: NAME,
                attach,
                detach,
            };

            static mut TARGET: Option<Narrow> = None;
            static mut DETOUR: Option<Narrow> = None;
            static mut HOOK: Hook<Narrow, Narrow> = Hook::new(
                unsafe { std::mem::transmute::<WideF, Narrow>(installed) },
                &raw mut TARGET,
                &raw mut DETOUR,
            );

            unsafe extern "C" fn installed(handle: VoidPtr, fmt: *const u16, args: ...) {
                let cb = unsafe { *&raw const TARGET }.expect("target function should be set");
                let cb = unsafe { std::mem::transmute::<Narrow, WideF>(cb) };
                let Some(_guard) = Guard::enter().filter(|_| !fmt.is_null() && enabled(LEVEL))
                else {
                    // the arguments cannot be forwarded as-is, print the format verbatim:
                    // when filtered out by RED4ext, the line gets dropped without ever being formatted
                    return unsafe { cb(handle, WIDE_STRING_FMT.as_ptr(), fmt) };
                };
                let formatted = format_w(fmt, args);
                let msg = formatted
                    .as_deref()
                    .unwrap_or_else(|| unsafe { U16CStr::from_ptr_str(fmt) }.as_slice());
                // the arguments cannot be forwarded as-is, pass the formatted message instead
                unsafe { cb(handle, WIDE_STRING_FMT.as_ptr(), line_w(None, msg).as_ptr()) };
                if let Some((own, line)) = mirrored_w(handle, msg) {
                    unsafe { cb(own, WIDE_STRING_FMT.as_ptr(), line.as_ptr()) };
                }
            }

            fn attach(env: &SdkEnv, addr: usize) -> bool {
                unsafe {
                    env.attach_hook(
                        &raw mut HOOK,
                        std::mem::transmute::<usize, Narrow>(addr),
                        std::mem::transmute::<WideF, Narrow>(installed),
                    )
                }
            }
            fn detach(env: &SdkEnv, addr: usize) -> bool {
                unsafe { env.detach_hook(std::mem::transmute::<usize, Narrow>(addr)) }
            }
        }
    };
}

/// one module per [`RED4ext::v1::Logger`] entry, listed in table order
macro_rules! taps {
    ($($level:path, $name:literal: $plain:ident, $f:ident, $w:ident, $wf:ident;)*) => {
        $(
            tap!(narrow $plain, $name, $level);
            tap!(narrow_f $f, concat!($name, "F"), $level);
            tap!(wide $w, concat!($name, "W"), $level);
            tap!(wide_f $wf, concat!($name, "WF"), $level);
        )*
        const ENTRIES: [Entry; ENTRIES_LEN] = [$($plain::ENTRY, $f::ENTRY, $w::ENTRY, $wf::ENTRY,)*];
    };
}

taps! {
    config::TRACE, "Trace": trace, trace_f, trace_w, trace_wf;
    config::DEBUG, "Debug": debug, debug_f, debug_w, debug_wf;
    config::INFO, "Info": info, info_f, info_w, info_wf;
    config::WARN, "Warn": warn, warn_f, warn_w, warn_wf;
    config::ERROR, "Error": error, error_f, error_w, error_wf;
    config::CRITICAL, "Critical": critical, critical_f, critical_w, critical_wf;
}

/// address range of the module image in memory, read from its PE headers
fn image_range(base: usize) -> Range<usize> {
    let size = unsafe {
        let e_lfanew = ((base + IMAGE_DOS_E_LFANEW) as *const u32).read_unaligned() as usize;
        let size_of_image =
            base + e_lfanew + IMAGE_NT_OPTIONAL_HEADER + IMAGE_OPTIONAL_SIZE_OF_IMAGE;
        (size_of_image as *const u32).read_unaligned() as usize
    };
    base..base + size
}

/// [`SdkEnv`] keeps `{ handle, sdk: Sdk }` private and rustc decides the order of both fields:
/// `handle` (this plugin's own HMODULE) is either its first or its last word, the [`Sdk`] the others.
///
/// Returns the word index of the [`Sdk`], and the `Sdk::logger` table as plain addresses.
fn find_logger(
    env: &SdkEnv,
    own: usize,
    range: &Range<usize>,
) -> Option<(usize, &'static [usize; ENTRIES_LEN])> {
    const WORD: usize = size_of::<usize>();
    const SDK_ENV_WORDS: usize = 1 + size_of::<Sdk>() / WORD;
    if size_of::<SdkEnv>() != SDK_ENV_WORDS * WORD {
        return None;
    }
    let words = unsafe { &*std::ptr::from_ref(env).cast::<[usize; SDK_ENV_WORDS]>() };
    let sdk_word = match words.iter().position(|&w| w == own)? {
        0 => 1,
        last if last == SDK_ENV_WORDS - 1 => 0,
        _ => return None,
    };
    let sdk = unsafe { &*std::ptr::from_ref(&words[sdk_word]).cast::<Sdk>() };
    let logger = unsafe { sdk.logger.as_ref() }?;
    // a RED4ext.SDK `Logger` with other entries must not be misread
    if size_of_val(logger) != size_of::<[usize; ENTRIES_LEN]>() {
        return None;
    }
    let table = unsafe { &*std::ptr::from_ref(logger).cast::<[usize; ENTRIES_LEN]>() };
    // every entry is implemented by RED4ext.dll
    table
        .iter()
        .all(|entry| range.contains(entry))
        .then_some((sdk_word, table))
}

fn sample_resolve() {
    let resolve = Library::open_already_loaded(MODULE).and_then(|lib| {
        unsafe { lib.get::<ResolveAddress>(b"RED4ext_ResolveAddress\0") }.map(|f| *f)
    });
    match resolve {
        Ok(resolve) => {
            let ours = unsafe { resolve(FTLOG_HASH) };
            let theirs = addr_hashes::resolve(FTLOG_HASH);
            red4::trace!(
                "RED4ext_ResolveAddress({FTLOG_HASH:#X}) = {ours:#X} (addr_hashes::resolve = {theirs:#X}, equal: {})",
                ours == theirs
            );
        }
        Err(e) => red4::warn!("unable to get RED4ext_ResolveAddress: {e}"),
    }
}

/// call every entry once through the table, including the variants red4ext-rs never uses
fn self_test(own: usize, table: &[usize; ENTRIES_LEN]) {
    let handle = own as VoidPtr;
    let arg_w: Vec<u16> = "answer\0".encode_utf16().collect();
    for (idx, (entry, &addr)) in ENTRIES.iter().zip(table).enumerate() {
        // the line no longer tells which variant was called, so the message does
        let msg = CString::new(format!("self-test {}", entry.name)).unwrap_or_default();
        let msg_w: Vec<u16> = format!("self-test {}\0", entry.name)
            .encode_utf16()
            .collect();
        let fmt = CString::new(format!("self-test {}: %s = %d", entry.name)).unwrap_or_default();
        let fmt_w: Vec<u16> = format!("self-test {}: %ls = %d\0", entry.name)
            .encode_utf16()
            .collect();
        unsafe {
            match idx % VARIANTS {
                PLAIN => std::mem::transmute::<usize, Narrow>(addr)(handle, msg.as_ptr()),
                FORMAT => std::mem::transmute::<usize, NarrowF>(addr)(
                    handle,
                    fmt.as_ptr(),
                    c"answer".as_ptr(),
                    42i32,
                ),
                WIDE => std::mem::transmute::<usize, Wide>(addr)(handle, msg_w.as_ptr()),
                WIDE_FORMAT => std::mem::transmute::<usize, WideF>(addr)(
                    handle,
                    fmt_w.as_ptr(),
                    arg_w.as_ptr(),
                    42i32,
                ),
                _ => unreachable!(),
            }
        }
    }
}

pub fn attach_hook(env: &SdkEnv) {
    let range = match module_base(MODULE) {
        Ok(base) => image_range(base),
        Err(e) => return red4::warn!("unable to open {MODULE}: {e}"),
    };
    red4::trace!("{MODULE} image at {:#X}..{:#X}", range.start, range.end);
    sample_resolve();

    let own = match module_base(OWN) {
        Ok(base) => base,
        Err(e) => return red4::warn!("unable to open {OWN}: {e}"),
    };
    // before any hook is attached, so that other plugins' lines get mirrored right away
    let _ = OWN_HANDLE.set(own);

    let Some((idx, table)) = find_logger(env, own, &range) else {
        return red4::warn!("unable to locate the Logger table in SdkEnv");
    };
    red4::trace!("Logger table {table:p} (Sdk at SdkEnv word #{idx})");
    if printf().is_none() {
        red4::warn!(
            "unable to resolve printf from {CRT}, formatted entries will print their format verbatim"
        );
    }

    // attached last, so every line above is logged untouched
    let mut seen = HashSet::with_capacity(ENTRIES_LEN);
    let mut attached = Vec::with_capacity(ENTRIES_LEN);
    let mut skipped = vec![];
    for (idx, (entry, &addr)) in ENTRIES.iter().zip(table).enumerate() {
        // the same implementation cannot be detoured twice
        if !seen.insert(addr) {
            skipped.push(entry.name);
            continue;
        }
        if (entry.attach)(env, addr) {
            attached.push((idx, addr));
        } else {
            skipped.push(entry.name);
        }
    }
    let count = attached.len();
    let _ = ATTACHED.set(attached);
    red4::info!("attached {count}/{ENTRIES_LEN} Logger hooks, skipped: {skipped:?}");

    if cfg!(debug_assertions) {
        self_test(own, table);
    }
}

pub fn detach_hook(env: &SdkEnv) {
    let Some(attached) = ATTACHED.get() else {
        return;
    };
    let detached = attached
        .iter()
        .filter(|&&(idx, addr)| (ENTRIES[idx].detach)(env, addr))
        .count();
    red4::info!("detached {detached}/{} Logger hooks", attached.len());
}
