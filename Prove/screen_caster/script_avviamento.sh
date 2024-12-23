#!/bin/bash

# Define the paths to the binaries
FIRST_COMPONENT_BINARY="target/release/first_component"
SECOND_SCRIPT_BINARY="target/release/second_script"
THIRD_SCRIPT_BINARY="target/release/second_script"
MAIN_APP_BINARY="target/release/main_app"

# Function to check if a binary exists and build if not
build_if_not_exists() {
    local binary_path=$1
    local build_command=$2

    if [ -f "$binary_path" ]; then
        echo "Binary for $binary_path already exists. Skipping build."
    else
        echo "Building $binary_path..."
        $build_command
        if [ $? -ne 0 ]; then
            echo "Build failed for $binary_path!"
            exit 1
        fi
    fi
}

# 1. Handle the first component (build only, no run)
echo "Handling first component..."
build_if_not_exists "$FIRST_COMPONENT_BINARY" "cargo build --release --bin first_component"

# 2. Handle the second script (build if needed, then run)
echo "Handling second script..."
build_if_not_exists "$SECOND_SCRIPT_BINARY" "cargo build --release --bin second_script"

echo "Running the second script..."
cargo run --release --bin second_script
if [ $? -ne 0 ]; then
    echo "Execution of second script failed!"
    exit 1
fi

# 3. Handle the main application (build if needed, then run)
echo "Handling main application..."
build_if_not_exists "$MAIN_APP_BINARY" "cargo build --release --bin main_app"

echo "Running the main application..."
cargo run --release --bin main_app
if [ $? -ne 0 ]; then
    echo "Execution of main application failed!"
    exit 1
fi
