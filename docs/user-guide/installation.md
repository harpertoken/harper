# Installation

This guide covers how to install Harper on your system. Harper is a Rust application that can be built from source.

## Prerequisites

Before installing Harper, ensure you have the following:

### Required Software

- **Rust 1.85.0 or later**: Harper is written in Rust. Install via [rustup](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Git**: For cloning the repository
  ```bash
  # On macOS
  brew install git

  # On Ubuntu/Debian
  sudo apt-get install git

  # On Fedora
  sudo dnf install git
  ```

### System Requirements

- **Operating System**: macOS, Linux, or Windows (with WSL)
- **RAM**: 4GB minimum, 8GB recommended
- **Disk Space**: 500MB for installation
- **Internet**: Required for API access

## Installation Methods

### Method 1: Build from Source (Recommended)

1. Clone the repository:
   ```bash
   git clone https://github.com/harpertoken/harper.git
   cd harper
   ```

2. Build the release version:
   ```bash
   cargo build --release
   ```

3. Build Harper:
   ```bash
   cargo build --release
   ```

4. Run Harper:
   ```bash
   # Option 1: Using cargo
   cargo run -p harper-ui --bin harper

   # Option 2: Using the built binary
   ./target/release/harper
   ```

### Method 2: Using Make

If the project includes a Makefile:

```bash
git clone https://github.com/harpertoken/harper.git
cd harper
make
make run
```

## Post-Installation

### Configure API Key

After installation, you'll need to configure your AI API key:

1. Create a config file:
   ```bash
   mkdir -p config
   nano config/local.toml
   ```

2. Add your API key:
   ```toml
   [api]
   key = "your-api-key-here"
   provider = "OpenAI"
   ```

3. Save and exit. You're ready to use Harper!

## Verifying Installation

Run Harper to verify everything works:

```bash
./target/release/harper
```

You should see a welcome message. If you get an error, check:
- Your Rust installation: `rustc --version`
- Your API key configuration
- Network connectivity

## Updating Harper

To update to the latest version:

```bash
git pull origin main
cargo build --release
```

## Uninstallation

To remove Harper:

```bash
# Remove the repository
cd ..
rm -rf harper

# Optionally remove Rust if needed
rustup self uninstall
```

## Troubleshooting

### Build Errors

If the build fails:
- Ensure Rust is up to date: `rustup update`
- Clear build cache: `cargo clean && cargo build --release`

### Permission Denied

On Unix systems, you may need to make the binary executable:
```bash
chmod +x target/release/harper
```

### Missing Dependencies

If you get library errors:
- **macOS**: Install Xcode Command Line Tools: `xcode-select --install`
- **Linux**: Install development libraries for your distribution
