#!/bin/sh
# @Author: BlahGeek
# @Date:   2017-07-15
# @Last Modified by:   BlahGeek
# @Last Modified time: 2017-07-15

for terminal in "$TERMINAL" x-terminal-emulator urxvt rxvt termit terminator Eterm aterm uxterm xterm gnome-terminal roxterm xfce4-terminal termite lxterminal mate-terminal terminology st qterminal; do
    if command -v "$terminal" > /dev/null 2>&1; then
        exec "$terminal" "$@"
    fi
done

exit 1
