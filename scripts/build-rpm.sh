#!/usr/bin/env bash
set -euxo pipefail

CHIM_VERSION=$(./scripts/get-version.sh)

tar -xvJf "dist/chim-$CHIM_VERSION-linux-x64.tar.xz"
fpm -s dir -t rpm \
	--name chim \
	--license MIT \
	--version "${CHIM_VERSION#v*}" \
	--architecture x86_64 \
	--description "Cross-platform binary shims with optional remote fetching" \
	--url "https://chim.sh" \
	--maintainer "Jeff Dickey @jdxcode" \
	chim/bin/chim=/usr/bin/chim

tar -xvJf "dist/chim-$CHIM_VERSION-linux-arm64.tar.xz"
fpm -s dir -t rpm \
	--name chim \
	--license MIT \
	--version "${CHIM_VERSION#v*}" \
	--architecture aarch64 \
	--description "Cross-platform binary shims with optional remote fetching" \
	--url "https://chim.sh" \
	--maintainer "Jeff Dickey @jdxcode" \
	chim/bin/chim=/usr/bin/chim

cat <<EOF >~/.rpmmacros
%_signature gpg
%_gpg_name 7E07A8D14B7A5595
EOF

mkdir -p dist/rpmrepo/packages
cp -v packaging/rpm/chim.repo dist/rpmrepo
cp -v ./*.rpm dist/rpmrepo/packages
rpm --addsign dist/rpmrepo/packages/*.rpm
createrepo dist/rpmrepo
gpg --batch --yes --detach-sign --armor dist/rpmrepo/repodata/repomd.xml
