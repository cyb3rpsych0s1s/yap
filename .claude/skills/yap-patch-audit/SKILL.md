---
name: yap-patch-audit
description: Diagnose and fix the `yap` plugin (crates/yap) after a Cyberpunk 2077 game patch, a RED4ext / red4ext-rs / Codeware update, or a Rust toolchain bump. Use when yap stops loading, crashes the game, its FTLog script lines or its RED4ext `Logger` detours / mirrored `[plugin]` lines stop appearing, or before bumping any of those versions.
---

# yap patch-day audit

`yap` depends on several version-specific assumptions: undocumented addresses, script VM bytecode, private struct layouts and ABI details. Each one is listed below with **where it lives**, **how to check it** and **how to fix it**. Work through **Triage** first, then only the breakpoints the symptoms point to.

## Baseline (last known good)

Update this table after every successful fix.

| Component | Version |
|---|---|
| Cyberpunk 2077 | 2.31 (exe file version 3.0.5294808) |
| RED4ext | 1.30.0 (SDK API **v1**) |
| red4ext-rs | git rev `0f77591b67aec31893257ff753fa7466c388830a` (`crates/yap/Cargo.toml`) |
| Codeware | 1.20.3 |
| rustc | 1.99.0 (`rust-version` in root `Cargo.toml`) |
| libloading | 0.8 |
| FTLog address hash | `0xCDF52009` = `3455393801`, DB entry offset `0001:00f5ffe4` → `0x140F60FE4` |
| LogChannel / LogChannelError address hashes (one shared native) | `0xB7C91CD0` = `3083410640`, `0x63201EDA` = `1663049434`, DB entry offset `0001:00d269d8` → `0x140D279D8` |
| LogChannelWarning address hash | `0xA9621FA6` = `2841780134`, DB entry offset `0001:0205473c` → `0x14205573C` |

## What yap hooks

