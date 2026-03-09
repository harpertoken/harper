class HarperAi < Formula
  desc "AI for the terminal - translates natural language into reviewed, executable commands"
  homepage "https://github.com/harpertoken/harper"
  version "0.7.0"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/harpertoken/harper/archive/refs/tags/v0.7.0.tar.gz"
      sha256 "d5558cd419c8d46bdc958064cb97f963d1ea793866414c025906ec15033512ed"
    else
      url "https://github.com/harpertoken/harper/archive/refs/tags/v0.7.0.tar.gz"
      sha256 "d5558cd419c8d46bdc958064cb97f963d1ea793866414c025906ec15033512ed"
    end
  end

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release", "-p", "harper-ui", "--bin", "harper", "--bin", "harper-batch", "--manifest-path=Cargo.toml"

    # Check both possible locations for binaries
    if File.exist?("bin/harper")
      bin.install "bin/harper" => "harper"
      bin.install "bin/harper-batch" => "harper-batch" if File.exist?("bin/harper-batch")
    elsif File.exist?("target/release/harper")
      bin.install "target/release/harper" => "harper"
      bin.install "target/release/harper-batch" => "harper-batch" if File.exist?("target/release/harper-batch")
    else
      raise "Binary not found in expected location. Contents of target/release: #{Dir.glob("target/release/*")}"
    end
  end

  test do
    assert_match "Harper", shell_output("#{bin}/harper --version")
  end
end
