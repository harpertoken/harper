# Installation

This guide covers the supported ways to install Harper and how updates behave for each install source.

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

### Method 1: Homebrew

Use Homebrew if you want a package-managed install on macOS.

```bash
brew tap harpertoken/tap
brew install harpertoken/tap/harper-ai
```

To update later:

```bash
brew upgrade harpertoken/tap/harper-ai
```

### Method 2: Build from Source

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

### Method 3: Direct Release Artifact

Download the release artifact for your platform from GitHub Releases, extract it, and place the `harper` binary somewhere on your `PATH`.

Direct installs support Harper's built-in updater:

```bash
harper self-update --check
harper self-update
```

Direct self-update verifies the published manifest, checksum, and detached signature before replacing the local binary.

### Method 4: Using Make

If the project includes a Makefile:

```bash
git clone https://github.com/harpertoken/harper.git
cd harper
make
make run
```

## Post-Installation

### Configure Provider Access

After installation, configure your provider settings:

1. Create a config file:
   ```bash
   mkdir -p config
   cp config/local.example.toml config/local.toml
   nano config/local.toml
   ```

2. Add provider configuration:
   ```toml
   [provider]
   name = "openai"
   model = "gpt-5"
   ```

3. Export the matching provider credential, for example:
   ```bash
   export OPENAI_API_KEY="your-api-key-here"
   ```

4. Save and exit. You're ready to use Harper.

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

Update paths depend on how Harper was installed:

- **Homebrew**: `brew upgrade harpertoken/tap/harper-ai`
- **Direct release install**: `harper self-update --check` or `harper self-update`
- **Source build**:
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
