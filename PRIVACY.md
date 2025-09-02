# Privacy Policy

**Last Updated:** September 3, 2025

## Introduction

Harper AI Agent ("we", "our", or "us") respects your privacy and is committed to protecting your personal information. This Privacy Policy explains how we collect, use, disclose, and safeguard your information when you use our AI assistant application.

This policy applies to the Harper AI Agent software, which is a locally-running CLI application that connects to various AI providers and maintains conversation history.

## Data We Collect

### Essential Data (Required for Core Functionality)

**API Keys and Authentication Data:**
- API keys for AI providers (OpenAI, Sambanova, Google Gemini)
- Authentication tokens and credentials
- Provider-specific configuration settings

**Conversation and Session Data:**
- Chat messages and conversation history
- Session metadata (timestamps, session IDs)
- User prompts and AI responses
- Command execution history

**Configuration Data:**
- Application settings and preferences
- Database configuration
- MCP (Model Context Protocol) settings

### Performance and Analytics Data

**Usage Statistics:**
- Application usage patterns
- Feature utilization metrics
- Performance metrics and response times
- Error rates and failure patterns

**Device and System Information:**
- Operating system and version
- Hardware specifications
- Application version and build information
- Network connectivity status

### Log Data

**Application Logs:**
- Error logs and crash reports
- Debug information for troubleshooting
- Performance monitoring data

## How We Use Your Data

### Core Functionality
- **AI Processing**: API keys are used to authenticate with AI providers for processing your requests
- **Conversation Management**: Storing and retrieving conversation history for continuity
- **Command Execution**: Processing shell commands and system interactions
- **Configuration Management**: Maintaining user preferences and settings

### Service Improvement
- **Performance Optimization**: Analyzing usage patterns to improve response times
- **Bug Detection**: Identifying and fixing application issues
- **Feature Development**: Understanding user needs for new features

### Security and Compliance
- **Security Monitoring**: Detecting and preventing security threats
- **Data Integrity**: Ensuring conversation data remains accurate and uncorrupted
- **Compliance**: Meeting legal and regulatory requirements

## Data Sharing and Disclosure

### Third-Party Service Providers

**AI Providers:**
We share your prompts and conversation data with the AI providers you configure:
- OpenAI (for GPT models)
- Sambanova (for Meta-Llama models)
- Google Gemini (for multimodal capabilities)

These providers process your data according to their own privacy policies.

### No Unauthorized Sharing

We do **NOT** share your data with:
- Advertising networks
- Social media platforms
- Data brokers or aggregators
- Any third parties for marketing purposes

### Legal Requirements

We may disclose your information if required by:
- Law enforcement requests with proper legal authority
- Court orders or subpoenas
- Legal proceedings
- Protection of our rights or safety

## Data Storage and Security

### Local Storage Only

**Database Storage:**
- Conversation history stored in local SQLite database
- Session data and metadata
- User preferences and settings

**File Storage:**
- API keys in local `.env` files
- Configuration in TOML files
- Application logs and cache files

### Security Measures

**Encryption:**
- AES-256-GCM encryption for sensitive data
- Secure key generation and management
- Encrypted database storage

**Access Controls:**
- Local-only data access
- No remote server storage by default
- User-controlled data retention

**Data Integrity:**
- SHA-256 hashing for data verification
- Checksum validation for downloads
- Secure random number generation

## Your Rights and Choices

### Data Access and Control

**You have the right to:**
- **Access**: View all your stored conversation data
- **Delete**: Remove conversation history and session data
- **Modify**: Update API keys and configuration settings
- **Export**: Download your data in readable formats
- **Portability**: Move your data to other applications

### Opt-Out Options

**You can opt out of:**
- Analytics and performance monitoring
- Error reporting and crash logs
- Usage statistics collection

### Data Deletion

**To delete your data:**
1. Delete the SQLite database file (`chat_sessions.db`)
2. Remove API key files (`.env`)
3. Clear configuration files (`config/local.toml`)
4. Delete application cache and logs

## Cookies and Tracking

As a local CLI application, Harper AI Agent does not use traditional web cookies. However, we may collect certain identifiers for functionality and analytics.

### Essential Identifiers

**Session Management:**
- UUIDs for conversation session tracking
- Temporary identifiers for request correlation
- Device fingerprints for license management

### Analytics Tracking

