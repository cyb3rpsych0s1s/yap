# Nexus page

Full description for the Nexus Mods page, in BBCode.
Paste the block below into the description editor in BBCode mode (the `[/]` toggle).

```bbcode
[center][size=6][b]yap[/b][/size]
[i]yet another logger... but with a twist![/i][/center]

[line]

[size=5][b]What is it?[/b][/size]

[b]yap[/b] is a small [url=https://www.nexusmods.com/cyberpunk2077/mods/2380]RED4ext[/url] plugin for modders
which can aggregate human-readable color-coded logs from [i]anywhere[/i] in a single unified file!

[b]Players[/b]: you should not need yap, unless a mod explicitly states it as requirement.

[size=5][b]Features[/b][/size]

It comes in 2 flavors. Choose one.

[size=4]the [b]NO_HOOK[/b] file[/size]
[list]
[*]new logging methods: [color=#f1c232][font=Roboto]Life[/font][/color], [color=#f1c232][font=Roboto]Trace[/font][/color], [color=#f1c232][font=Roboto]Debug[/font][/color], [color=#f1c232][font=Roboto]Info[/font][/color], [color=#f1c232][font=Roboto]Warn[/font][/color] and [color=#f1c232][font=Roboto]Error[/font][/color], callable from any script, prefixed with the calling [color=#f1c232][font=Roboto]Class::Func[/font][/color] or [color=#f1c232][font=Roboto]Func[/font][/color] if any.[/*]
[*]abides by RED4ext's [color=#f1c232][font=Roboto][logging] level[/font][/color] from [color=#f1c232][font=Roboto]red4ext\config.ini[/font][/color].[/*]
[/list]

[size=4]the standard file[/size]
[list]
[*]same as above, plus[/*]
[*]captures the game's own [color=#f1c232][font=Roboto]FTLog[/font][/color] calls.[/*]
[*]captures every RED4ext [color=#f1c232][font=Roboto]Logger[/font][/color] calls.[/*]
[/list]

The hooked flavor relies on game and RED4ext internals.
It is expected to break after a game patch,
but you can temporarily switch to [color=#f1c232][font=Roboto]NO_HOOK[/font][/color] version until yap is updated:
your mods' log calls keep working.

Given it aggregates all logs in one log file which can quickly bloats,
you might want to consider increasing [b]max_file_size[/b] in [color=#f1c232][font=Roboto]red4ext\config.ini[/font][/color]
or simply uninstall it whenever you don't need it.

[size=5][b]Installation[/b][/size]
Install with Vortex or MO2, or extract the archive into your game folder. You should end up with:
[code]Cyberpunk 2077/red4ext/plugins/yap/yap.dll
Cyberpunk 2077/r6/scripts/Yap/Log.reds[/code]

To uninstall, delete those two folders.

[size=5][b]For modders[/b][/size]
From your own module, import the functions you call, e.g. in [i]Cat[/i]:
[code]module Cat
import Yap.Info

public class CatServiceDebug extends ScriptableService {
    private cb func OnLoad() {
        Info("calling Yap from Cat");
    }
}[/code]

which lands in [color=#f1c232][font=Roboto]Cyberpunk 2077/red4ext/logs/yap-*.log[/font][/color] as:
[code][2026-10-09 18:12:56.460] [info    ] [ 48396] [yap] 🔷 [ reds ] Cat.CatServiceDebug::OnLoad calling Yap from Cat[/code]

Set [color=#f1c232][font=Roboto]level = "debug"[/font][/color] (or [color=#f1c232][font=Roboto]"trace"[/font][/color]) in [color=#f1c232][font=Roboto]red4ext/config.ini[/font][/color] to also see [color=#f1c232][font=Roboto]Debug[/font][/color] / [color=#f1c232][font=Roboto]Trace[/font][/color] lines.

Lines are colored with ANSI escape codes, which look like [color=#f1c232][font=Roboto][33m[/font][/color] noise in a plain text editor. To turn colors off, create [color=#f1c232][font=Roboto]Cyberpunk 2077/red4ext/plugins/yap/config.ini[/font][/color] with:
[code][logging]
colors = false[/code]
or set the [url=https://no-color.org]NO_COLOR[/url] environment variable.

What each kind of line looks like (hooked flavor):
[code][2026-10-09 18:12:56.277] [error   ] [ 48396] [yap] 🔴 [ reds ] Yap.YapService::OnLoad Error WaWaWaWa
[2026-10-09 18:12:56.277] [warning ] [ 48396] [yap] 🔶 [ reds ] Yap.YapService::OnLoad Warn WaWaWaWa
[2026-10-09 18:12:56.278] [debug   ] [ 48396] [yap] 🔹 [ reds ] Yap.YapService::OnLoad Debug WaWaWaWa
[2026-10-09 18:12:56.278] [info    ] [ 48396] [yap] [FTLog] Yap.YapService::OnLoad FTLog WaWaWaWa
[2026-10-09 18:12:51.888] [info    ] [ 48396] [yap] [octopak] on RTTI post register[/code]

Each main file comes with an optional [b]PDB[/b] file, only useful to analyze crash reports.

[size=5][b]Source code[/b][/size]
Open source on [url=https://github.com/cyb3rpsych0s1s/yap]GitHub[/url], where releases are built by CI.
Bug reports and suggestions are welcome there or in the Bugs tab.

[size=5][b]Disclaimer[/b][/size]

The hooks were implemented with partial assistance from AI, but [i]every single commit[/i] were manually tested.
Usage of AI in contributions is welcomed if properly harnessed, and review comments addressed.

[size=5][b]Credits[/b][/size]
[list]
[*][url=https://www.nexusmods.com/profile/Demon9ne]Demon9ne[/url]: for suggesting it in the first place 💡.[/*]
[*][url=https://github.com/WopsS]WopsS[/url]: the author of [url=https://github.com/WopsS/RED4ext]RED4ext[/url].[/*]
[*][url=https://github.com/jac3km4]jekky[/url]: the author of [url=https://github.com/jac3km4/red4ext-rs]red4ext-rs[/url] and [url=https://github.com/jac3km4/redscript]Redscript[/url].[/*]
[*][url=https://github.com/psiberx]psiberx[/url]: the author of [url=https://github.com/psiberx/cp2077-codeware]Codeware[/url].[/*]
[/list]
```
