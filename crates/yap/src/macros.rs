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
    // the game's `LogChannel*`: the line carries its own `[LogChannel:<channel>]` tag
    ($level:ident, $icon:literal, chan, $($arg:tt)+) => {{
        let icon = $icon.to_string();
        $crate::__private::red4ext_rs::log::$level!(
            "{icon}{}{}",
            if icon.is_empty() { "" } else { " " },
            format_args!($($arg)+)
        )
    }};
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

/// Logs through one of yap's scopes (`red4`, `reds`, `ftlog`, `ext`), e.g.
/// `yap::log!(red4, info, "loaded {}", x)` or `yap::log!(reds, error, hint: "check the path", "unable to read {}", p)`.
///
/// Levels: `error` and `warn` (both accept a leading `hint: "..."` or `hint: CONST`), `info`, `pipe`, `life`, `hook`, `debug`, `trace`.
// gated by RED4ext's log level, which drops these lines anyway: skips formatting them
#[macro_export]
macro_rules! log {
    // a literal hint goes through `format_args!` for inline captures (`hint: "check {path}"`),
    // so it must come before the `expr` arm, which a literal matches too
    ($scope:ident, error, hint: $hint:literal, $($arg:tt)+) => {
        $crate::__emit!(error, "🔴", $scope, "{}\n   💡 hint: {}", format_args!($($arg)+), format_args!($hint))
    };
    ($scope:ident, error, hint: $hint:expr, $($arg:tt)+) => {
        $crate::__emit!(error, "🔴", $scope, "{}\n   💡 hint: {}", format_args!($($arg)+), $hint)
    };
    ($scope:ident, error, $($arg:tt)+) => {
        $crate::__emit!(error, "🔴", $scope, $($arg)+)
    };
    ($scope:ident, warn, hint: $hint:literal, $($arg:tt)+) => {
        $crate::__emit!(warn, "🔶", $scope, "{}\n   💡 hint: {}", format_args!($($arg)+), format_args!($hint))
    };
    ($scope:ident, warn, hint: $hint:expr, $($arg:tt)+) => {
        $crate::__emit!(warn, "🔶", $scope, "{}\n   💡 hint: {}", format_args!($($arg)+), $hint)
    };
    ($scope:ident, warn, $($arg:tt)+) => {
        $crate::__emit!(warn, "🔶", $scope, $($arg)+)
    };
    ($scope:ident, info, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::INFO) {
            $crate::__emit!(info, '🔷', $scope, $($arg)+)
        }
    };
    ($scope:ident, life, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::DEBUG) {
            $crate::__emit!(debug, '🌱', $scope, $($arg)+)
        }
    };
    ($scope:ident, pipe, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::INFO) {
            $crate::__emit!(info, "", $scope, $($arg)+)
        }
    };
    ($scope:ident, hook, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::DEBUG) {
            $crate::__emit!(debug, '🪝', $scope, $($arg)+)
        }
    };
    ($scope:ident, debug, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::DEBUG) {
            $crate::__emit!(debug, '🔹', $scope, $($arg)+)
        }
    };
    ($scope:ident, trace, $($arg:tt)+) => {
        if $crate::config::enabled($crate::config::TRACE) {
            $crate::__emit!(trace, '•', $scope, $($arg)+)
        }
    };
}

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
pub(crate) use scoped;
