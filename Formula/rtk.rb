class Rtk < Formula
  desc "Rust Context Engine (rtk) — token-saving context engine for AI agents"
  homepage "https://github.com/andreafinazziinfo/rust-context-engine"
  license "Apache-2.0"
  version "2.3.2"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.3.2/rtk-macos-arm64.tar.gz"
      sha256 "6c1f7359927e4444db83e98a11a29a5507b4ad84ede61ee6984c4b07aedcd08e"
    else
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.3.2/rtk-macos-amd64.tar.gz"
      sha256 "43a9a09f81f17be61c35d53ec88ba932873beadde5db1ea1e6c3b90209b955b3"
    end
  elsif OS.linux?
    url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.3.2/rtk-linux-amd64.tar.gz"
    sha256 "c8b160b39c1f6e0a2ccd4c685674ee03a6d14d0f14ae83b1dbc8237dd88a0258"
  end

  def install
    bin.install "rtk"
  end

  test do
    system "#{bin}/rtk", "--version"
  end
end
