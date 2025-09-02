# Privacy Policy

## Data Collection

### Essential Data
- API keys for AI providers (stored locally)
- Conversation history (SQLite database)
- Configuration settings (TOML files)

### Analytics Data
- Usage statistics
- Error logs
- Performance metrics

## Data Usage

### Core Functionality
- API authentication with providers
- Local conversation storage and retrieval
- Command execution processing
- Configuration management

### Service Operation
- Performance monitoring
- Error detection and reporting
- Usage analytics for optimization

## Data Sharing

### Third Parties
- AI providers receive prompts and conversation data
- No data shared with advertising or analytics platforms
- Data transmission only to configured AI endpoints

### Storage
- Local SQLite database for conversations
- Local file storage for configuration and keys
- AES-256-GCM encryption for sensitive data
- SHA-256 hashing for data integrity

## Data Control

### User Rights
- View stored conversation data
- Delete conversation history and sessions
- Modify API keys and configuration
- Export data in readable formats

### Data Deletion
- Remove SQLite database file
- Delete `.env` files
- Clear configuration files
- Remove cache and logs

## Data Retention

- Conversation history: Retained until deleted
- Logs: 30-90 days retention
- Cache: Automatic cleanup after 7 days

## Contact

GitHub Issues: https://github.com/harpertoken/harper/issues

## Technical Implementation

- AES-256-GCM encryption for sensitive data
- HTTPS/TLS 1.3 for all API communications
- SQLite with prepared statements
- HMAC-SHA256 request signing