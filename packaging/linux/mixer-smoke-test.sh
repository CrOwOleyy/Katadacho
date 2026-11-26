#!/usr/bin/env bash
# Teste le mixeur Linux contre un vrai serveur audio : démarre PipeWire dans le conteneur, fait jouer
# un son à `paplay` (sur une sortie virtuelle), puis lance le test qui règle le volume de cette
# application avec pactl. Le dépôt est monté en lecture seule sur /repo.
set -euo pipefail

dnf install -y -q pipewire wireplumber pipewire-pulseaudio pulseaudio-utils > /dev/null

mkdir -p /build
tar -C /repo --exclude=./node_modules --exclude=./dist --exclude=./src-tauri/target -cf - . | tar -C /build -xf -
# Le test n'a pas besoin du frontend, mais Tauri exige que son dossier existe.
mkdir -p /build/dist && touch /build/dist/index.html

export XDG_RUNTIME_DIR=/tmp/xdg-runtime
mkdir -p "$XDG_RUNTIME_DIR" && chmod 700 "$XDG_RUNTIME_DIR"

run_with_audio() {
    pipewire > /tmp/pipewire.log 2>&1 &
    sleep 1
    wireplumber > /tmp/wireplumber.log 2>&1 &
    pipewire-pulse > /tmp/pipewire-pulse.log 2>&1 &
    for _ in $(seq 30); do
        pactl info > /dev/null 2>&1 && break
        sleep 0.3
    done
    # Pas de carte son dans un conteneur : une sortie virtuelle reçoit le son.
    pactl load-module module-null-sink sink_name=virtuelle > /dev/null
    pactl set-default-sink virtuelle

    # Une « application » qui joue du silence en continu.
    paplay --raw --rate=44100 --channels=2 --format=s16le /dev/zero &
    sleep 1
    echo "--- flux vus par pactl"
    pactl list short sink-inputs

    cd /build/src-tauri
    cargo test --locked --lib controls_real_app_streams -- --ignored --nocapture
}
export -f run_with_audio
dbus-run-session -- bash -c run_with_audio
