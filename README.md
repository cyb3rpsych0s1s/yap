# yap

![Cyberpunk 2077 version compatibility][patch-badge]
[![download][download-badge]][download]
[![release][release-badge]][actions]
[![build][build-badge]][actions]

Logging plugin for [Cyberpunk 2077][cyberpunk] modders,
built on [RED4ext][red4ext].

It gives redscript mods leveled logging straight into RED4ext's log files,
each line tagged with the script function that emitted it.

## features

This plugin comes in 2 flavors.

### a version without any hook with basic features

- [x] `Life`, `Trace`, `Debug`, `Info`, `Warn` and `Error`
  callable from any script, prefixed with the calling `Class::Func`.
- [x] follows RED4ext's `[logging] level` from `red4ext/config.ini`,
  in every build.

### a version with hooks with additional features

- [x] captures the game's own `FTLog` calls.
- [x] mirrors every other RED4ext plugin's log lines
  into yap's log as `[<plugin>] …`.

## download

Grab the [latest release][download], then unzip it into your game folder.
Requires [RED4ext][red4ext] and [redscript][redscript].

| Archive | Contains |
| --- | --- |
| `Yap-windows-<tag>.zip` | all of the above, recommended |
| `Yap-windows-<tag>-NO_HOOK.zip` | script API only, no detours |

Each comes with a matching `-PDB.zip` for crash reports.

## usage

```swift
module MyMod
import Yap.Info

public class MySystem extends ScriptableSystem {
    private func OnAttach() {
        Info("hello from MyMod");
    }
}
```

Lands in `<game>/red4ext/logs/yap-*.log` as:

```text
🔷 [ reds ] MyMod.MySystem::OnAttach hello from MyMod
```

Set `level = "debug"` (or `"trace"`) in `red4ext/config.ini`
to also see `Debug` / `Trace` lines.

## output

Each kind of line in `just log yap`,
without RED4ext's `[timestamp] [thread] [yap]` prefix:

```text
# scripts (Yap.*)
[error   ] 🔴 [ reds ] Yap.YapService::OnLoad Error WaWaWaWa
[warning ] 🔶 [ reds ] Yap.YapService::OnLoad Warn WaWaWaWa
[info    ] 🔷 [ reds ] Cat.CatServiceDebug::OnLoad calling Yap from Cat
[debug   ] 🌱 [ reds ] Yap.YapService::OnLoad Service/OnLoad
[debug   ] 🔹 [ reds ] Yap.YapService::OnLoad Debug WaWaWaWa
[trace   ] • [ reds ] Yap.YapService::OnLoad Trace WaWaWaWa
# FTLog (hooks), without caller when called from native code
[info    ] [FTLog] Yap.YapService::OnLoad FTLog WaWaWaWa
[info    ] [FTLog] Audioware initialized successfully!
# other plugins (hooks)
[info    ] [octopak] on RTTI post register
[info    ] [cat] 🔷 [ red4 ] clear registry
# yap itself
[debug   ] 🌱 [ red4 ] Plugin/OnLoad
[info    ] 🔷 [ red4 ] attached 24/24 Logger hooks, skipped: []
```

Debug builds also color them:

| Part | Color |
| --- | --- |
| `[ red4 ]` | yellow |
| `[ reds ]` | red |
| `[FTLog]`, `Class::Func` caller | grey |
| `[<plugin>]` | purple |

## build

Requires [just][just]. Set `GAME_DIR` in a `.env`
if the game isn't in Steam's default folder.

- `just dev Yap`: release build, deployed to the game.
- `just dev Yap no-default`: same, `NO_HOOK` flavor.
- `just dbg Yap`: debug build with the `Debug.reds` harness
  (requires [Codeware][codeware]).
- `just log yap`: pipe the latest log file to the terminal.

## credits

- [@jekky][jekky]: author of [red4ext-rs][red4ext-rs]
  and [redscript][redscript].
- [@WopsS][wopss]: author of [RED4ext][red4ext].

[patch-badge]: https://img.shields.io/badge/Cyberpunk_2077-patch_2.31-yellow
[download-badge]: https://img.shields.io/github/v/release/cyb3rpsych0s1s/yap?display_name=tag&label=Download
[release-badge]: https://github.com/cyb3rpsych0s1s/yap/actions/workflows/release.yml/badge.svg
[build-badge]: https://github.com/cyb3rpsych0s1s/yap/actions/workflows/quality.yml/badge.svg
[download]: https://github.com/cyb3rpsych0s1s/yap/releases/latest
[actions]: https://github.com/cyb3rpsych0s1s/yap/actions
[cyberpunk]: https://www.cyberpunk.net/us/en/
[red4ext]: https://github.com/WopsS/RED4ext
[red4ext-rs]: https://github.com/jac3km4/red4ext-rs
[redscript]: https://github.com/jac3km4/redscript
[codeware]: https://github.com/psiberx/cp2077-codeware
[just]: https://github.com/casey/just
[jekky]: https://github.com/jac3km4
[wopss]: https://github.com/WopsS
