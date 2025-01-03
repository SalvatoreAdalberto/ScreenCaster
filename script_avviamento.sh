cartelle="annotation_tool overlay_crop screen_caster setup"
if $OSTYPE == "linux-gnu"; then
    sudo apt-get install libgtk-3-dev
fi
for d in $cartelle; do
    cd "$d" || exit 1


    if $1; then
        echo "Cleaning $d"
        cargo clean
    fi

    # Check if target/release exists before building
    if [[ ! -f "./target/release/$d" || $d == "screen_caster" ]]; then
        echo "Building $d"
        cargo build --release
    else
        echo "target/release/$d already exists"
    fi

    cd ..
done

cd setup || exit 1
./target/release/setup

cd ../screen_caster || exit 1
./target/release/screen_caster
