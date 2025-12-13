# Harper

Harper is an AI agent for multi-provider integration, command execution, and MCP protocol support with SQLite storage.

Harper provides a unified interface to multiple AI providers (OpenAI, Sambanova, Gemini) with persistent chat sessions, command execution capabilities, and Model Context Protocol (MCP) support. This documentation covers secure configuration and deployment to prevent common security vulnerabilities.

## Security Issues Detected

This configuration detects Harper misconfigurations that can lead to security vulnerabilities, specifically:

- **Missing API key validation** - Unauthenticated access to AI providers
- **Insecure command execution** - Shell injection and unauthorized system access
- **Unauthorized file operations** - Path traversal and sensitive file access
- **Weak session management** - Session hijacking and data leakage

Users can extend Harper's security features by configuring additional validation rules and access controls.

## Recommendation

To help mitigate these vulnerabilities, ensure that the following Harper security features are properly configured:

- **API key validation** - Never commit real API keys to version control
- **Command execution controls** - Use user approval for destructive operations
- **File operation restrictions** - Validate paths and require explicit consent
- **Session security** - Use secure session management and data encryption

## Secure Configuration

### Environment Setup

**Secure API Key Management**

```bash
# Create environment file (never commit to git)
cp config/env.example .env

# Edit .env with your actual API keys
GEMINI_API_KEY=your_secure_key_here
```

**Never commit sensitive data:**

```toml
# config/local.toml - Use placeholders only
[api]
api_key = "your_api_key_here"  # Never put real keys here
```

### Secure Installation

**Local Build with Security Checks**

```bash
git clone https://github.com/harpertoken/harper.git
cd harper

# Run security validation
bash scripts/validate.sh

# Build with security features
cargo build --release

# Configure securely
cp config/env.example .env
# Edit .env with real keys (file is gitignored)

cargo run --release
```

**Docker with Security**

```bash
git clone https://github.com/harpertoken/harper.git
cd harper

# Secure environment setup
cp config/env.example .env
# Configure API keys in .env

# Build and run securely
docker build -t harper .
docker run --rm -it --env-file .env \
  --read-only \
  --tmpfs /tmp \
  harper
```

## Security Fixes

### API Key Exposure Prevention

To fix the risk of API key exposure in configuration files, Harper implements environment-based credential management. The codebase shows that sensitive API keys were previously stored in `config/local.toml`, which could be accidentally committed to version control.

The most secure fix is to use environment variables for all sensitive credentials, with configuration files containing only placeholders. This ensures that real API keys are never stored in the repository, mitigating credential exposure risks.

Specifically:

1. **Use environment variables** for all API keys (`GEMINI_API_KEY`, `OPENAI_API_KEY`, etc.)
2. **Store only placeholders** in configuration files that are safe to commit
3. **Configure `.gitignore`** to exclude `.env` files containing real credentials
4. **Implement runtime validation** to ensure required environment variables are set

### Command Injection Prevention

To fix the risk of command injection in shell execution, Harper implements user approval and input sanitization for all command operations. The codebase shows that direct command execution (`!command`) could potentially execute malicious commands if not properly validated.

The most secure fix is to implement approval-based command execution with input validation, ensuring that potentially dangerous commands require explicit user consent before execution.

Specifically:

1. **Validate command input** to prevent shell injection patterns (`;`, `|`, `&`, etc.)
2. **Require user approval** for all command execution via interactive prompts
3. **Log all command operations** for audit trails
4. **Limit command scope** to safe operations within the project directory

### File Operation Security

To fix the risk of unauthorized file access and path traversal attacks, Harper implements comprehensive file operation validation. The codebase shows that file operations (`[READ_FILE path]`, `[WRITE_FILE path content]`) could potentially access sensitive files or directories outside the intended scope.

The most secure fix is to implement path validation and user consent for all file operations, ensuring that only authorized file access is permitted.

Specifically:

1. **Validate all file paths** to prevent directory traversal (`../`, absolute paths)
2. **Require user approval** for write operations and potentially sensitive reads
3. **Restrict file operations** to the project workspace by default
4. **Implement file type restrictions** and size limits for safety

## Implementation Details

### Environment-Based Configuration

Harper's configuration system prioritizes environment variables for sensitive data:

```rust
// Secure credential loading (src/main.rs)
let mut api_key = config.api.api_key.clone();
if config.api.provider == "Gemini" {
    if let Ok(env_key) = std::env::var("GEMINI_API_KEY") {
        api_key = env_key;  // Override with secure env var
    }
}
```

This ensures that committed configuration files contain only placeholders, while runtime environment provides actual credentials.

### Command Execution Safety

Harper implements multi-layer command security:

```rust
// Command validation and approval (src/tools/shell.rs)
if command_str.chars().any(|c| matches!(c, ';' | '|' | '&' | '`' | '$' | '(' | ')')) {
    return Err(HarperError::Command("Dangerous characters detected".to_string()));
}

// User approval required
println!("Execute command? {} (y/n): ", command_str);
```

### File Operation Controls

Path validation prevents unauthorized access:

```rust
// Path security validation (src/tools/filesystem.rs)
fn validate_path(path: &str) -> Result<(), HarperError> {
    if path.contains("..") {
        return Err(HarperError::Security("Path traversal detected"));
    }
    // Additional validation logic...
}
```

## Benefits

These security implementations provide:

- **Zero credential exposure** - API keys never committed to version control
- **Runtime attack prevention** - Command injection and path traversal blocked
- **User-controlled operations** - Explicit consent required for destructive actions
- **Comprehensive audit trails** - All operations logged for security review
- **Maintainable security** - Modular security components that can be extended

No changes are required to existing functionality, and the fixes are entirely local to their respective security modules. The implementation maintains backward compatibility while significantly improving the security posture.

## References

- [Harper Documentation](docs/)
- [Contributing Guide](docs/CONTRIBUTING.md)
- [Security Policy](docs/SECURITY.md)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [OWASP API Security](https://owasp.org/www-project-api-security/)
- [Common Weakness Enumeration](https://cwe.mitre.org/)
