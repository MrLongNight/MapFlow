#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${1:-0.0.0-dev}"
BUILD_DIR="${2:-$ROOT_DIR/target/release}"
APP_NAME="${APP_NAME:-MapFlow}"
BUNDLE_ID="${MAPFLOW_BUNDLE_ID:-info.mapmapteam.MapFlow}"
APP_DIR="${BUILD_DIR}/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"
PLIST_TEMPLATE="${ROOT_DIR}/resources/macOS/Info.plist"
BINARY_PATH="${BUILD_DIR}/${APP_NAME}"
ICON_PATH="${ROOT_DIR}/resources/app_icons/mapflow.icns"

if [[ ! -f "${BINARY_PATH}" ]]; then
    echo "Expected app binary at ${BINARY_PATH}" >&2
    exit 1
fi

if [[ ! -f "${PLIST_TEMPLATE}" ]]; then
    echo "Missing Info.plist template at ${PLIST_TEMPLATE}" >&2
    exit 1
fi

if [[ ! -f "${ICON_PATH}" ]]; then
    echo "Missing macOS icon at ${ICON_PATH}" >&2
    exit 1
fi

rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"

sed \
    -e "s/__MAPFLOW_VERSION__/${VERSION}/g" \
    -e "s/__MAPFLOW_BUNDLE_ID__/${BUNDLE_ID}/g" \
    "${PLIST_TEMPLATE}" > "${CONTENTS_DIR}/Info.plist"

cp "${BINARY_PATH}" "${MACOS_DIR}/${APP_NAME}"
chmod +x "${MACOS_DIR}/${APP_NAME}"
cp "${ICON_PATH}" "${RESOURCES_DIR}/mapflow.icns"
cp -R "${ROOT_DIR}/assets" "${RESOURCES_DIR}/"
cp -R "${ROOT_DIR}/resources" "${RESOURCES_DIR}/"

echo "${APP_DIR}"
