#!/bin/sh

set -ex

export APPIMAGE_EXTRACT_AND_RUN=1
export ARCH=${ARCH:-"$(uname -m)"}

LIB4BN="https://raw.githubusercontent.com/VHSgunzo/sharun/refs/heads/main/lib4bin"
APPIMAGETOOL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-$ARCH.AppImage"
UPINFO="gh-releases-zsync|pkgforge-dev|AppImageUpdate|latest|*$ARCH.AppImage.zsync"

# use RAM disk if possible
TEMP_BASE=/tmp

BUILD_DIR="$(mktemp -d -p "$TEMP_BASE" AppImageUpdate-build-XXXXXX)"

cleanup () {
    if [ -d "$BUILD_DIR" ]; then
        rm -rf "$BUILD_DIR"
    fi
}

trap cleanup EXIT

# store repo root as variable
REPO_ROOT="$(readlink -f "$(dirname "$(dirname "$0")")")"
OLD_CWD="$(readlink -f .)"

cd "$BUILD_DIR"
cmake "$REPO_ROOT" \
    -DBUILD_QT_UI=OFF \
    -DCMAKE_INSTALL_PREFIX=/usr \
    -DCMAKE_BUILD_TYPE=MinSizeRel

# next step is to build the binaries
make -j"$(nproc)"

# set up the AppDirs initially
for appdir in appimageupdatetool.AppDir validate.AppDir; do
	make install DESTDIR="$appdir"
	mkdir -p "$appdir"/resources
	cp -v "$REPO_ROOT"/resources/*.xpm "$appdir"/resources/
done

# determine Git commit ID
# appimagetool uses this for naming the file
VERSION="$(cd "$REPO_ROOT" && git rev-parse --short HEAD)"
export VERSION
#echo "$VERSION" > ~/version

# prepend GitHub run number if possible
if [ "$GITHUB_RUN_NUMBER" != "" ]; then
	export VERSION="$GITHUB_RUN_NUMBER-$VERSION"
fi


# remove unnecessary files from AppDirs
rm appimageupdatetool.AppDir/usr/bin/AppImageUpdate || true
rm appimageupdatetool.AppDir/usr/lib/*/libappimageupdate-qt*.so* || true
rm -rf appimageupdatetool.AppDir/usr/include || true
find appimageupdatetool.AppDir -type f -iname '*.a' -delete

find appimageupdatetool.AppDir/
export OUTPUT="appimageupdatetool"-"$ARCH".AppImage

# bundle application
cd appimageupdatetool.AppDir && (
	wget "$LIB4BN" -O ./lib4bin
	chmod +x ./lib4bin

	mv ./usr ./shared
	cp -v "$REPO_ROOT"/resources/appimageupdatetool.desktop ./ 
	cp -v "$REPO_ROOT"/resources/appimage.png ./
	ln -s appimage.png ./.DirIcon

	./lib4bin -p -v -k -s ./shared/bin/*
	ln ./sharun ./AppRun
	./sharun -g
)

# Make appimage with uruntime
wget "$APPIMAGETOOL" -O ./appimagetool
chmod +x ./appimagetool

echo "Generating AppImage..."
./appimagetool --comp zstd -n -u "$UPINFO" \
	"$PWD"/appimageupdatetool.AppDir "$PWD"/appimageupdatetool-"$ARCH".AppImage

# move AppImage to old cwd
mv appimageupdatetool*.AppImage* "$OLD_CWD"/
cd -
