class Memo < Formula
  desc "Persistent memory for AI coding agents"
  homepage "https://github.com/rustkit-ai/memo-agent"
  version "0.1.8"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/rustkit-ai/memo-agent/releases/download/v#{version}/memo-aarch64-apple-darwin.tar.gz"
      sha256 :no_check
    end
    on_intel do
      url "https://github.com/rustkit-ai/memo-agent/releases/download/v#{version}/memo-x86_64-apple-darwin.tar.gz"
      sha256 :no_check
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/rustkit-ai/memo-agent/releases/download/v#{version}/memo-aarch64-unknown-linux-gnu.tar.gz"
      sha256 :no_check
    end
    on_intel do
      url "https://github.com/rustkit-ai/memo-agent/releases/download/v#{version}/memo-x86_64-unknown-linux-gnu.tar.gz"
      sha256 :no_check
    end
  end

  def install
    bin.install "memo"
  end

  test do
    assert_match "memo #{version}", shell_output("#{bin}/memo --version")
  end
end
