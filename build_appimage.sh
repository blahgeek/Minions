#!/bin/sh
# @Author: BlahGeek
# @Date:   2017-07-09
# @Last Modified by:   BlahGeek
# @Last Modified time: 2017-07-10

cd "$(dirname "$0")"
cargo build --release --verbose

VERSION=$(cat Cargo.toml | grep version | head -n 1 | sed -e 's/version = //; s/"//g')

APP=Minions

rm -rf build/$APP/$APP.AppDir
mkdir -p build/$APP/$APP.AppDir/usr/bin/
mkdir -p build/$APP/$APP.AppDir/usr/lib/
mkdir -p build/$APP/$APP.AppDir/usr/share/
mkdir -p build/$APP/$APP.AppDir/etc/

cp minions.desktop build/$APP/$APP.AppDir
cp minions.png build/$APP/$APP.AppDir
cp target/release/minions build/$APP/$APP.AppDir/usr/bin/minions
cp -r plugins build/$APP/$APP.AppDir/usr/share/minions-plugins
cp config/fonts.conf build/$APP/$APP.AppDir/etc/fonts.conf
cp -r fonts build/$APP/$APP.AppDir/fonts

cat > build/$APP/$APP.AppDir/usr/bin/minions.wrapper << EOF
#!/bin/sh
cd "\$(dirname "\$0")"
cd ../../

echo "Building font cache..."
mkdir -p ~/.minions/fontconfig/
export FONTCONFIG_FILE="\$(pwd)/etc/fonts.conf"
fc-cache

unset XDG_DATA_DIRS
minions "\$@"
EOF

chmod a+x build/$APP/$APP.AppDir/usr/bin/minions.wrapper
sed -i "s/Exec=minions/Exec=minions.wrapper/" build/$APP/$APP.AppDir/minions.desktop

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
