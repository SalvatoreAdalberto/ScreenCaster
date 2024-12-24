cartelle="annotation_tool overlay_crop screen_caster setup"

for d in $cartelle; do
    cd "$d" || exit 1

    # Check if target/release exists before building
    if [ ! -f "./target/release/$d" ]; then
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
