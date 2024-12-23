@echo off

set "cartelle=annotation_tool overlay_crop screen_caster setup"

for %%d in (%cartelle%) do (
    echo Entrando nella cartella %%d
    cd %%d

    if exist target\release (
        echo La cartella target/release esiste.
    ) else (
        cargo build --release
    )
    cd ..
)

cd setup
target\release\setup.exe

cd ..\screen_caster
target\release\screen_caster.exe
