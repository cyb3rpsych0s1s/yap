include!(concat!(env!("OUT_DIR"), "/version.rs"));

#[cfg(all(windows, feature = "plugin", feature = "hook"))]
mod hook;

#[allow(dead_code, reason = "crate profile")]
mod config;
mod log;

#[cfg(all(windows, feature = "plugin"))]
pub struct Yap;

#[cfg(all(windows, feature = "plugin"))]
impl red4ext_rs::Plugin for Yap {
    const NAME: &'static red4ext_rs::U16CStr = red4ext_rs::wcstr!(env!("CARGO_PKG_NAME"));
    const AUTHOR: &'static red4ext_rs::U16CStr = red4ext_rs::wcstr!("Roms1383");
    const VERSION: red4ext_rs::SemVer = PLUGIN_VERSION;
    #[cfg_attr(not(feature = "hook"), allow(unused_variables))]
    fn on_load(env: &red4ext_rs::SdkEnv) {
        // first, so that every line below already follows RED4ext's log level
        crate::config::init();
        crate::log::red4::life!("Plugin/OnLoad");
        #[cfg(feature = "hook")]
        hook::attach_hook(env);
        red4ext_rs::RttiRegistrator::add(Some(register), Some(post_register));
    }
    #[cfg_attr(not(feature = "hook"), allow(unused_variables))]
    fn on_unload(env: &red4ext_rs::SdkEnv) {
        crate::log::red4::life!("Plugin/OnUnload");
        #[cfg(feature = "hook")]
        hook::detach_hook(env);
    }
    fn exports() -> impl red4ext_rs::Exportable {
        use red4ext_rs::exports;

        exports![
            script_log!(c"Yap.Life", debug, '🌱'),
            script_log!(c"Yap.Trace", trace, '•'),
            script_log!(c"Yap.Debug", debug, '🔹'),
            script_log!(c"Yap.Info", info, '🔷'),
            script_log!(c"Yap.Warn", warn, "🔶"),
            script_log!(c"Yap.Error", error, "🔴"),
        ]
    }
}

/// global `func <name>(msg: String)` exposed to scripts, emitted regardless of the build profile:
/// RED4ext's log level filters them.
///
/// Same as [`red4ext_rs::global`] but keeps the [`StackFrame`](red4ext_rs::types::StackFrame),
/// to log the calling script function like the `FTLog` hook does.
#[cfg(all(windows, feature = "plugin"))]
macro_rules! script_log {
    ($name:literal, $level:ident, $icon:literal) => {{
        use red4ext_rs::{
            GlobalExport, GlobalMetadata, VoidPtr,
            types::{IScriptable, RedString, StackFrame},
        };

        extern "C" fn native_impl(_: &IScriptable, frame: &mut StackFrame, _: VoidPtr, _: i64) {
            // natives run on their caller's frame: read it before consuming the argument
            let from = crate::config::caller(Some(frame.func()));
            let msg = unsafe { frame.get_arg::<RedString>() };
            unsafe { frame.step() };
            crate::log::emit!($level, $icon, reds, "{from}{msg}");
        }
        /// only gives RTTI the `String -> Void` signature
        fn signature(_: RedString) {}

        GlobalExport(GlobalMetadata::new($name, native_impl, &signature))
    }};
}
#[cfg(all(windows, feature = "plugin"))]
use script_log;

#[allow(dead_code, reason = "crate profile")]
extern "C" fn register() {
    crate::log::red4::life!("Plugin/Register");
}
/// Once RTTI registered.
#[allow(dead_code, reason = "crate profile")]
extern "C" fn post_register() {
    crate::log::red4::life!("Plugin/PostRegister");
}

#[cfg(all(windows, feature = "plugin"))]
red4ext_rs::export_plugin_symbols!(Yap);
