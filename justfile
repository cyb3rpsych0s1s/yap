set shell := ["powershell.exe", "-NoLogo", "-Command"]
set dotenv-load := true

# default to steam default game dir

DEFAULT_GAME_DIR := join("C:\\", "Program Files (x86)", "Steam", "steamapps", "common", "Cyberpunk 2077")
game_dir := env("GAME_DIR", DEFAULT_GAME_DIR)
here := justfile_directory()

# game folders

redscript_dir := join("r6", "scripts")
red4ext_dir := join("red4ext", "plugins")

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

@setup package to=game_dir:
    just create '{{ join(to, red4ext_dir, lowercase(package)) }}'
    just create '{{ join(to, redscript_dir, package) }}'

@build package to=game_dir features='default':
    cargo build -p '{{ lowercase(package) }}' --release {{ if features == "no-default" { "--no-default-features" } else { "" } }}
    just copy '{{ join(here, "target", "release", lowercase(package) + ".dll") }}' '{{ join(to, red4ext_dir, lowercase(package), lowercase(package) + ".dll") }}'
@build-dbg package to=game_dir:
    cargo build -p '{{ lowercase(package) }}'
    just copy '{{ join(here, "target", "debug", lowercase(package) + ".dll") }}' '{{ join(to, red4ext_dir, lowercase(package), lowercase(package) + ".dll") }}'
    just copy '{{ join(here, "target", "debug", lowercase(package) + ".pdb") }}' '{{ join(to, red4ext_dir, lowercase(package), lowercase(package) + ".pdb") }}'

@reds package to=game_dir:
    just copy-recurse '{{ join(here, "crates", lowercase(package), "reds", "*") }}' '{{ join(to, redscript_dir, package) }}'

@reload package to=game_dir: (reds package to)

@dev package features='default': (setup package) (build package game_dir features) (reload package)
@dbg package: (setup package) (build-dbg package) (reload package)

@uninstall package from=game_dir:
    just delete '{{ join(from, red4ext_dir, lowercase(package)) }}'
    just delete '{{ join(from, redscript_dir, package) }}'

[private]
no-debug path:
    @Get-ChildItem -Path '{{ path }}' -Recurse -Filter 'Debug*.reds' | Remove-Item -Force

@ci package folder features='default':
    just setup '{{ package }}' '{{ join(here, folder, "bin") }}'
    just create '{{ join(here, folder, "pdb", red4ext_dir, lowercase(package)) }}'
    just build '{{ package }}' '{{ join(here, folder, "bin") }}' '{{ features }}'
    just copy '{{ join(here, "target", "release", lowercase(package) + ".pdb") }}' '{{ join(here, folder, "pdb", red4ext_dir, lowercase(package), lowercase(package) + ".pdb") }}'
    just reload '{{ package }}' '{{ join(here, folder, "bin") }}'
    just no-debug '{{ join(here, folder, "bin") }}'

@check:
    cargo check -p yap --release
    cargo check -p yap --release --no-default-features

@qa:
    cargo clippy -p yap -- -D warnings
    cargo clippy -p yap --no-default-features -- -D warnings
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
