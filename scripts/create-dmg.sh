#!/usr/bin/env bash
# 使用 macOS 内置 hdiutil 打包 DMG，绕过 create-dmg 不支持 macOS 26+ 的问题
set -e

APP_NAME="MyloAir"
VERSION="1.0.0"
APP_PATH="src-tauri/target/release/bundle/macos/${APP_NAME}.app"
DMG_DIR="src-tauri/target/release/bundle/dmg"
DMG_NAME="${APP_NAME}_${VERSION}_aarch64.dmg"
TMP_DIR=$(mktemp -d)

if [ ! -d "$APP_PATH" ]; then
  echo "Error: ${APP_PATH} not found. Run 'npm run tauri:build' first."
  exit 1
fi

echo "Creating DMG: ${DMG_NAME}"

# 拷贝 .app 到临时目录，添加 Applications 软链接
cp -r "$APP_PATH" "$TMP_DIR/"
ln -s /Applications "$TMP_DIR/Applications"

# 用 hdiutil 创建 DMG
mkdir -p "$DMG_DIR"
hdiutil create \
  -volname "$APP_NAME" \
  -srcfolder "$TMP_DIR" \
  -ov \
  -format UDZO \
  "${DMG_DIR}/${DMG_NAME}"

rm -rf "$TMP_DIR"
echo "Done: ${DMG_DIR}/${DMG_NAME}"
