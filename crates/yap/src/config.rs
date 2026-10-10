use red4ext_rs::types::Function;
use std::{
    path::Path,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::log::{ftlog, red4};

/// `<game>/bin/x64/Cyberpunk2077.exe`: the game folder is 3 levels above the executable
const EXE_DEPTH: usize = 3;
/// RED4ext always lives in `<game>/red4ext/`, next to its config
const RED4EXT_DIR: &str = "red4ext";
/// RED4ext's and yap's own (optional, never shipped so that updates don't overwrite it)
const CONFIG: &str = "config.ini";
/// https://no-color.org: disables colors when set and not empty, unless yap's config says otherwise
const NO_COLOR: &str = "NO_COLOR";

/// whether lines get ANSI colors, on until [`init`] reads the opt-outs
static COLORS: AtomicBool = AtomicBool::new(true);

pub(crate) fn colors() -> bool {
    COLORS.load(Ordering::Relaxed)
}

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

/// read the colors opt-outs, then RED4ext's log level from its `config.ini`, wherever the game is installed
pub(crate) fn init() {
    let game = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.ancestors().nth(EXE_DEPTH)?.to_path_buf()));
    init_colors(game.as_deref());
    // one component per join, so that the path only uses the platform separator
    let config = game.map(|game| game.join(RED4EXT_DIR).join(CONFIG));
    match config.as_deref().map(|path| (path, read_level(path))) {
        Some((path, Some(level))) => {
            let _ = MIN_LEVEL.set(level);
            // the config path only helps when diagnosing
            if enabled(DEBUG) {
                red4::info!("RED4ext log level: {} ({})", LEVELS[level], path.display());
            } else {
                red4::info!("RED4ext log level: {}", LEVELS[level]);
            }
        }
        Some((path, None)) => red4::warn!("unable to read [logging] level from {}", path.display()),
        None => red4::warn!("unable to locate RED4ext's {CONFIG}"),
    }
}

/// `[logging] colors` from yap's own `config.ini` if set, otherwise off when `NO_COLOR` is set
fn init_colors(game: Option<&Path>) {
    let config = game.map(|game| {
        game.join(RED4EXT_DIR)
            .join("plugins")
            .join("yap")
            .join(CONFIG)
    });
    let from_config = config.as_deref().and_then(|path| {
        let value = read_key(path, "logging", "colors")?;
        match value.as_str() {
            "true" | "on" | "yes" | "1" => Some(true),
            "false" | "off" | "no" | "0" => Some(false),
            _ => {
                red4::warn!("ignored [logging] colors = {value} from {}", path.display());
                None
            }
        }
    });
    let colors = from_config
        .unwrap_or_else(|| std::env::var_os(NO_COLOR).is_none_or(|value| value.is_empty()));
    COLORS.store(colors, Ordering::Relaxed);
}

/// `[logging] level` from RED4ext's `config.ini`, as an index of [`LEVELS`]
fn read_level(path: &Path) -> Option<usize> {
    // spdlog level names, short and long
    match read_key(path, "logging", "level")?.as_str() {
        "trace" => Some(TRACE),
        "debug" => Some(DEBUG),
        "info" => Some(INFO),
        "warn" | "warning" => Some(WARN),
        "err" | "error" => Some(ERROR),
        "critical" => Some(CRITICAL),
        "off" => Some(OFF),
        _ => None,
    }
}

/// `[section] key = value` from a TOML-like `.ini`, unquoted and lowercased
fn read_key(path: &Path, section: &str, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut current = "";
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            current = name.trim();
            continue;
        }
        let Some((k, value)) = line.split_once('=') else {
            continue;
        };
        if current == section && k.trim() == key {
            return Some(value.trim().trim_matches(['"', '\'']).to_ascii_lowercase());
        }
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
            ftlog::color(),
            class.name().as_str(),
            f.short_name().as_str(),
            ftlog::reset()
        ),
        None => format!(
            "{}{}{} ",
            ftlog::color(),
            f.short_name().as_str(),
            ftlog::reset()
        ),
    }
}
