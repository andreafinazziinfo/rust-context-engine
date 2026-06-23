class Rtk < Formula
  desc "Rust Context Engine (rtk) — token-saving context engine for AI agents"
  homepage "https://github.com/andreafinazziinfo/rust-context-engine"
  license "Apache-2.0"
  version "2.3.1"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.3.1/rtk-macos-arm64.tar.gz"
      sha256 "b8f299788d080f7f004e7d2421f978f0c2db4aec661f4ae9c498d2995d68d124"
    else
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.3.1/rtk-macos-amd64.tar.gz"
      sha256 "8b6ae6d5b6ffb598fbcce775737fe936a181da4aaca2378d8380f3b75fc86bca"
    end
  elsif OS.linux?
    url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.3.1/rtk-linux-amd64.tar.gz"
    sha256 "09f83bf0dc93aa3555378d40f91d30411997a6261e7d67edc2e6118dbd4c1848"
  end

  def install
    bin.install "rtk"
  end

  test do
    system "#{bin}/rtk", "--version"
  end
end