| File | What it does | Depends on |
|---|---|---|
| `crates/yap/src/hook/ftlog.rs` (**`hook` feature**) | Detours the **game's** native `FTLog` (by address hash) and reads its `script_ref<String>` argument straight from script bytecode to log `Class::Func msg` | game exe, script VM, red4ext-rs `StackFrame` API |
| `crates/yap/src/hook/logchannel.rs` (**`hook` feature**) | Detours the **game's** natives `LogChannel`, `LogChannelWarning` and `LogChannelError(channel: CName, text: script_ref<String>)` (by address hash, 2 hooks: `LogChannel` and `LogChannelError` share one native, told apart by the callee's name) and logs `[<native>:<channel>] Class::Func msg` as info / warn / error. Reuses `ftlog.rs`'s bytecode helpers | game exe, script VM, red4ext-rs `StackFrame` API |
| `crates/yap/src/hook/logger.rs` (**`hook` feature**) | Detours all 24 entries of **RED4ext's** `v1::Logger` table, mirrors other plugins' lines into yap's log as `[<plugin>] msg` (purple), skips levels filtered by `red4ext/config.ini` | RED4ext.dll + SDK v1 layouts, red4ext-rs private layout, UCRT, kernel32, `c_variadic` |
| `crates/yap/reds/Log.reds` | Declares the public script API: `Yap.Life/Trace/Debug/Info/Warn/Error(msg: String)` natives (no `Critical`) | `src/lib.rs` exports |
| `crates/yap/reds/Debug.reds` | Debug harness: `YapService` / `YapSystem`, `Session/*` callbacks, calls `FTLog`, `LogChannel`, `LogChannelWarning`, `LogChannelError` and Codeware's `Print` / `ModLog` (which call `LogChannel` natively, exercising the native-caller path) with `"WaWaWaWa"` as a smoke test, reports via `Life` | Codeware, game script API, `Log.reds` |
| `crates/yap/reds/DebugNatives.reds` | Root-namespace (no `module`) re-declarations of the game's `LogChannel*(channel: CName, text: script_ref<String>)` natives, which redscript needs before `Debug.reds` can call them. Debug only: `just ci` → `just no-debug` strips every `Debug*.reds` from releases, so they never clash with another mod's declarations | game script API |
| `crates/yap/src/lib.rs` | Plugin entry, and the `Yap.*` log natives. These are registered through a local `script_log!` that mirrors `global!` (`GlobalMetadata::new`), so it can prefix the script caller (`Class::Func`, read from `StackFrame::func()`). They call the **ungated** `log::emit!`, so only RED4ext's `config.ini` level filters them, even in release builds | red4ext-rs |
| `crates/yap/src/log.rs` | `red4`/`reds`/`ftlog`/`ext` scoped log macros, ANSI colours (on by default; `init()` first reads the opt-outs: `[logging] colors` in yap's own optional `red4ext/plugins/yap/config.ini`, else `NO_COLOR`), and the config.ini level gate: `init()` reads `[logging] level` at the start of `on_load`, and every macro except `error!`/`warn!` checks `enabled(level)` | game layout (`current_exe`), config.ini format |

yap doesn't depend on any particular mod. FTLog lines come from any script that calls the game's `FTLog` (the game's own scripts, or mods such as `cat`). Mirrored lines come from any installed RED4ext plugin, with no per-plugin special case. In-game testing only needs at least one other plugin that logs at runtime.

## Triage: read the logs first

Commands: `just dbg Yap` (debug build + deploy), launch the game, then `just log yap`. All hooks (`src/hook/`, FTLog, `Logger` detours, mirroring, self-test, and the `libloading` dependency) are behind the **default** `hook` cargo feature. Building with `--no-default-features` (release: `just dev Yap no-default`) leaves only the config.ini level, the formatting macros and the `Yap.*` script natives. In that variant expected lines 2–4, 6, 7, 8 and 10 below don't appear, and sections A1–A4, B1–B4, C1, C2's hook parts and D1–D3 don't apply. Also open `<game>/red4ext/logs/red4ext.log`, which records plugin load failures, API version mismatches and hook errors.

Expected `yap-*.log` start, in order, with `level = "debug"` (the self-test needs a debug build; every other line only depends on the level). Lines marked *(trace)* are `red4::trace!` diagnostics. They only appear on a **trace run**: `level = "trace"` in `<game>/red4ext/config.ini`, in any build. Set it for the diagnostic run, then restore the previous level.

1. `🌱 Plugin/OnLoad`
2. `attached native internal hook for FTLog`, then `… for LogChannel/LogChannelError` and `… for LogChannelWarning`
3. *(trace)* `RED4ext.dll image at 0x…..0x…`
4. *(trace)* `RED4ext_ResolveAddress(0xCDF52009) = 0x… (addr_hashes::resolve = 0x…, equal: true)`
5. `RED4ext log level: <Level> (<game>\red4ext\config.ini)` (the path only at `debug`/`trace`)
6. *(trace)* `Logger table 0x… (Sdk at SdkEnv word #1)` (word #0 is also valid)
7. `attached 24/24 Logger hooks, skipped: []`
8. Self-test: `self-test <Entry>` for every entry at or above the configured level, with `F`/`WF` ones reading `…: answer = 42`
9. Later: `🪝 Yap.YapService::OnLoad WaWaWaWa`, `🌱 [ reds ] Session/*`, and mirrored `[<plugin>] …` lines from any other installed RED4ext plugin (e.g. `[cat] …`, `[octopak] …`)
10. On exit: `detached 24/24 Logger hooks`, `detached native internal hook for LogChannelWarning`, `… for LogChannel/LogChannelError`, `detached native internal hook for FTLog`

| Symptom | Go to |
|---|---|
| No `yap-*.log` at all, or red4ext.log says yap failed to load / unsupported API | B1, C1 |
| Game crashes at startup right after `Plugin/OnLoad` | A1 (resolve of a missing hash), then A2 |
| (trace run) Line 4 shows `equal: false` or a value of `0` | A1 |
| `WaWaWaWa` / `Class::Func` FTLog lines missing, garbled or crashing in-game | A2, A3, A4 |
| Crash in `yap.dll` reading `0x10` right after the `LogChannel*` lines (Codeware's `Print` / `ModLog`), or when a native plugin calls `Yap.*` | a `frame.func()` on a frame from `Function::Execute`, whose function is null: always go through `config::func` (C2) |
| Crash at startup in `Cyberpunk2077.exe+0x1fe0fe` (read at `0x14`, null `CString`) right after a `[LogChannel:DEBUG] …` line | `crates/yap/reds/DebugNatives.reds` declares `text` with the wrong type: it must be `script_ref<String>`, since the natives read it as a ref (A6) |
| `[LogChannel*:DEBUG] … WaWaWaWa` lines (or Codeware's `Print` / `ModLog` ones, without caller) missing (trace run: look for `<native>:<channel>: no text from native caller` / `dropped empty or invalid text`), at the wrong level, garbled or crashing, or `unable to resolve LogChannel…` | A6 |
| `unable to open RED4ext.dll` / `unable to open yap.dll` | B1 |
| `unable to locate the Logger table in SdkEnv` | C1, B2, B3 |
| `attached n/24` with n < 24, or a non-empty `skipped` list | B2, B4 |
| Self-test lines missing, at the wrong level, or with the wrong variant name | B2 |
| `F`/`WF` self-test lines show the raw format (`%s = %d`) | D2 |
| `unable to read [logging] level` / `unable to locate the config.ini` | B5 |
| Mirrored lines show `[0x…]` or a DLL stem instead of the plugin name | B3 |
| `Session/*` / `System/*` lines missing, or redscript compile errors for `Log.reds` / `Debug.reds`, or other mods failing to resolve `Yap.Info` & co. | A5, C3 |
| `Yap.*` script lines lose their `Class::Func` caller, show a wrong one, or crash when called | C2 |
| Build fails after a red4ext-rs bump or toolchain change | C1–C3, D1 |

## A. Game patch (Cyberpunk2077.exe)

### A1. FTLog address hash: `FTLOG_HASH` in `ftlog.rs` (also used by `sample_resolve` in `logger.rs`)
- **Assumes:** RED4ext's address database `<game>/bin/x64/cyberpunk2077_addresses.json` still has hash `3455393801` (`0xCDF52009`), and it still points at the native `FTLog` implementation. RED4ext regenerates this file for each patch. The entry has **no symbol name**, only `hash`, `secondary hash` and `offset`.
- **Check:** `grep -o '{"hash": "3455393801"[^}]*}' "<game>/bin/x64/cyberpunk2077_addresses.json"`. The file is one ~130 MB line, so use `grep -o` and not a line-based tool. Absolute address = `0x140000000 + 0x1000 (code constant offset) + segment offset`. Compare with log line 4 (needs a trace run).
- **Fix:** find the new hash of the function the game registers as the native of global `FTLog`:
  - Enumerate RTTI at `post_register`: the global function named `FTLog` (`RttiSystem::get()` → global functions) holds its native handler pointer. Convert that pointer to `offset = addr - base - 0x1000` and search the JSON for that `"offset": "0001:<hex>"`. That gives the new `hash`.
  - Optionally switch `ftlog.rs` to resolve the target **from RTTI** instead of a hash, which makes it patch-proof. Keep the hash path as a fallback.
  - Update `FTLOG_HASH` and its `// 0x… (2.31)` comment. `logger.rs` reuses it only for the `RED4ext_ResolveAddress` equality log.
- **Guard:** if the hash resolves to `0`, `target()` returns `None`, and `attach_hook` logs `unable to resolve FTLog (0xCDF52009), not hooked` instead of hooking address 0. If RED4ext itself aborts on an unknown hash, the game crashes before that guard runs (see the triage row "crashes right after `Plugin/OnLoad`").

### A2. Native function signature: `type Native` in `ftlog.rs`
- **Assumes:** `(IScriptable*, CStackFrame*, void* out, void* a4)`, the standard script-native ABI.
- **Check:** if FTLog hooks attach but the game crashes when FTLog is called, the ABI may have changed. Compare with other natives in RED4ext.SDK (`CBaseFunction` / `ScriptingFunction_t`).
- **Fix:** match the new signature in `Native`, the `hooks!` declaration and `detour`.

### A3. Script bytecode layout: constants in `ftlog.rs`
- **Assumes:**
  - `REF_OPCODE = 0x66` wraps every `script_ref<T>` argument, followed by a raw `CBaseRTTIType*` (8 bytes, `REF_INSTR_SIZE = 1 + 8`) and then the inner expression.
  - `CStackFrame::code` is its first field and points at the first argument. It's read by the `code()` helper, because red4ext-rs keeps it private and `Instr` is sealed.
  - The call site is an `InvokeStatic` found at `-CALL_INSTR_SIZE - OPCODE_SIZE` from red4ext-rs (`types/bytecode.rs`).
  - `RedString` is the `String` RTTI type (`string_type()`).
- **Check:** in a debug build, temporarily dump the first ~16 bytes at `code` from the detour, to a file and not the log (logging inside a hook is fine here, but not from `logger.rs` detours). Compare them with the redscript compiler's opcode table. Symptoms are missing lines (`is_string_ref` false), garbage text, or a crash in `get_arg`.
- **Fix:** update `REF_OPCODE` / `REF_INSTR_SIZE`, or read the operand differently. If `InvokeStatic` changed, a red4ext-rs bump is likely needed (C2).

### A4. `FTLog` script declaration
- **Assumes:** the game's scripts still declare `native func FTLog(value: script_ref<String>)`, a global with that exact name (`NAME` in `ftlog.rs`, compared through `CName` in `detour`).
- **Check:** the wolvenkit MCP `script_api_index` / `read_script`, or search the decompiled `r6/cache` scripts for `FTLog`.
- **Fix:** follow a rename (`NAME`) or an argument type change (the `script_ref` / `RedString` handling in `detour`).

### A5. Codeware / script API used by `Debug.reds`
- **Assumes:** the public `Yap.*` natives in `Log.reds` match the `global!(c"Yap.<Name>", …)` exports in `src/lib.rs` (name and `String` argument), plus `ScriptableService` callbacks (`OnLoad`, `OnInitialize`, `OnUninitialize`, `OnReload`), `GameInstance.GetCallbackSystem()`, `RegisterCallback`, `GameSessionEvent`, the event names `Session/BeforeStart|Start|Ready|BeforeEnd|End`, and `ScriptableSystem.OnAttach/OnDetach`.
- **Check:** the redscript compile errors in `red4ext/logs` or `r6/logs/redscript_rCURRENT.log`, and the Codeware changelog.
- **Fix:** update `Debug.reds` (or `Log.reds` together with the matching exports in `src/lib.rs`, since they're a public API other mods compile against), then deploy with `just reload Yap`.

### A6. `LogChannel*` natives: `logchannel.rs`
- **Assumes:**
  - The address hashes in `SHARED_HASHES` / `WARNING_HASHES` still resolve. `LogChannel` and `LogChannelError` share one native (identical code folding), so they get **one** hook (MinHook can't hook an address twice). The detour tells them apart by the `InvokeStatic` callee's `short_name`, and skips any other native that shares the stub. If a patch splits them, add a third hook.
  - The script declarations are `LogChannel*(channel: CName, text: script_ref<String>)`. From scripts, the `text` expression may come wrapped in `REF_OPCODE` (if it's declared `script_ref<String>`): the detour unwraps it when present. For native callers (`Execute(CStack)`, e.g. Codeware's `Red::Log::Channel`, which passes a `script_ref:String`), `text` is always read as a `ScriptRef<RedString>`, because the natives themselves read it as a ref (disassembly: the arg is read with both out-pointers set). A caller passing anything else would crash the game anyway.
  - For native callers the level comes from the frame's function name (`config::func`), which is null under `Execute`, so it falls back to `LogChannel` (info).
  - Native callers (`Function::Execute`, e.g. Codeware's `Print` / `ModLog`) run on a frame whose `func` is **null**. red4ext-rs's `StackFrame::func()` doesn't check for that, so every read goes through `config::func` (word 1 of `CStackFrame`, after `code`).
- **Check:** same as A1 for the hashes (`grep -o '{"hash": "3083410640"[^}]*}'` …): the `LogChannel*` entries have no symbol name either. Find the new native through RTTI (`get_function(CName::new("LogChannel"))`'s native handler), like A1. Use the `Debug.reds` smoke lines for the levels and the script path, and any Codeware-based plugin for the native path.
- **Fix:** update the hashes and their `// 0x… (2.31)` comments, `Channel` names or levels, or the argument reads in `capture`. If the natives' signatures change, update `crates/yap/reds/DebugNatives.reds` too.

## B. RED4ext update (usually shipped right after a game patch)

### B1. Module, exports and API version
- **Assumes:** RED4ext is loaded as `RED4ext.dll` (`MODULE`), it exports `RED4ext_ResolveAddress(u32) -> usize`, and it still accepts plugins built on SDK API **v1** (`Supports()` from red4ext-rs).
- **Check:** red4ext.log at startup, and the PE export table of the new `RED4ext.dll`.
- **Fix:** update `MODULE`. If the API version moved to v2, bump red4ext-rs (C) and re-derive B2–B3 from the new headers.

### B2. `Logger` table layout: `ENTRIES_LEN`, `taps!` and `self_test` in `logger.rs`
- **Assumes:** `RED4ext/Api/v1/Logger.hpp` has 24 function pointers ordered `Level{plain, F, W, WF}` for `Trace, Debug, Info, Warn, Error, Critical`. Index = `level * 4 + variant`. Signatures are `(PluginHandle, const char*)`, `(PluginHandle, const char*, ...)`, `(PluginHandle, const wchar_t*)` and `(PluginHandle, const wchar_t*, ...)`.
- **Check:** diff `deps/RED4ext.SDK/include/RED4ext/Api/v1/Logger.hpp` in the red4ext-rs checkout (`~/.cargo/git/checkouts/red4ext-rs-*/<rev>/`) against upstream RED4ext.SDK. The self-test prints every entry's name at its level, so a shuffled table shows up as names at wrong levels or wrong formatting.
- **Fix:** update the named constants in `logger.rs`, which must match the header:
  - `LOGGER_LEVELS` / `VARIANTS` (`ENTRIES_LEN` is their product);
  - `PLAIN` / `FORMAT` / `WIDE` / `WIDE_FORMAT`, which the `self_test` dispatch matches on;
  - the `taps!` invocation (level constant, name, variant order);
  - the level constants in `crates/yap/src/log.rs` (`TRACE` … `CRITICAL`), shared by `taps!`, `read_level` and `enabled()`.

  `find_logger` refuses a `Logger` whose size isn't `ENTRIES_LEN` words, so a changed header shows up as `unable to locate the Logger table`.

### B3. `Sdk` / `PluginHandle` / `PluginInfo`
- **Assumes:**
  - `v1::Sdk` is `#[repr(C)] { runtime, logger, hooking, gameStates, scripts }`. yap reads it as red4ext-rs's own `internal::Sdk` and uses the named field `sdk.logger`, so a field reorder is picked up by bumping red4ext-rs.
  - `PluginHandle` is the plugin's `HMODULE` (`find_logger` matches yap's own module handle, from `libloading`, and the mirror uses it as a key).
  - `Query(PluginInfo*)` is a mandatory export, and `PluginInfo { sdk, name: *const wchar_t, author, version, runtime }` (`plugin_name`).
- **Check:** `Sdk.hpp`, `PluginHandle.hpp`, `PluginInfo.hpp` in the SDK. `find_logger` failing or names falling back to DLL stems or `0x…`.
- **Fix:** bump red4ext-rs if the SDK structs changed, and adjust the `PluginInfo` read in `plugin_name`.

### B4. Hooking semantics and where Logger lives
- **Assumes:**
  - `Hooking::Attach` (MinHook) patches the code immediately, during `on_load`.
  - All 24 entries are distinct functions **inside RED4ext.dll's image**: `find_logger` validates each pointer against `image_range` (PE `SizeOfImage`).
  - Duplicates (identical COMDAT folding) are skipped, because the same address can't be hooked twice.
- **Check:** `attached n/24, skipped: [...]`. If RED4ext moved logging into another DLL (e.g. a shared spdlog), the range check fails with `unable to locate the Logger table`.
- **Fix:** widen the range check to the module that actually owns the pointers (`GetModuleHandleExW` with `FROM_ADDRESS`, via libloading/kernel32). For folded entries, give one detour per address and derive the level another way, or accept the skip.

### B5. `config.ini` location and format: `init` / `read_level` / `read_key` / `enabled` in `crates/yap/src/log.rs`
- **Assumes:** the config is `<game>/red4ext/config.ini`, located with std only, so any install path works: `std::env::current_exe()` is `<game>/bin/x64/Cyberpunk2077.exe`, the game folder is `EXE_DEPTH` (3) ancestors up, then `RED4EXT_CONFIG`. A layout change shows up as `unable to locate RED4ext's red4ext/config.ini`. It's TOML-like with `[logging]` → `level = "<name>"`, names `trace|debug|info|warn(ing)|err(or)|critical|off`. Current docs: https://docs.red4ext.com/getting-started/configuration
- **Check:** the `RED4ext log level: …` line. Change `level`, relaunch, and confirm self-test lines below the level vanish.
- **Fix:** update the parser for a renamed key, section or value. If unreadable, yap must keep working without the early-out (`MIN_LEVEL` unset ⇒ every yap macro and `Logger` detour is enabled, RED4ext still filters downstream). This gate replaces any build-profile gating: yap's own debug/trace lines also show in release builds when `level` allows them. `read_key` also reads yap's own `[logging] colors`, so a parser change must keep both working. That file belongs to yap, so a RED4ext config change never affects it.

## C. red4ext-rs bump (`rev` in `crates/yap/Cargo.toml`)

### C1. Private `SdkEnv` layout: `find_logger`
- **Assumes:** `SdkEnv { handle: PluginHandle, sdk: Sdk }` is `SDK_ENV_WORDS` = 1 + `size_of::<internal::Sdk>()` words, with `handle` first or last (rustc may reorder). So the `Sdk` starts at word 1 or word 0. `find_logger` returns `None` if the size differs or yap's handle isn't at either end.
- **Check:** `src/lib.rs` `pub struct SdkEnv` in the new rev. Better: look for a public accessor to the `Sdk` / logger, which would make the pointer cast unnecessary.
- **Fix:** adapt the word mapping in `find_logger`, or switch to the public API.

### C2. Hook plumbing and StackFrame API
- **Assumes:**
  - `hooks!` expands to a `static mut HOOK: *mut Hook<O, R>` with a non-variadic `internal` trampoline.
  - `Hook::new(original, cb_ref, detour_ref)` is public (it's `#[doc(hidden)]`).
  - `SdkEnv::attach_hook` / `detach_hook` take `FnPtr` types, which aren't implemented for variadic fn pointers. That's why `narrow_f` / `wide_f` transmute to `Narrow`.
  - `red4ext_rs::internal::PluginInfo` is exported.
  - `CStackFrame::func` is word 1 of the frame (after `code`), and is null for natives called through `Function::Execute`. `config::func` reads it null-safely, instead of red4ext-rs's `StackFrame::func()`, which never checks. Replace it if a red4ext-rs bump adds a null-safe accessor.
  - `ftlog.rs` / `logchannel.rs` use `StackFrame::{instr_at, has_code, args_state, restore_args, step, get_arg}`, `InvokeStatic`, `CALL_INSTR_SIZE`, `OPCODE_SIZE` and `addr_hashes::resolve`.
  - `script_log!` in `crates/yap/src/lib.rs` mirrors red4ext-rs's `global!` expansion (`src/invocable.rs`) instead of calling it:
    - It builds `GlobalMetadata::new(name, handler, &signature)` itself. That function is `pub` but `#[doc(hidden)]`.
    - `handler` is a `FunctionHandler<IScriptable, VoidPtr>`, i.e. `extern "C" fn(&IScriptable, &mut StackFrame, VoidPtr, i64)`.
    - It reads its argument with `get_arg`, then calls `frame.step()`.
    - It does this so it can read the script caller first, through `StackFrame::func()`. A native runs on its caller's frame.
  - `crate::log::caller` formats that caller with `Function::{parent, short_name, flags().is_native()}`.
- **Check:** `cargo clippy -p yap -- -D warnings` after the bump. Diff `src/lib.rs` (`macro_rules! hooks`, `struct Hook`, `trait FnPtr`), `src/invocable.rs` (`macro_rules! global`, `GlobalMetadata::new`) and `src/types/stack.rs` / `bytecode.rs`.
- **Fix:** re-mirror the `hooks!` expansion in the `narrow_f` / `wide_f` arms and the `global!` expansion in `script_log!`, and adapt the renamed APIs.

### C3. `export_plugin_symbols!` / `Plugin` trait
- **Check:** `crates/yap/src/lib.rs` compiles, and `Query`/`Main`/`Supports` are still exported.

## D. Toolchain and OS

### D1. `c_variadic`
- **Assumes:** `unsafe extern "C" fn installed(handle, fmt, args: ...)` and `VaList` are stable (since rustc 1.99), and on win64 a `VaList` passed by value is the C `va_list` (a `char*`). `args.clone()` is used to format twice.
- **Fix:** if it regresses, pin the toolchain or go back to hooking only the non-variadic entries.

### D2. UCRT formatting: `printf()` / `format()` / `format_w()`
- **Assumes:** `ucrtbase.dll` is loaded and exports `__stdio_common_vsprintf` / `__stdio_common_vswprintf`, called with options `2 | 4` (standard snprintf behaviour + legacy wide specifiers, MSVC's default). A null buffer with count 0 returns the required length.
- **Check:** the warning `unable to resolve printf from ucrtbase.dll`, or `F`/`WF` self-test lines printing the raw format.
- **Fix:** resolve from `api-ms-win-crt-stdio-l1-1-0.dll` instead, or adjust the options.

### D3. kernel32
- **Assumes:** (`hook` feature only) `GetModuleFileNameW` is resolvable from `kernel32.dll`. It's used for the DLL-stem fallback of mirrored plugin names (`module_path` / `module_base` in `logger.rs`).

## Fix rules (always)

1. **Never crash the game.** Every failed assumption must turn into a `red4::warn!` plus a skip or a pass-through to the original function. Never call `unwrap` on something a patch can change.
2. **Never log from inside a `logger.rs` detour.** Every `red4::*` macro goes back through the `Logger` table. The detours call `cb` / `TARGET` only, and the `Guard` thread-local stops double mirroring.
3. Keep the original call path working: plain and wide entries forward the original pointer. `F`/`WF` must forward the **formatted** text as `"%s"` / `L"%ls"`, because the `...` arguments can't be forwarded as they are.
4. Update the version comments (e.g. `// 0x140F60FE4 (2.31)`) and the **Baseline** table above.

## Verification checklist

1. Both configurations: `cargo clippy -p yap -- -D warnings` and `cargo clippy -p yap --no-default-features -- -D warnings`, then `cargo build -p yap` and `cargo build -p yap --no-default-features`. `cargo tree -p yap -e normal --no-default-features` must not list `libloading`.
2. `just dbg Yap`, launch the game and load a save. Also do one run with `just dev Yap no-default` (release, no hooks): only `RED4ext log level: …` and the `Yap.*` / `Debug.reds` lines, with no FTLog attach, no `[FTLog]`, no mirrored lines and no self-test.
3. `just log yap` matches the expected start in **Triage** (lines 1–8, where 3, 4 and 6 only appear on a trace run), shows `WaWaWaWa`, and shows mirrored `[plugin]` lines with the name in purple (`\x1b[35m`) (unless colors are opted out: no `colors = false` in `red4ext/plugins/yap/config.ini`, no `NO_COLOR`).
4. Set `level = "info"` in `<game>/red4ext/config.ini`, relaunch, and check that `RED4ext log level: Info` appears, without the config path, and that no `Debug*` self-test lines do. Then restore the previous value.
4b. Optional diagnostic run: set `level = "trace"`, relaunch, and check that the *(trace)* lines 3, 4 (`equal: true`) and 6 appear. Then restore the previous value.
5. Any other plugin's own log (e.g. `just log cat`) shows its lines unchanged, without a prefix.
5b. The script API works: the lines from `Debug.reds` (via `Yap.Life`) appear with their caller, like FTLog lines: `🌱 [ reds ] Yap.YapService::OnLoad Service/OnLoad`, `… Yap.YapService::OnSession Session/Start`, `… Yap.YapSystem::OnAttach System/OnAttach`. A test call to `Yap.Info` / `Warn` / `Error` lands at `[info]` / `[warning]` / `[error]`, and in a release build `Yap.Debug` / `Life` still appear when `level = "debug"`.
6. Quit to desktop and check that `detached 24/24 Logger hooks`, the `LogChannel*` and `detached native internal hook for FTLog` lines appear, with no crash.
