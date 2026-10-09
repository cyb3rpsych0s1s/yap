use red4ext_rs::types::Function;
use std::{path::Path, sync::OnceLock};

/// `<game>/bin/x64/Cyberpunk2077.exe`: the game folder is 3 levels above the executable
const EXE_DEPTH: usize = 3;
/// RED4ext always lives in `<game>/red4ext/`, next to its config
const RED4EXT_CONFIG: &str = "red4ext/config.ini";

/// spdlog levels, in RED4ext's `Logger` order
pub(crate) const TRACE: usize = 0;
pub(crate) const DEBUG: usize = 1;
pub(crate) const INFO: usize = 2;
pub(crate) const WARN: usize = 3;
pub(crate) const ERROR: usize = 4;
pub(crate) const CRITICAL: usize = 5;
const OFF: usize = 6;
/// display names, indexed by the levels above
const LEVELS: [&str; OFF + 1] = ["Trace", "Debug", "Info", "Warn", "Error", "Critical", "Off"];

/// lowest level RED4ext writes, from `[logging] level` in its `config.ini`, if known
static MIN_LEVEL: OnceLock<usize> = OnceLock::new();

/// whether RED4ext writes lines at `level`, when unknown let RED4ext decide
pub(crate) fn enabled(level: usize) -> bool {
    MIN_LEVEL.get().is_none_or(|&min| level >= min)
}

/// read RED4ext's log level from its `config.ini`, wherever the game is installed
pub(crate) fn init() {
    let config = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.ancestors().nth(EXE_DEPTH)?.join(RED4EXT_CONFIG)));
    match config.as_deref().map(|path| (path, read_level(path))) {
        Some((path, Some(level))) => {
            let _ = MIN_LEVEL.set(level);
            red4::info!("RED4ext log level: {} ({})", LEVELS[level], path.display());
        }
        Some((path, None)) => red4::warn!("unable to read [logging] level from {}", path.display()),
        None => red4::warn!("unable to locate RED4ext's {RED4EXT_CONFIG}"),
    }
}

/// `[logging] level` from RED4ext's `config.ini`, as an index of [`LEVELS`]
fn read_level(path: &Path) -> Option<usize> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut section = "";
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = name.trim();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if section != "logging" || key.trim() != "level" {
            continue;
        }
        // spdlog level names, short and long
        return match value
            .trim()
            .trim_matches(['"', '\''])
            .to_ascii_lowercase()
            .as_str()
        {
            "trace" => Some(TRACE),
            "debug" => Some(DEBUG),
            "info" => Some(INFO),
            "warn" | "warning" => Some(WARN),
            "err" | "error" => Some(ERROR),
            "critical" => Some(CRITICAL),
            "off" => Some(OFF),
            _ => None,
        };
    }
    None
}

/// `Class::func ` (or `func `) of the calling script function, empty when unknown or native
pub(crate) fn caller(f: Option<&Function>) -> String {
    let Some(f) = f.filter(|f| !f.flags().is_native()) else {
        return String::new();
    };
    match f.parent() {
        Some(class) => format!(
            "{}{}::{}{} ",
            ftlog::COLOR,
            class.name().as_str(),
            f.short_name().as_str(),
            ftlog::RESET
        ),
        None => format!(
            "{}{}{} ",
            ftlog::COLOR,
            f.short_name().as_str(),
            ftlog::RESET
        ),
    }
}

#[allow(unused_macros)]
macro_rules! emit {
    ($level:ident, $icon:literal, ftlog, $($arg:tt)+) => {
        ::red4ext_rs::log::$level!(
            "[{}FTLog{}] {}",
            $crate::log::ftlog::COLOR,
            $crate::log::ftlog::RESET,
            format_args!($($arg)+)
        )
    };
    ($level:ident, $icon:literal, $name:ident, $($arg:tt)+) => {
        ::red4ext_rs::log::$level!(
            "{} {}[{:^6}]{} {}",
            $icon,
            $crate::log::$name::COLOR,
            stringify!($name),
            $crate::log::$name::RESET,
            format_args!($($arg)+)
        )
    };
}
pub(crate) use emit;

// gated by RED4ext's log level, which drops these lines anyway: skips formatting them
macro_rules! scoped {
    // no color given
    ($d:tt $name:ident) => { scoped!($d $name, ""); };

    ($d:tt $name:ident, $color:literal) => {
        // some scopes (e.g. `ext`) are only used by the `hook` feature
        #[allow(dead_code, unused_imports, unused_macros)]
        pub mod $name {
            pub(crate) const COLOR: &str = if cfg!(debug_assertions) { $color } else { "" };
            pub(crate) const RESET: &str = if COLOR.is_empty() { "" } else { "\x1b[0m" };

            macro_rules! error {
                (hint: $hint:literal, $d($d arg:tt)+) => {
                    $crate::log::emit!(error, "🔴", $name, "{}\n   💡 hint: {}", format_args!($d($d arg)+), format_args!($hint))
                };
                ($d($d arg:tt)+) => {
                    $crate::log::emit!(error, "🔴", $name, $d($d arg)+)
                };
            }
            macro_rules! warn_ {
                (hint: $hint:literal, $d($d arg:tt)+) => {
                    $crate::log::emit!(warn, "🔶", $name, "{}\n   💡 hint: {}", format_args!($d($d arg)+), format_args!($hint))
                };
                ($d($d arg:tt)+) => {
                    $crate::log::emit!(warn, "🔶", $name, $d($d arg)+)
                };
            }
            macro_rules! info { ($d($d arg:tt)+) => {
                if $crate::log::enabled($crate::log::INFO) {
                    $crate::log::emit!(info, '🔷', $name, $d($d arg)+)
                }
            } }
            macro_rules! life { ($d($d arg:tt)+) => {
                if $crate::log::enabled($crate::log::DEBUG) {
                    $crate::log::emit!(debug, '🌱', $name, $d($d arg)+)
                }
            } }
            macro_rules! pipe { ($d($d arg:tt)+) => {
                if $crate::log::enabled($crate::log::INFO) {
                    $crate::log::emit!(info, "", $name, $d($d arg)+)
                }
            } }
            macro_rules! hook { ($d($d arg:tt)+) => {
                if $crate::log::enabled($crate::log::DEBUG) {
                    $crate::log::emit!(debug, '🪝', $name, $d($d arg)+)
                }
            } }
            macro_rules! debug { ($d($d arg:tt)+) => {
                if $crate::log::enabled($crate::log::DEBUG) {
                    $crate::log::emit!(debug, '🔹', $name, $d($d arg)+)
                }
            } }
            macro_rules! trace { ($d($d arg:tt)+) => {
                if $crate::log::enabled($crate::log::TRACE) {
                    $crate::log::emit!(trace, '•', $name, $d($d arg)+)
                }
            } }
            pub(crate) use {debug, error, hook, life, info, pipe, trace, warn_ as warn};
        }
    };
}

scoped!($ red4, "\x1b[33m"); // yellow
scoped!($ reds, "\x1b[31m"); // red
scoped!($ ftlog, "\x1b[90m"); // grey
scoped!($ ext, "\x1b[35m"); // purple
