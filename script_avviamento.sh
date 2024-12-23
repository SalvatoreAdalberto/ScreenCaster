cartelle="annotation_tool overlay_crop screen_caster setup"

for d in $cartelle; do
    echo "Entering folder $d"
    cd "$d" || exit 1

    # Check if target/release exists before building
    if [ ! -d "./target/release" ]; then
        echo "Running cargo build --release"
        cargo build --release
    else
        echo "target/release already exists, skipping build"
    fi

    cd ..
done

echo "Operations completed."

cd setup || exit 1
./target/release/setup

cd ../screen_caster || exit 1
./target/release/screen_caster
