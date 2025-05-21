#!/usr/bin/env bash

set -e

# https://doc.rust-lang.org/rustc/platform-support.html
TARGET_LINUX_X86_64="x86_64-unknown-linux-gnu"
TARGET_WINDOWS_X86_64="x86_64-pc-windows-gnu"
TARGET_MACOS_X86_64="x86_64-apple-darwin"
TARGET_MACOS_AARCH64="aarch64-apple-darwin"

BIN_NAME="csvpeek-rs"
RELEASE_DIR="releases"

if ! command -v cross &> /dev/null
then
    echo "'cross' seems not be installed"
    echo "Please install with: cargo install cross"
    exit 1
fi

echo ""
echo "Building for Linux (x86_64)..."
cross build --target ${TARGET_LINUX_X86_64} --release --locked --verbose

echo ""
echo "Building for Windows (x86_64)..."
cross build --target ${TARGET_WINDOWS_X86_64} --release --locked --verbose

echo ""
echo "Skip building for macOS (Intel x86_64)..."
# cross build --target ${TARGET_MACOS_X86_64} --release --locked --verbose

echo ""
echo "Skip building for macOS (Apple Silicon aarch64)..."
# cross build --target ${TARGET_MACOS_AARCH64} --release --locked --verbose

mkdir -p "${RELEASE_DIR}/linux-x86_64"
mkdir -p "${RELEASE_DIR}/macos-x86_64"
mkdir -p "${RELEASE_DIR}/macos-aarch64"

WIN_PLATFORM_BASE_DIR="${RELEASE_DIR}/windows-x86_64"
WIN_UNPACKED_DIR="${WIN_PLATFORM_BASE_DIR}/unpacked"
WIN_PACKED_DIR="${WIN_PLATFORM_BASE_DIR}/packed"
mkdir -p "${WIN_UNPACKED_DIR}"
mkdir -p "${WIN_PACKED_DIR}"


echo ""
echo "Copying binaries to '${RELEASE_DIR}' subdirectories..."

cp "target/${TARGET_LINUX_X86_64}/release/${BIN_NAME}" "${RELEASE_DIR}/linux-x86_64/"
# cp "target/${TARGET_MACOS_X86_64}/release/${BIN_NAME}" "${RELEASE_DIR}/macos-x86_64/"
# cp "target/${TARGET_MACOS_AARCH64}/release/${BIN_NAME}" "${RELEASE_DIR}/macos-aarch64/"

WINDOWS_SOURCE_EXE="target/${TARGET_WINDOWS_X86_64}/release/${BIN_NAME}.exe"
cp "${WINDOWS_SOURCE_EXE}" "${WIN_UNPACKED_DIR}/${BIN_NAME}.exe"
cp "${WINDOWS_SOURCE_EXE}" "${WIN_PACKED_DIR}/${BIN_NAME}.exe"
echo "Binaries copied."


if command -v upx &> /dev/null
then
    echo ""
    echo "Attempting to compress binaries with UPX (--best --lzma)..."
    
    echo "Compressing Linux binary..."
    upx --best --lzma "${RELEASE_DIR}/linux-x86_64/${BIN_NAME}" || echo "UPX compression failed for Linux binary (continuing)..."
    
    echo "Compressing Windows packed binary in ${WIN_PACKED_DIR}..."
    upx --best --lzma "${WIN_PACKED_DIR}/${BIN_NAME}.exe" || echo "UPX compression failed for Windows binary in ${WIN_PACKED_DIR} (continuing)..."
    
  #  echo "Compressing macOS Intel binary..."
  #  upx --best --lzma "${RELEASE_DIR}/macos-x86_64/${BIN_NAME}" || echo "UPX compression failed for macOS Intel binary (continuing)..."
    
  #  echo "Compressing macOS Apple Silicon binary..."
  #  upx --best --lzma "${RELEASE_DIR}/macos-aarch64/${BIN_NAME}" || echo "UPX compression failed for macOS Apple Silicon binary (continuing)..."
    
    echo "UPX compression attempt finished."
else
    echo ""
    echo "UPX command not found, skipping compression. All binaries are uncompressed."
fi

echo ""
echo "-----------------------------------------------------"
echo "Build and packaging process finished!"
echo "Original binaries are in target/<target_triple>/release/"
echo "Final binaries are in the '${RELEASE_DIR}' subdirectories:"
echo "- Linux x86_64:      ${RELEASE_DIR}/linux-x86_64/${BIN_NAME}"
echo "- Windows x86_64 (unpacked): ${WIN_UNPACKED_DIR}/${BIN_NAME}.exe"
echo "- Windows x86_64 (packed):   ${WIN_PACKED_DIR}/${BIN_NAME}.exe"
# echo "- macOS Intel:       ${RELEASE_DIR}/macos-x86_64/${BIN_NAME}"
# echo "- macOS Apple Silicon: ${RELEASE_DIR}/macos-aarch64/${BIN_NAME}"
echo "-----------------------------------------------------"
