#!/bin/bash
set -e

# Build tools
cargo build --bin imgzen
cargo build --bin gen_test_image

# Setup test dir
rm -rf test_run
mkdir test_run
cd test_run

# Generate inputs
../target/debug/gen_test_image
echo '<img src="test_image.png">' > index.html

# Run ImgZen
# Inputs normally passed via env vars in Action, but CLI supports flags.
# We test CLI flags mapping to env vars implicitly via clap or just flags.
# clap 'env' feature works if var is set.
export INPUT_PATHS="."
export INPUT_IGNORE=""
export INPUT_GENERATE_FORMATS="avif,webp"
export INPUT_RESPONSIVE_WIDTHS="50"
export INPUT_INJECT_LAZY_LOADING="true"

# Run binary
../target/debug/imgzen

# Verify outputs
if [ ! -f "test_image.avif" ]; then
    echo "FAILED: test_image.avif missing"
    exit 1
fi
if [ ! -f "test_image.webp" ]; then
    echo "FAILED: test_image.webp missing"
    exit 1
fi
if [ ! -f "test_image-50.png" ]; then
    echo "FAILED: test_image-50.png missing (responsive)"
    exit 1
fi
# Check HTML injection
if ! grep -q 'loading="lazy"' index.html; then
    echo "FAILED: loading=lazy not injected"
    exit 1
fi

echo "E2E Test Passed!"
