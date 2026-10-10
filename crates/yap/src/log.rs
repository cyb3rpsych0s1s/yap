#[allow(unused_macros)]
macro_rules! emit {
    ($level:ident, $icon:literal, ftlog, $($arg:tt)+) => {
        ::red4ext_rs::log::$level!(
            "[{}FTLog{}] {}",
            $crate::log::ftlog::color(),
            $crate::log::ftlog::reset(),
            format_args!($($arg)+)
        )
    };
    ($level:ident, $icon:literal, $name:ident, $($arg:tt)+) => {
        ::red4ext_rs::log::$level!(
            "{} {}[{:^6}]{} {}",
            $icon,
            $crate::log::$name::color(),
            stringify!($name),
            $crate::log::$name::reset(),
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
            pub(crate) fn color() -> &'static str { if $crate::config::colors() { $color } else { "" } }
            pub(crate) fn reset() -> &'static str { if color().is_empty() { "" } else { "\x1b[0m" } }

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
                if $crate::config::enabled($crate::config::INFO) {
                    $crate::log::emit!(info, '🔷', $name, $d($d arg)+)
                }
            } }
            macro_rules! life { ($d($d arg:tt)+) => {
                if $crate::config::enabled($crate::config::DEBUG) {
                    $crate::log::emit!(debug, '🌱', $name, $d($d arg)+)
                }
            } }
            macro_rules! pipe { ($d($d arg:tt)+) => {
                if $crate::config::enabled($crate::config::INFO) {
                    $crate::log::emit!(info, "", $name, $d($d arg)+)
                }
            } }
            macro_rules! hook { ($d($d arg:tt)+) => {
                if $crate::config::enabled($crate::config::DEBUG) {
                    $crate::log::emit!(debug, '🪝', $name, $d($d arg)+)
                }
            } }
            macro_rules! debug { ($d($d arg:tt)+) => {
                if $crate::config::enabled($crate::config::DEBUG) {
                    $crate::log::emit!(debug, '🔹', $name, $d($d arg)+)
                }
            } }
            macro_rules! trace { ($d($d arg:tt)+) => {
                if $crate::config::enabled($crate::config::TRACE) {
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
