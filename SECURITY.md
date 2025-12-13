# Security Policy

[![Security: Responsible Disclosure](https://img.shields.io/badge/security-responsible%20disclosure-blue)](https://github.com/harpertoken/harper/security/policy)

Harper takes security seriously. We appreciate your help in keeping Harper and its users safe by following this security policy.

## Table of Contents

- [Supported Versions](#supported-versions)
- [Reporting Security Vulnerabilities](#reporting-security-vulnerabilities)
- [Security Assessment](#security-assessment)
- [Security Best Practices](#security-best-practices)
- [Incident Response](#incident-response)
- [Contact](#contact)

## Supported Versions

We actively maintain security updates for the following versions:

| Version | Supported | Security Updates | Bug Fixes |
|---------|-----------|------------------|------------|
| **0.1.6** |  Active |  Yes |  Yes |
| 0.1.5 |  Limited |  Critical only |  Yes |
| 0.1.4 |  Limited |  Critical only |  Yes |
| < 0.1.4 |  End of Life |  No |  No |

**Legend:**
-  **Active**: Full security support and updates
-  **Limited**: Critical security fixes only
-  **End of Life**: No security updates provided

## Reporting Security Vulnerabilities

**🚨 Do not report security vulnerabilities through public GitHub issues.**

### How to Report

Please report security vulnerabilities by emailing:
- **Email**: harpertoken@icloud.com
- **Subject**: `[SECURITY] Harper Vulnerability Report`

### What to Include

When reporting a security vulnerability, please provide:

- **Issue Type**: Buffer overflow, injection, authentication bypass, etc.
- **Severity**: Critical, High, Medium, Low (with justification)
- **Affected Versions**: Which versions are impacted
- **File Paths**: Specific files and code locations affected
- **Reproduction Steps**: Clear, step-by-step instructions
- **Proof of Concept**: Code or detailed description demonstrating the issue
- **Potential Impact**: What an attacker could achieve
- **Mitigation**: Any suggested fixes or workarounds

### Response Process

1. **Acknowledgment**: We'll acknowledge receipt within 24 hours
2. **Investigation**: We'll investigate and validate the report within 48 hours
3. **Updates**: We'll provide regular updates on our progress
4. **Fix Development**: We'll develop and test a fix
5. **Disclosure**: We'll coordinate disclosure with you
6. **Resolution**: We'll release the fix and security advisory

### Responsible Disclosure

We follow responsible disclosure principles:

- We will not publicly disclose the vulnerability until a fix is available
- We will credit you (if desired) in our security advisory
- We will keep you informed throughout the process
- We will not pursue legal action for security research conducted in good faith

## Security Assessment

### Automated Security Scanning

Harper uses multiple automated security tools:

- **Cargo Audit**: Rust dependency vulnerability scanning
- **Cargo Deny**: License and dependency policy checking
- **Clippy**: Linting with security-focused rules

### Security Headers and Practices

- **Input Validation**: All user inputs are validated and sanitized
- **SQL Injection Protection**: Parameterized queries prevent SQL injection
- **XSS Prevention**: Output encoding prevents cross-site scripting
- **CSRF Protection**: State-changing operations require proper validation
- **Secure Defaults**: Conservative defaults that prioritize security

### Encryption and Data Protection

- **AES-GCM-256**: Strong encryption for sensitive data storage
- **Local Storage**: All data remains on the user's device
- **API Key Protection**: Keys are stored securely and never transmitted unnecessarily
- **Memory Safety**: Rust's memory safety prevents common vulnerabilities

## Security Best Practices

### For Users

1. **Keep Updated**: Use the latest version of Harper
2. **Secure API Keys**: Store API keys securely and rotate regularly
3. **Network Security**: Use Harper on trusted networks
4. **Regular Backups**: Backup your conversation data regularly
5. **Monitor Activity**: Review your chat sessions periodically

### For Developers

1. **Code Review**: All changes undergo security review
2. **Testing**: Comprehensive security testing for new features
3. **Dependency Updates**: Regular updates of all dependencies
4. **Secure Coding**: Follow OWASP and Rust security guidelines
5. **Audit Logging**: Security-relevant events are logged appropriately

## Incident Response

### If You Suspect a Security Issue

1. **Stop Using**: Immediately stop using the affected functionality
2. **Report**: Follow the reporting process above
3. **Monitor**: Watch for official updates and advisories
4. **Update**: Apply security patches as soon as available

### Official Security Advisories

Security advisories will be published at:
- [GitHub Security Advisories](https://github.com/harpertoken/harper/security/advisories)
- [Harper Changelog](CHANGELOG.md)
- Official communication channels

## Contact

For security-related questions or concerns:

- **Security Issues**: harpertoken@icloud.com
- **General Support**: [GitHub Issues](https://github.com/harpertoken/harper/issues)
- **Discussions**: [GitHub Discussions](https://github.com/harpertoken/harper/discussions)

## Acknowledgments

We appreciate the security research community for helping keep Harper secure. Security researchers who report valid vulnerabilities will be acknowledged in our security advisories (unless they request anonymity).

---

**Last Updated**: November 2025
**Version**: 1.0
