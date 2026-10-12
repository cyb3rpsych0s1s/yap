set shell := ["powershell.exe", "-NoLogo", "-Command"]
set dotenv-load := true

# default to steam default game dir

DEFAULT_GAME_DIR := join("C:\\", "Program Files (x86)", "Steam", "steamapps", "common", "Cyberpunk 2077")
game_dir := env("GAME_DIR", DEFAULT_GAME_DIR)
here := justfile_directory()

# game folders

redscript_dir := join("r6", "scripts")
red4ext_dir := join("red4ext", "plugins")

package := 'Yap'

# cargo features of each flavor
# NO_HOOK keeps `plugin`: without it the DLL exports nothing
# crate is for red4ext-rs developers only: never deployed

hook_features := ''
no_hook_features := '--no-default-features --features plugin'
crate_features := '--no-default-features'

# shortcuts

[private]
create path:
    @if (!(Test-Path '{{ path }}')) { [void](New-Item '{{ path }}' -ItemType Directory); Write-Host "Created folder at {{ path }}"; }

[private]
copy from to:
    @Copy-Item -Force '{{ from }}' '{{ to }}'
    @Write-Host "Copied {{ from }} -> {{ to }}"

[private]
copy-recurse from to:
    @Copy-Item -Force -Recurse '{{ from }}' '{{ to }}'
    @Write-Host "Copied {{ from }} -> {{ to }}"

[private]
delete path:
    @if (Test-Path '{{ path }}') { [void](Remove-Item -Force -Recurse '{{ path }}'); Write-Host "Deleted {{ path }}"; }

# commands

@setup to=game_dir:
    just create '{{ join(to, red4ext_dir, lowercase(package)) }}'
    just create '{{ join(to, redscript_dir, package) }}'

@build to=game_dir features='':
    cargo build -p '{{ lowercase(package) }}' --release {{ features }}
    just copy '{{ join(here, "target", "release", lowercase(package) + ".dll") }}' '{{ join(to, red4ext_dir, lowercase(package), lowercase(package) + ".dll") }}'
@build-dbg to=game_dir features='':
    cargo build -p '{{ lowercase(package) }}' {{ features }}
    just copy '{{ join(here, "target", "debug", lowercase(package) + ".dll") }}' '{{ join(to, red4ext_dir, lowercase(package), lowercase(package) + ".dll") }}'
    just copy '{{ join(here, "target", "debug", lowercase(package) + ".pdb") }}' '{{ join(to, red4ext_dir, lowercase(package), lowercase(package) + ".pdb") }}'

@reds to=game_dir:
    just copy-recurse '{{ join(here, "crates", lowercase(package), "reds", "*") }}' '{{ join(to, redscript_dir, package) }}'

@reload to=game_dir: (reds to)

@no-hook to=game_dir: (setup to) (build to no_hook_features) (reload to)
@hook to=game_dir: (setup to) (build to hook_features) (reload to)
@crate:
    cargo build -p '{{ lowercase(package) }}' --release {{ crate_features }}

@dbg to=game_dir: (setup to) (build-dbg to) (reload to)

@uninstall package from=game_dir:
    just delete '{{ join(from, red4ext_dir, lowercase(package)) }}'
    just delete '{{ join(from, redscript_dir, package) }}'

# strips the debug harness from releases, including `DebugNatives.reds`'s
# root-namespace re-declarations of game natives, which must never ship
[private]
no-debug path:
    @Get-ChildItem -Path '{{ path }}' -Recurse -Filter 'Debug*.reds' | Remove-Item -Force

# flavor: hook | no-hook
@ci folder flavor='hook':
    {{ if flavor =~ '^(hook|no-hook)$' { "" } else { error("unknown flavor '" + flavor + "', expected hook or no-hook") } }}
    just setup '{{ join(here, folder, "bin") }}'
    just create '{{ join(here, folder, "pdb", red4ext_dir, lowercase(package)) }}'
    just build '{{ join(here, folder, "bin") }}' '{{ if flavor == "no-hook" { no_hook_features } else { hook_features } }}'
    just copy '{{ join(here, "target", "release", lowercase(package) + ".pdb") }}' '{{ join(here, folder, "pdb", red4ext_dir, lowercase(package), lowercase(package) + ".pdb") }}'
    just reload '{{ join(here, folder, "bin") }}'
    just no-debug '{{ join(here, folder, "bin") }}'

@check:
    cargo check -p '{{ lowercase(package) }}' --release {{ hook_features }}
    cargo check -p '{{ lowercase(package) }}' --release {{ no_hook_features }}
    cargo check -p '{{ lowercase(package) }}' --release {{ crate_features }}

@qa:
    cargo clippy -p '{{ lowercase(package) }}' {{ hook_features }} -- -D warnings
    cargo clippy -p '{{ lowercase(package) }}' {{ no_hook_features }} -- -D warnings
    cargo clippy -p '{{ lowercase(package) }}' {{ crate_features }} -- -D warnings
    cargo fmt --all --check

@log package TO=game_dir:
    $folder = '{{ join(TO, "red4ext", "logs") }}'; \
    $files = Get-ChildItem -Path $folder -Filter "{{ lowercase(package) }}-*.log"; \
    $logs = @(); \
    foreach ($file in $files) { \
      if ($file.Name -match "{{ lowercase(package) }}-(\d{4})-(\d{2})-(\d{2})-(\d{2})-(\d{2})-(\d{2})") { \
        $dateString = "$($matches[1])-$($matches[2])-$($matches[3]) $($matches[4]):$($matches[5]):$($matches[6])"; \
        $date = [datetime]::ParseExact($dateString, "yyyy-MM-dd HH:mm:ss", $null); \
        $logs += [PSCustomObject]@{ \
          File = $file.FullName; \
          Date = $date; \
        } \
      } \
    } \
    $latest = $logs | Sort-Object Date -Descending | Select-Object -First 1; \
    if ($latest) { \
      Write-Output "The most recent log file is: $($latest.File)"; \
      Get-Content -Path $latest.File -Wait -Encoding UTF8; \
    } else { \
      Write-Output "No log files found."; \
    }

@link THERE HERE='':
    $there = '{{ THERE }}'; \
    $here  = "{{ if HERE == "" { `Split-Path $there -Leaf` } else { HERE } }}"; \
    New-Item -Path $here -ItemType SymbolicLink -Target '{{ THERE }}'

@lint:
    cargo fix --allow-dirty
    cargo clippy --fix --allow-dirty
    cargo fmt --all
