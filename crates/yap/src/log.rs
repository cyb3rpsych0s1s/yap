use crate::macros::scoped;

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

#[doc(hidden)]
#[macro_export]
macro_rules! __yap_chan_error { ($($arg:tt)+) => { $crate::log!(chan, error, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_chan_warn { ($($arg:tt)+) => { $crate::log!(chan, warn, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_chan_info { ($($arg:tt)+) => { $crate::log!(chan, info, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_chan_life { ($($arg:tt)+) => { $crate::log!(chan, life, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_chan_pipe { ($($arg:tt)+) => { $crate::log!(chan, pipe, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_chan_hook { ($($arg:tt)+) => { $crate::log!(chan, hook, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_chan_debug { ($($arg:tt)+) => { $crate::log!(chan, debug, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_chan_trace { ($($arg:tt)+) => { $crate::log!(chan, trace, $($arg)+) } }

#[doc(hidden)]
#[macro_export]
macro_rules! __yap_cet_error { ($($arg:tt)+) => { $crate::log!(cet, error, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_cet_warn { ($($arg:tt)+) => { $crate::log!(cet, warn, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_cet_info { ($($arg:tt)+) => { $crate::log!(cet, info, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_cet_life { ($($arg:tt)+) => { $crate::log!(cet, life, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_cet_pipe { ($($arg:tt)+) => { $crate::log!(cet, pipe, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_cet_hook { ($($arg:tt)+) => { $crate::log!(cet, hook, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_cet_debug { ($($arg:tt)+) => { $crate::log!(cet, debug, $($arg)+) } }
#[doc(hidden)]
#[macro_export]
macro_rules! __yap_cet_trace { ($($arg:tt)+) => { $crate::log!(cet, trace, $($arg)+) } }

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
// cyan
#[rustfmt::skip]
scoped!(chan, "\x1b[36m",
    error = __yap_chan_error,
    warn = __yap_chan_warn,
    info = __yap_chan_info,
    life = __yap_chan_life,
    pipe = __yap_chan_pipe,
    hook = __yap_chan_hook,
    debug = __yap_chan_debug,
    trace = __yap_chan_trace,
);
// blue
#[rustfmt::skip]
scoped!(cet, "[34m",
    error = __yap_cet_error,
    warn = __yap_cet_warn,
    info = __yap_cet_info,
    life = __yap_cet_life,
    pipe = __yap_cet_pipe,
    hook = __yap_cet_hook,
    debug = __yap_cet_debug,
    trace = __yap_cet_trace,
);
