#[doc(hidden)]
#[macro_export]
macro_rules! __emit {
    ($level:ident, $icon:literal, ftlog, $($arg:tt)+) => {
        $crate::__private::red4ext_rs::log::$level!(
            "[{}FTLog{}] {}",
            $crate::log::ftlog::color(),
            $crate::log::ftlog::reset(),
            format_args!($($arg)+)
        )
    };
    ($level:ident, $icon:literal, $name:ident, $($arg:tt)+) => {
        $crate::__private::red4ext_rs::log::$level!(
            "{} {}[{:^6}]{} {}",
            $icon,
            $crate::log::$name::color(),
            stringify!($name),
            $crate::log::$name::reset(),
            format_args!($($arg)+)
        )
    };
}
#[doc(hidden)]
pub use crate::__emit as emit;

/// Logs through one of yap's scopes (`red4`, `reds`, `ftlog`, `ext`), e.g.
/// `yap::log!(red4, info, "loaded {}", x)` or `yap::log!(reds, error, hint: "check the path", "unable to read {}", p)`.
///
/// Levels: `error` and `warn` (both accept a leading `hint: "..."`), `info`, `pipe`, `life`, `hook`, `debug`, `trace`.
// gated by RED4ext's log level, which drops these lines anyway: skips formatting them
#[macro_export]
macro_rules! log {
    ($scope:ident, error, hint: $hint:literal, $($arg:tt)+) => {
        $crate::log::emit!(error, "🔴", $scope, "{}\n   💡 hint: {}", format_args!($($arg)+), format_args!($hint))
    };
    ($scope:ident, error, $($arg:tt)+) => {
        $crate::log::emit!(error, "🔴", $scope, $($arg)+)
    };
    ($scope:ident, warn, hint: $hint:literal, $($arg:tt)+) => {
        $crate::log::emit!(warn, "🔶", $scope, "{}\n   💡 hint: {}", format_args!($($arg)+), format_args!($hint))
    };
    ($scope:ident, warn, $($arg:tt)+) => {
        $crate::log::emit!(warn, "🔶", $scope, $($arg)+)
    };
    ($scope:ident, info, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::INFO) {
            $crate::log::emit!(info, '🔷', $scope, $($arg)+)
        }
    };
    ($scope:ident, life, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::DEBUG) {
            $crate::log::emit!(debug, '🌱', $scope, $($arg)+)
        }
    };
    ($scope:ident, pipe, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::INFO) {
            $crate::log::emit!(info, "", $scope, $($arg)+)
        }
    };
    ($scope:ident, hook, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::DEBUG) {
            $crate::log::emit!(debug, '🪝', $scope, $($arg)+)
        }
    };
    ($scope:ident, debug, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::DEBUG) {
            $crate::log::emit!(debug, '🔹', $scope, $($arg)+)
        }
    };
    ($scope:ident, trace, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::TRACE) {
            $crate::log::emit!(trace, '•', $scope, $($arg)+)
        }
    };
}

// crate-local shorthands for `log!`, e.g. `red4::info!(...)`
macro_rules! scoped {
    // no color given
    ($d:tt $name:ident) => { scoped!($d $name, ""); };

    ($d:tt $name:ident, $color:literal) => {
        // some scopes (e.g. `ext`) are only used by the `hook` feature
        #[allow(dead_code, unused_imports, unused_macros)]
        pub mod $name {
            pub fn color() -> &'static str { if $crate::config::colors() { $color } else { "" } }
            pub fn reset() -> &'static str { if color().is_empty() { "" } else { "\x1b[0m" } }

            macro_rules! error { ($d($d arg:tt)+) => { $crate::log!($name, error, $d($d arg)+) } }
            macro_rules! warn_ { ($d($d arg:tt)+) => { $crate::log!($name, warn, $d($d arg)+) } }
            macro_rules! info { ($d($d arg:tt)+) => { $crate::log!($name, info, $d($d arg)+) } }
            macro_rules! life { ($d($d arg:tt)+) => { $crate::log!($name, life, $d($d arg)+) } }
            macro_rules! pipe { ($d($d arg:tt)+) => { $crate::log!($name, pipe, $d($d arg)+) } }
            macro_rules! hook { ($d($d arg:tt)+) => { $crate::log!($name, hook, $d($d arg)+) } }
            macro_rules! debug { ($d($d arg:tt)+) => { $crate::log!($name, debug, $d($d arg)+) } }
            macro_rules! trace { ($d($d arg:tt)+) => { $crate::log!($name, trace, $d($d arg)+) } }
            pub(crate) use {debug, error, hook, life, info, pipe, trace, warn_ as warn};
        }
    };
}

scoped!($ red4, "\x1b[33m"); // yellow
scoped!($ reds, "\x1b[31m"); // red
scoped!($ ftlog, "\x1b[90m"); // grey
scoped!($ ext, "\x1b[35m"); // purple
