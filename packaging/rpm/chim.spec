Summary: Cross-platform binary shims with optional remote fetching
Name: chim
Version: 1.1.0
Release: 1
URL: https://chim.sh/
Group: System
License: MIT
Packager: @jdxcode
BuildRoot: /root/chim

%description
Chim is a cross-platform binary shims with optional remote fetching.

%install
mkdir -p %{buildroot}/usr/bin/
cp /root/chim/target/release/chim %{buildroot}/usr/bin

%files
/usr/bin/chim
