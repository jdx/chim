class Chim < Formula
  desc "Cross-platform binary fetcher/runner"
  homepage "https://chim.sh"
  license "MIT"
  version "$CHIM_VERSION"

  on_macos do
    if Hardware::CPU.intel?
      url "https://chim.sh/releases/v$CHIM_VERSION/chim-v$CHIM_VERSION-macos-x64.tar.xz"
      sha256 "$CHIM_CHECKSUM_MACOS_X86_64"

      def install
        bin.install "bin/chim"
      end
    end
    if Hardware::CPU.arm?
      url "https://chim.sh/releases/v$CHIM_VERSION/chim-v$CHIM_VERSION-macos-arm64.tar.xz"
      sha256 "$CHIM_CHECKSUM_MACOS_ARM64"

      def install
        bin.install "bin/chim"
      end
    end
  end

  on_linux do
    if Hardware::CPU.arm? && Hardware::CPU.is_64_bit?
      url "https://chim.sh/releases/v$CHIM_VERSION/chim-v$CHIM_VERSION-linux-arm64.tar.xz"
      sha256 "$CHIM_CHECKSUM_LINUX_ARM64"

      def install
        bin.install "bin/chim"
      end
    end
    if Hardware::CPU.intel?
      url "https://chim.sh/releases/v$CHIM_VERSION/chim-v$CHIM_VERSION-linux-x64.tar.xz"
      sha256 "$CHIM_CHECKSUM_LINUX_X86_64"

      def install
        bin.install "bin/chim"
      end
    end
  end

  test do
    system "#{bin}/chim --version"
    (testpath/"test-chim").write <<EOF
#!/usr/bin/env chim
path = './test-chim-bin'
EOF
    (testpath/"test-chim-bin").write <<EOF
#!/bin/sh
echo "it works!"
EOF
    chmod 0755, testpath/"test-chim-bin"

    assert_match "it works!", shell_output("#{bin}/chim ./test-chim")
  end
end
