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

// `<scope>::<level>!` shorthands for `log!`, e.g. `yap::log::red4::info!(...)`, usable from other crates.
// Written out: a macro defined by a macro expansion can't be exported by path,
// so each one is exported under a hidden name at the crate root, then re-exported by its scope module.
// (no `#[rustfmt::skip]` wrapper module: rustc would then treat them as macro-expanded)
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_red4_error { ($($arg:tt)+) => { $crate::log!(red4, error, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_red4_warn { ($($arg:tt)+) => { $crate::log!(red4, warn, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_red4_info { ($($arg:tt)+) => { $crate::log!(red4, info, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_red4_life { ($($arg:tt)+) => { $crate::log!(red4, life, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_red4_pipe { ($($arg:tt)+) => { $crate::log!(red4, pipe, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_red4_hook { ($($arg:tt)+) => { $crate::log!(red4, hook, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_red4_debug { ($($arg:tt)+) => { $crate::log!(red4, debug, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_red4_trace { ($($arg:tt)+) => { $crate::log!(red4, trace, $($arg)+) } }

#[doc(hidden)]
#[macro_export]
macro_rules! __yap_reds_error { ($($arg:tt)+) => { $crate::log!(reds, error, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_reds_warn { ($($arg:tt)+) => { $crate::log!(reds, warn, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_reds_info { ($($arg:tt)+) => { $crate::log!(reds, info, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_reds_life { ($($arg:tt)+) => { $crate::log!(reds, life, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_reds_pipe { ($($arg:tt)+) => { $crate::log!(reds, pipe, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_reds_hook { ($($arg:tt)+) => { $crate::log!(reds, hook, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_reds_debug { ($($arg:tt)+) => { $crate::log!(reds, debug, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_reds_trace { ($($arg:tt)+) => { $crate::log!(reds, trace, $($arg)+) } }

#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ftlog_error { ($($arg:tt)+) => { $crate::log!(ftlog, error, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ftlog_warn { ($($arg:tt)+) => { $crate::log!(ftlog, warn, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ftlog_info { ($($arg:tt)+) => { $crate::log!(ftlog, info, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ftlog_life { ($($arg:tt)+) => { $crate::log!(ftlog, life, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ftlog_pipe { ($($arg:tt)+) => { $crate::log!(ftlog, pipe, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ftlog_hook { ($($arg:tt)+) => { $crate::log!(ftlog, hook, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ftlog_debug { ($($arg:tt)+) => { $crate::log!(ftlog, debug, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ftlog_trace { ($($arg:tt)+) => { $crate::log!(ftlog, trace, $($arg)+) } }

#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ext_error { ($($arg:tt)+) => { $crate::log!(ext, error, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ext_warn { ($($arg:tt)+) => { $crate::log!(ext, warn, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ext_info { ($($arg:tt)+) => { $crate::log!(ext, info, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ext_life { ($($arg:tt)+) => { $crate::log!(ext, life, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ext_pipe { ($($arg:tt)+) => { $crate::log!(ext, pipe, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ext_hook { ($($arg:tt)+) => { $crate::log!(ext, hook, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ext_debug { ($($arg:tt)+) => { $crate::log!(ext, debug, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_ext_trace { ($($arg:tt)+) => { $crate::log!(ext, trace, $($arg)+) } }

macro_rules! scoped {
    ($name:ident, $color:literal, $($level:ident = $mac:ident),+ $(,)?) => {
        // some scopes (e.g. `ext`) are only used by the `hook` feature
        #[allow(dead_code, unused_imports)]
        pub mod $name {
            pub fn color() -> &'static str { if $crate::config::colors() { $color } else { "" } }
            pub fn reset() -> &'static str { if color().is_empty() { "" } else { "\x1b[0m" } }

            $(pub use $crate::$mac as $level;)+
        }
    };
}

// yellow
#[rustfmt::skip]
scoped!(red4, "\x1b[33m",
    error = __yap_red4_error,
    warn = __yap_red4_warn,
    info = __yap_red4_info,
    life = __yap_red4_life,
    pipe = __yap_red4_pipe,
    hook = __yap_red4_hook,
    debug = __yap_red4_debug,
    trace = __yap_red4_trace,
);
// red
#[rustfmt::skip]
scoped!(reds, "\x1b[31m",
    error = __yap_reds_error,
    warn = __yap_reds_warn,
    info = __yap_reds_info,
    life = __yap_reds_life,
    pipe = __yap_reds_pipe,
    hook = __yap_reds_hook,
    debug = __yap_reds_debug,
    trace = __yap_reds_trace,
);
// grey
#[rustfmt::skip]
scoped!(ftlog, "\x1b[90m",
    error = __yap_ftlog_error,
    warn = __yap_ftlog_warn,
    info = __yap_ftlog_info,
    life = __yap_ftlog_life,
    pipe = __yap_ftlog_pipe,
    hook = __yap_ftlog_hook,
    debug = __yap_ftlog_debug,
    trace = __yap_ftlog_trace,
);
// purple
#[rustfmt::skip]
scoped!(ext, "\x1b[35m",
    error = __yap_ext_error,
    warn = __yap_ext_warn,
    info = __yap_ext_info,
    life = __yap_ext_life,
    pipe = __yap_ext_pipe,
    hook = __yap_ext_hook,
    debug = __yap_ext_debug,
    trace = __yap_ext_trace,
);
