class Rtk < Formula
  desc "Rust Context Engine (rtk) — token-saving context engine for AI agents"
  homepage "https://github.com/andreafinazziinfo/rust-context-engine"
  license "Apache-2.0"
  version "2.4.2"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.4.2/rtk-macos-arm64.tar.gz"
      sha256 "dc65ae7668ba6e69f757a9c77945b2c970e6c6eb3dd20de60db9b60ece2d66e1"
    else
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.4.2/rtk-macos-amd64.tar.gz"
      sha256 "1e0cf83c90ed3b7c07246c5d396b3d3a9a3e5ea0f7a48f162d6e01167599ace8"
    end
  elsif OS.linux?
    url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.4.2/rtk-linux-amd64.tar.gz"
    sha256 "52557770e6c55db292253a2c3a4c13e0402f78eec2bf10c31c1a403edc9b2355"
  end

  def install
    bin.install "rtk"
  end

  test do
    system "#{bin}/rtk", "--version"
  end
end
