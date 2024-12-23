cartelle="annotation_tool overlay_crop screen_caster"

for d in $cartelle; do
    echo "Entering folder $d"
    cd "$d" || exit 1

    echo "Running cargo clean"
    cargo clean
    echo "Running cargo build --release"
    cargo build --release

    cd ..
done

echo "Operations completed."

cd screen_caster || exit 1
./target/release/screen_caster