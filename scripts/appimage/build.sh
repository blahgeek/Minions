#!/bin/sh
# @Author: BlahGeek
# @Date:   2017-07-09
# @Last Modified by:   BlahGeek
# @Last Modified time: 2017-08-13

cd "$(dirname "$0")"
cd ../../

cargo build --release --verbose

VERSION=$(cat Cargo.toml | grep version | head -n 1 | sed -e 's/version = //; s/"//g')
GITHASH=$(echo "$TRAVIS_COMMIT" | cut -c1-6)
VERSION="$VERSION-$GITHASH"

APP=Minions

rm -rf build/$APP/$APP.AppDir
mkdir -p build/$APP/$APP.AppDir/usr/bin/
mkdir -p build/$APP/$APP.AppDir/usr/lib/
mkdir -p build/$APP/$APP.AppDir/usr/share/
mkdir -p build/$APP/$APP.AppDir/etc/

cp scripts/appimage/minions.desktop build/$APP/$APP.AppDir
cp scripts/appimage/minions.png build/$APP/$APP.AppDir
cp target/release/minions build/$APP/$APP.AppDir/usr/bin/minions
cp -r plugins build/$APP/$APP.AppDir/usr/share/minions-plugins
cp config/fonts.conf build/$APP/$APP.AppDir/etc/fonts.conf
cp -r fonts build/$APP/$APP.AppDir/fonts

cp $(which zenity) build/$APP/$APP.AppDir/usr/bin/

cat scripts/appimage/minions.wrapper.sh | \
    sed "s/__CURRENT_VERSION__/Minions-$VERSION/g" > \
        build/$APP/$APP.AppDir/usr/bin/minions.wrapper.sh
chmod a+x build/$APP/$APP.AppDir/usr/bin/minions.wrapper.sh

cd build/$APP
wget -q https://github.com/probonopd/AppImages/raw/master/functions.sh -O ./functions.sh
. ./functions.sh

cd $APP.AppDir

copy_deps
delete_blacklisted
find ./usr/lib/ -name 'libharfbuzz.*' | xargs -i rm {}

get_apprun

# do not copy libraries
# expect gtk is installed on target system

cd ..

generate_type2_appimage
