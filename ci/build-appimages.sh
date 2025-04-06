#!/bin/sh

set -ex

export APPIMAGE_EXTRACT_AND_RUN=1

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

export ARCH=${ARCH:-"$(uname -m)"}

cmake "$REPO_ROOT" \
    -DBUILD_QT_UI=ON \
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
echo "$VERSION" ~/version

# prepend GitHub run number if possible
if [ "$GITHUB_RUN_NUMBER" != "" ]; then
    export VERSION="$GITHUB_RUN_NUMBER-$VERSION"
fi


# remove unnecessary binaries from AppDirs
rm appimageupdatetool.AppDir/usr/bin/AppImageUpdate
rm appimageupdatetool.AppDir/usr/bin/validate
rm appimageupdatetool.AppDir/usr/lib/*/libappimageupdate-qt*.so*
rm validate.AppDir/usr/bin/{AppImageUpdate,appimageupdatetool}
rm validate.AppDir/usr/lib/*/libappimageupdate*.so*


# remove other unnecessary data
find {appimageupdatetool,validate}.AppDir -type f -iname '*.a' -delete
rm -rf appimageupdatetool.AppDir/usr/include


# get linuxdeploy and its qt plugin
wget https://github.com/TheAssassin/linuxdeploy/releases/download/continuous/linuxdeploy-"$CMAKE_ARCH".AppImage
wget https://github.com/TheAssassin/linuxdeploy-plugin-qt/releases/download/continuous/linuxdeploy-plugin-qt-"$CMAKE_ARCH".AppImage
wget https://github.com/darealshinji/linuxdeploy-plugin-checkrt/releases/download/continuous/linuxdeploy-plugin-checkrt.sh
chmod +x linuxdeploy*.AppImage linuxdeploy-plugin-checkrt.sh

for app in appimageupdatetool validate; do
	find "$app".AppDir/
	export UPD_INFO="gh-releases-zsync|pkgforge-dev|AppImageUpdate|continuous|$app-*$ARCH.AppImage.zsync"
	
	# overwrite AppImage filename to get static filenames
	# see https://github.com/AppImage/AppImageUpdate/issues/89
	export OUTPUT="$app"-"$ARCH".AppImage
	
	# bundle application
	./linuxdeploy-"$CMAKE_ARCH".AppImage --appdir "$app".AppDir --output appimage -d "$REPO_ROOT"/resources/"$app".desktop -i "$REPO_ROOT"/resources/appimage.png --plugin checkrt
done

# move AppImages to old cwd
mv appimageupdatetool*.AppImage* "$OLD_CWD"/
mv validate*.AppImage* "$OLD_CWD"/

cd -
