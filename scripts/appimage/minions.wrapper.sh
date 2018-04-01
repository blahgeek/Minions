#!/bin/sh
# @Author: BlahGeek
# @Date:   2017-08-13
# @Last Modified by:   BlahGeek
# @Last Modified time: 2018-04-01

cd "$(dirname "$0")"
cd ../../

check_update() {
    RELEASE_URL="https://api.github.com/repos/blahgeek/Minions/releases/latest"
    LATEST_VERSION=$(curl -q "$RELEASE_URL" | \
                     grep -o "Minions-.\+-[0-9a-f]\{6\}" | \
                     head -n 1)
    if [ "$LATEST_VERSION" != "__CURRENT_VERSION__" ]; then
        MSG="New version $LATEST_VERSION found. \n"
        MSG="${MSG}Current version is __CURRENT_VERSION__. \n"
        MSG="${MSG}Visit <a href=\"https://github.com/blahgeek/Minions/releases\">Release page</a> to download.\n"
        echo "$MSG"
        zenity --info --text "$MSG"
    fi
}

unset XDG_DATA_DIRS
check_update &
minions "$@"
