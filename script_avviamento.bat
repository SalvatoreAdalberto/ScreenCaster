@echo off

set "cartelle=annotation_tool overlay_crop screen_caster setup"

for %%d in (%cartelle%) do (
    cd "%%d"

    if not exist "target\release\%%d.exe" (
        echo Building %%d...
        cargo build --release
    ) else (
        if /I "%%d"=="screen_caster" (
            echo Building %%d...
            cargo build --release
        ) else (
            echo target\release\%%d.exe already exists
        )
    )
    cd ..
)

cd "setup"
"target\release\setup.exe"

cd "..\screen_caster"
"target\release\screen_caster.exe"
