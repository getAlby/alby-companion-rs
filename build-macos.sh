#!/bin/sh
set -x

cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

lipo target/aarch64-apple-darwin/release/alby target/x86_64-apple-darwin/release/alby -create -output alby


FILE_NAME="alby-macos.zip"
zip -9r $FILE_NAME alby
CHECKSUM=$(sha256sum "${FILE_NAME}" | cut -d ' ' -f 1)
echo "$CHECKSUM $FILE_NAME" > "$FILE_NAME.sha256sum"