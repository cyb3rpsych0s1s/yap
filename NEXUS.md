# Nexus page

Full description for the Nexus Mods page, in BBCode.
Paste the block below into the description editor in BBCode mode (the `[/]` toggle).

```bbcode
[center][size=6][b]yap[/b][/size]
[i]leveled logging for redscript mods, straight into RED4ext's logs[/i][/center]

[line]

[size=5][b]What is it?[/b][/size]

[b]yap[/b] is a small [url=https://www.nexusmods.com/cyberpunk2077/mods/2380]RED4ext[/url] plugin for modders.

It gives any redscript mod six log functions, writes their lines into RED4ext's log files,
and tags each line with the script function that emitted it.

[b]Players[/b]: you only need yap if a mod lists it as a requirement. It doesn't change anything in-game.

[size=5][b]Features[/b][/size]

yap comes in [b]2 flavors[/b]: pick [b]one[/b] of the two main files.

[size=4][b]Without hooks[/b] (the [i]NO_HOOK[/i] file)[/size]
[list]
[*][font=Courier New]Life[/font], [font=Courier New]Trace[/font], [font=Courier New]Debug[/font], [font=Courier New]Info[/font], [font=Courier New]Warn[/font] and [font=Courier New]Error[/font], callable from any script, prefixed with the calling [font=Courier New]Class::Func[/font].[/*]
[*]Follows RED4ext's [font=Courier New][logging] level[/font] from [font=Courier New]red4ext/config.ini[/font].[/*]
[/list]

[size=4][b]With hooks[/b] (recommended for modders)[/size]
Everything above, plus:
[list]
[*]Captures the game's own [font=Courier New]FTLog[/font] calls, which are otherwise silent in retail builds.[/*]
[*]Mirrors every other RED4ext plugin's log lines into yap's log as [font=Courier New][<plugin>] …[/font], so one file aggregates all the mods from [i]anywhere[/i].[/*]
[/list]

The hooked flavor relies on game and RED4ext internals. If it ever misbehaves right after a game patch, switch to [i]NO_HOOK[/i] until yap is updated: your mods' log calls keep working.

[size=5][b]Requirements[/b][/size]
[list]
[*][url=https://www.nexusmods.com/cyberpunk2077/mods/2380]RED4ext[/url][/*]
[*][url=https://www.nexusmods.com/cyberpunk2077/mods/1511]redscript[/url][/*]
[/list]

[size=5][b]Installation[/b][/size]
Install with Vortex or MO2, or extract the archive into your game folder. You should end up with:
[code]Cyberpunk 2077/red4ext/plugins/yap/yap.dll
Cyberpunk 2077/r6/scripts/Yap/Log.reds[/code]

To uninstall, delete those two folders.

[size=5][b]For modders[/b][/size]
[code]module MyMod
import Yap.Info

public class MySystem extends ScriptableSystem {
    private func OnAttach() {
        Info("hello from MyMod");
    }
}[/code]

Lines land in [font=Courier New]Cyberpunk 2077/red4ext/logs/yap-*.log[/font], for example:
[code][2026-10-09 18:12:56.460] [info    ] [ 48396] [yap] 🔷 [ reds ] Cat.CatServiceDebug::OnLoad calling Yap from Cat[/code]

Set [font=Courier New]level = "debug"[/font] (or [font=Courier New]"trace"[/font]) in [font=Courier New]red4ext/config.ini[/font] to also see [font=Courier New]Debug[/font] / [font=Courier New]Trace[/font] lines.

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

[size=5][b]Acknowledgement[/b][/size]

The hooks were implemented with partial assistance from AI, but [i]every single commit[/i] were manually tested.

Usage of AI in contributions is tolerated if properly harnessed, and review comments addressed.

[i]Using AI does not mean slop is welcomed.[/i]

[size=5][b]Credits[/b][/size]
[list]
[*][url=https://github.com/jac3km4]jekky[/url]: author of [url=https://github.com/jac3km4/red4ext-rs]red4ext-rs[/url] and [url=https://github.com/jac3km4/redscript]redscript[/url].[/*]
[*][url=https://github.com/WopsS]WopsS[/url]: author of [url=https://github.com/WopsS/RED4ext]RED4ext[/url].[/*]
[/list]
```
