class HarperAi < Formula
  desc "AI for the terminal - translates natural language into reviewed, executable commands"
  homepage "https://github.com/harpertoken/harper"
  version "0.4.0"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/harpertoken/harper/archive/refs/tags/0.4.0.tar.gz"
      sha256 "b5ecf8227c2fb6510aa51d2c6454d80b8bad075e571456131dac1007a38bfef9"
    else
      url "https://github.com/harpertoken/harper/archive/refs/tags/0.4.0.tar.gz"
      sha256 "b5ecf8227c2fb6510aa51d2c6454d80b8bad075e571456131dac1007a38bfef9"
    end
  end

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release", "--manifest-path=Cargo.toml"
    
    # Check both possible locations for binaries
    if File.exist?("bin/harper")
      bin.install "bin/harper" => "harper"
      bin.install "bin/harper-batch" => "harper-batch" if File.exist?("bin/harper-batch")
    elsif File.exist?("target/release/harper")
      bin.install "target/release/harper" => "harper"
      bin.install "target/release/harper-batch" => "harper-batch" if File.exist?("target/release/harper-batch")
    else
      raise "Binary not found in expected location"
    end
  end

  test do
    assert_match "Harper", shell_output("#{bin}/harper --version")
  end
end
