#!/usr/bin/env bash
set -euxo pipefail

git config --global user.name chim-bot
git config --global user.email release@chim.sh

CHIM_VERSION=$(cd chim && ./scripts/get-version.sh)
RELEASE_DIR=chim.sh/static/releases
rm -rf "${RELEASE_DIR:?}/$CHIM_VERSION"
mkdir -p "$RELEASE_DIR/$CHIM_VERSION"

cp artifacts/tarball-x86_64-pc-windows-gnu/*.zip "$RELEASE_DIR/$CHIM_VERSION"
cp artifacts/tarball-x86_64-pc-windows-gnu/*.zip "$RELEASE_DIR/chim-latest-windows.zip"

targets=(
	x86_64-unknown-linux-gnu
	aarch64-unknown-linux-gnu
	x86_64-apple-darwin
	aarch64-apple-darwin
)
for target in "${targets[@]}"; do
	cp "artifacts/tarball-$target/"*.tar.gz "$RELEASE_DIR/$CHIM_VERSION"
	cp "artifacts/tarball-$target/"*.tar.xz "$RELEASE_DIR/$CHIM_VERSION"
done

platforms=(
	linux-x64
	linux-arm64
	macos-x64
	macos-arm64
)
for platform in "${platforms[@]}"; do
	cp "$RELEASE_DIR/$CHIM_VERSION/chim-$CHIM_VERSION-$platform.tar.gz" "$RELEASE_DIR/chim-latest-$platform.tar.gz"
	cp "$RELEASE_DIR/$CHIM_VERSION/chim-$CHIM_VERSION-$platform.tar.xz" "$RELEASE_DIR/chim-latest-$platform.tar.xz"
done

pushd "$RELEASE_DIR"
sha256sum ./*.zip ./*.tar.xz ./*.tar.gz >SHASUMS256.txt
gpg --clearsign -u 7E07A8D14B7A5595 <SHASUMS256.txt >SHASUMS256.asc
popd

pushd "$RELEASE_DIR/$CHIM_VERSION"
sha256sum ./* >SHASUMS256.txt
gpg --clearsign -u 7E07A8D14B7A5595 <SHASUMS256.txt >SHASUMS256.asc
popd

export CHIM_VERSION RELEASE_DIR
./chim/scripts/render-chimstrap >chim.sh/static/chimstrap

rm -rf chim.sh/static/rpm
mv artifacts/rpm chim.sh/static/rpm

rm -rf chim.sh/static/deb
mv artifacts/deb chim.sh/static/deb

pushd chim.sh
git add . && git commit -m "$CHIM_VERSION"
popd

./chim/scripts/render-homebrew >homebrew-tap/chim.rb
pushd homebrew-tap
git add . && git commit -m "$CHIM_VERSION"
popd
