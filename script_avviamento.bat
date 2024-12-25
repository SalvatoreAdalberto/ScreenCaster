@echo off

set "cartelle=annotation_tool overlay_crop screen_caster setup"

for %%d in (%cartelle%) do (
    cd "%%d"

    if exist "target\release\%%d.exe" (
        echo target/release/%%d.exe already exists.
    ) else (
        echo Building %%d...
        cargo build --release
    )
    cd ..
)

cd "setup"
"target\release\setup.exe"

cd "..\screen_caster"
"target\release\screen_caster.exe"
