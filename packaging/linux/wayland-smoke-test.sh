#!/usr/bin/env bash
# Lance Katadacho dans Sway sans écran (vrai compositeur Wayland en tuiles, avec layer-shell) et
# vérifie que l'Island n'est pas rangée comme une fenêtre ordinaire. Écrit une capture d'écran.
# Usage : wayland-smoke-test.sh <binaire> <capture.png>
set -euo pipefail

binary="$1"
shot="$2"

export XDG_RUNTIME_DIR=/tmp/xdg-runtime
mkdir -p "$XDG_RUNTIME_DIR" && chmod 700 "$XDG_RUNTIME_DIR"
# Rendu logiciel, sans écran ni périphérique d'entrée.
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export LIBGL_ALWAYS_SOFTWARE=1 WEBKIT_DISABLE_COMPOSITING_MODE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1
# Le bac à sable de WebKit ne démarre pas en root dans un conteneur : test uniquement.
export WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1

cat > /tmp/sway-test.conf <<'CONF'
output HEADLESS-1 resolution 1920x1080 bg #2f5d7c solid_color
# Cadre jaune épais : il se verrait tout de suite si l'Island était rangée comme une fenêtre.
default_border pixel 8
client.focused #ffcc00 #ffcc00 #000000 #ffcc00 #ffcc00
CONF

run_session() {
    sway -c /tmp/sway-test.conf > /tmp/sway.log 2>&1 &
    for _ in $(seq 50); do
        socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-*' ! -name '*.lock' | head -n 1)
        [ -n "$socket" ] && break
        sleep 0.2
    done
    [ -n "${socket:-}" ] || { echo "Sway n'a pas démarré"; cat /tmp/sway.log; exit 1; }
    export WAYLAND_DISPLAY
    WAYLAND_DISPLAY=$(basename "$socket")
    export SWAYSOCK
    SWAYSOCK=$(find "$XDG_RUNTIME_DIR" /tmp -maxdepth 1 -name 'sway-ipc.*' 2>/dev/null | head -n 1)

    "$binary" > /tmp/katadacho.log 2>&1 &
    app_pid=$!
    sleep 15
    grim "$shot"

    # Une app plantée serait absente de l'arbre elle aussi : on vérifie d'abord qu'elle tourne.
    if ! kill -0 "$app_pid" 2>/dev/null; then
        echo "ÉCHEC : Katadacho s'est arrêté"; tail -n 30 /tmp/katadacho.log; exit 1
    fi
    # Une surface layer-shell n'apparaît pas dans l'arbre des fenêtres de Sway.
    if swaymsg -t get_tree | grep -qi '"app_id": "[^"]*katadacho'; then
        echo "ÉCHEC : l'Island est rangée comme une fenêtre ordinaire"
        result=1
    else
        echo "OK : l'Island n'est pas dans la mise en page (surface layer-shell)"
        result=0
    fi
    echo "--- journal de Katadacho"
    tail -n 20 /tmp/katadacho.log
    exit "$result"
}
export -f run_session
export binary shot
dbus-run-session -- bash -c run_session
