#!/usr/bin/env bash
# Compile Katadacho dans un conteneur Linux. Usage : build-in-container.sh <check|bundle> [bundles]
# Le dépôt est monté en lecture seule sur /repo ; on le copie sans les artefacts Windows.
set -euo pipefail

mode="${1:-check}"
bundles="${2:-}"

mkdir -p /build
tar -C /repo \
    --exclude=./node_modules --exclude=./dist --exclude=./src-tauri/target --exclude=./.git \
    -cf - . | tar -C /build -xf -
cd /build

pnpm install --frozen-lockfile

case "$mode" in
    check)
        pnpm build
        cd src-tauri && cargo check --locked
        ;;
    bundle)
        pnpm tauri build --bundles "$bundles"
        mkdir -p /out
        find src-tauri/target/release/bundle -type f \( -name '*.rpm' -o -name '*.deb' -o -name '*.AppImage' -o -name '*.pkg.tar.zst' \) -exec cp {} /out/ \;
        ls -la /out
        ;;
    *)
        echo "mode inconnu : $mode" >&2
        exit 1
        ;;
esac