**Usage Analytics:**
- Feature usage patterns
- Performance metrics
- Error occurrence rates
- Application version tracking

### Tracking Controls

**You can control tracking by:**
- Disabling analytics in configuration
- Running in offline mode
- Using privacy-focused configurations
- Regular data cleanup procedures

## Data Retention

### Retention Periods

**Active Data:**
- Conversation history: Retained until manually deleted
- API keys: Retained until updated or removed
- Configuration: Retained until changed

**Log Data:**
- Application logs: 30 days retention
- Error logs: 90 days retention
- Performance data: 365 days retention

### Automatic Cleanup

**Data is automatically cleaned up:**
- Expired sessions after 90 days of inactivity
- Temporary cache files after 7 days
- Old log files based on size limits

## International Data Transfers

### Local Processing

Since Harper AI Agent runs locally on your device:
- No data is transferred to our servers
- All processing occurs on your local machine
- AI provider data transfers are direct from your device

### Provider Locations

**Data may be transferred to:**
- OpenAI servers (United States)
- Sambanova servers (Various locations)
- Google servers (United States and other countries)

## Children's Privacy

Harper AI Agent is not intended for use by children under 13 years of age. We do not knowingly collect personal information from children under 13. If you are a parent or guardian and believe your child has provided us with personal information, please contact us.

## Changes to This Privacy Policy

### Policy Updates

We may update this Privacy Policy from time to time. We will notify you of any changes by:
- Updating the "Last Updated" date at the top
- Providing in-app notifications
- Posting announcements on our GitHub repository

### Your Continued Use

Your continued use of Harper AI Agent after any changes indicates your acceptance of the updated Privacy Policy.

## Contact Information

### Privacy Inquiries

If you have any questions about this Privacy Policy or our data practices, please contact us:

**Email:** harpertoken@icloud.com
**GitHub Issues:** [Create an issue](https://github.com/harpertoken/harper/issues)
**GitHub Discussions:** [Start a discussion](https://github.com/harpertoken/harper/discussions)

### Data Protection Officer

For data protection inquiries:
**Email:** harpertoken@icloud.com

### Response Time

We aim to respond to privacy inquiries within 30 days.

## Technical Details

<details>
<summary>🔧 Click to expand technical implementation details</summary>

### Encryption Implementation

**AES-256-GCM Encryption:**
- 256-bit key length
- Galois/Counter Mode for authenticated encryption
- Random nonce generation for each encryption operation
- Secure key derivation from user-provided passwords

**Database Security:**
- SQLite with encrypted storage options
- Prepared statements to prevent SQL injection
- Transaction-based operations for data integrity
- Automatic backup and recovery mechanisms

### Network Security

**HTTPS Only:**
- All API communications use HTTPS/TLS 1.3
- Certificate pinning for additional security
- Perfect Forward Secrecy (PFS) enabled

**Request Signing:**
- HMAC-SHA256 request signing for API authentication
- Timestamp-based request validation
- Replay attack prevention

### Audit Logging

**Security Events:**
- Authentication attempts (success/failure)
- Data access operations
- Configuration changes
- System-level operations

**Log Security:**
- Encrypted log storage
- Automatic log rotation
- Secure log transmission (when enabled)

</details>

## Compliance and Certifications

<details>
<summary>📋 Click to expand compliance information</summary>

### Security Standards

**We adhere to:**
- OWASP Security Guidelines
- NIST Cybersecurity Framework
- ISO 27001 principles (where applicable)
- Rust Security Best Practices

### Open Source Transparency

**Code Security:**
- Regular security audits
- Dependency vulnerability scanning
- Open source security disclosures
- Responsible disclosure program

### Third-Party Audits

**Independent Reviews:**
- Code security reviews
- Penetration testing
- Cryptographic implementation validation
- Performance and scalability testing

</details>

## Additional Resources

### Related Documentation

- [Harper AI Agent README](README.md)
- [Project Wiki](https://github.com/harpertoken/harper/wiki) - Detailed patterns, guides, and documentation

### Community Resources

- [GitHub Repository](https://github.com/harpertoken/harper)
- [Community Discussions](https://github.com/harpertoken/harper/discussions)
- [Issue Tracker](https://github.com/harpertoken/harper/issues)

---

**By using Harper AI Agent, you acknowledge that you have read and understood this Privacy Policy.**