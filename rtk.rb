class Rtk < Formula
  desc "Rust Context Engine (rtk) — token-saving context engine for AI agents"
  homepage "https://github.com/andreafinazziinfo/rust-context-engine"
  license "Apache-2.0"
  version "2.3.0"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.3.0/rtk-macos-arm64.tar.gz"
      sha256 "d503b991fc388cd95e5e67e9b0970c9043af7f46987aec4690153e8c0fcee93f"
    else
      url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.3.0/rtk-macos-amd64.tar.gz"
      sha256 "f20d92f2b1c41c4d2a7f8b0205a162a28783f30ef71232611d5903ee4ad855ce"
    end
  elsif OS.linux?
    url "https://github.com/andreafinazziinfo/rust-context-engine/releases/download/v2.3.0/rtk-linux-amd64.tar.gz"
    sha256 "9fd1c9160566e95a6c72ea8f09e1e37346e2e8ec68f1adf01939ac08c1d039b5"
  end

  def install
    bin.install "rtk"
  end

  test do
    system "#{bin}/rtk", "--version"
  end
end
