#!/bin/sh
# @Author: BlahGeek
# @Date:   2017-07-09
# @Last Modified by:   BlahGeek
# @Last Modified time: 2017-07-09

# some functions is copied from https://github.com/probonopd/AppImages/raw/master/functions.sh

# Detect system architecture to know which binaries of AppImage tools
# should be downloaded and used.
case "$(uname -i)" in
  x86_64|amd64)
#    echo "x86-64 system architecture"
    SYSTEM_ARCH="x86_64";;
  i?86)
#    echo "x86 system architecture"
    SYSTEM_ARCH="i686";;
#  arm*)
#    echo "ARM system architecture"
#    SYSTEM_ARCH="";;
  unknown)
#         uname -i not answer on debian, then:
    case "$(uname -m)" in
      x86_64|amd64)
#        echo "x86-64 system architecture"
        SYSTEM_ARCH="x86_64";;
      i?86)
#        echo "x86 system architecture"
        SYSTEM_ARCH="i686";;
    esac ;;
  *)
    echo "Unsupported system architecture"
    exit 1;;
esac

# TODO
ARCH=${SYSTEM_ARCH}

cd "$(dirname "$0")"
cargo build --release --verbose

VERSION=$(cat Cargo.toml | grep version | head -n 1 | sed -e 's/version = //; s/"//g')

APP=Minions

rm -rf build/$APP/$APP.AppDir
mkdir -p build/$APP/$APP.AppDir/usr/bin/
mkdir -p build/$APP/$APP.AppDir/usr/share/

cp minions.desktop build/$APP/$APP.AppDir
cp minions.png build/$APP/$APP.AppDir
cp target/release/minions build/$APP/$APP.AppDir/usr/bin/minions
cp -r plugins build/$APP/$APP.AppDir/usr/share/minions-plugins

cat > build/$APP/$APP.AppDir/usr/bin/minions.wrapper << EOF
#!/bin/sh
unset XDG_DATA_DIRS
minions "\$@"
EOF

chmod a+x build/$APP/$APP.AppDir/usr/bin/minions.wrapper
sed -i "s/Exec=minions/Exec=minions.wrapper/" build/$APP/$APP.AppDir/minions.desktop

# bundle xbindkeys
cp $(which xbindkeys) build/$APP/$APP.AppDir/usr/bin/

cd build/$APP/$APP.AppDir

wget -c https://github.com/AppImage/AppImageKit/releases/download/continuous/AppRun-${ARCH} -O AppRun
chmod a+x AppRun

# do not copy libraries
# expect gtk is installed on target system

cd ..

# Download AppImageAssistant
wget -c "https://github.com/AppImage/AppImageKit/releases/download/6/AppImageAssistant_6-${SYSTEM_ARCH}.AppImage" -O AppImageAssistant
chmod a+x ./AppImageAssistant

mkdir -p ../out || true
rm ../out/$APP"-"$VERSION"-"${ARCH}".AppImage" 2>/dev/null || true
./AppImageAssistant ./$APP.AppDir/ ../out/$APP"-"$VERSION"-"${ARCH}".AppImage"

