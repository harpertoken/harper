# Troubleshooting

This guide helps you resolve common issues you might encounter while using Harper.

## Common Issues

### API Key Problems

#### "Invalid API Key" Error

**Symptom**: You see an error message about an invalid API key.

**Solutions**:
1. Verify your API key is correct in `config/local.toml`
2. Check for extra spaces or quotes around the key
3. Ensure you have an active subscription with the provider
4. Try regenerating your API key from the provider's dashboard

#### "Rate Limit Exceeded" Error

**Symptom**: You see a rate limiting error message.

**Solutions**:
1. Wait a few minutes before trying again
2. Check your API usage limits
3. Consider switching to a different model
4. Contact your provider for rate limit increases

### Connection Issues

#### "Network Error" or "Connection Timeout"

**Symptom**: Harper can't connect to the AI service.

**Solutions**:
1. Check your internet connection
2. Verify your firewall isn't blocking the connection
3. Try using a different network
4. Check if the API service is down (provider status page)

#### "Proxy Error"

**Symptom**: Connection fails when using a proxy.

**Solutions**:
1. Configure proxy settings in your config:
   ```toml
   [network]
   proxy = "http://proxy:port"
   ```
2. Verify proxy credentials are correct
3. Try disabling the proxy temporarily to test

### Performance Issues

#### Slow Responses

**Symptom**: Responses take a long time to appear.

**Solutions**:
1. Try a faster model (e.g., gpt-3.5-turbo instead of gpt-4)
2. Reduce max_tokens setting
3. Check your network latency
4. Close other bandwidth-intensive applications

#### High Memory Usage

**Symptom**: Harper uses too much memory.

**Solutions**:
1. Reduce max_history in config
2. Start fresh sessions with `/clear`
3. Save and close old sessions
4. Restart Harper periodically

### Session Problems

#### Can't Load Session

**Symptom**: Session loading fails.

**Solutions**:
1. Verify the session file exists
2. Check file permissions
3. Ensure the session file isn't corrupted
4. Try loading a different session

#### Session Not Saving

**Symptom**: Save command seems to work but session is gone.

**Solutions**:
1. Check write permissions in the session directory
2. Verify disk space is available
3. Check the session_dir path exists

### Build and Installation

#### Build Fails

**Symptom**: cargo build command fails.

**Solutions**:
1. Update Rust: `rustup update`
2. Clean build: `cargo clean && cargo build --release`
3. Check for missing dependencies
4. Ensure you have enough disk space

#### Binary Won't Start

**Symptom**: Running `./target/release/harper` fails.

**Solutions**:
1. Make executable: `chmod +x target/release/harper`
2. Check if binary exists: `ls -la target/release/harper`
3. Verify libraries: `ldd target/release/harper`

### Clipboard Issues

#### Images Not Processing

**Symptom**: Clipboard images aren't being analyzed.

**Solutions**:
1. Verify the image is in clipboard (copy again)
2. Ensure it's a supported format (PNG, JPEG)
3. Check image file size (not too large)
4. Try pasting directly in terminal

#### Text Not Available

**Symptom**: Clipboard text isn't being read.

**Solutions**:
1. Copy text again
2. Ensure you're not copying from a restricted app
3. Try a simple text first

## Error Messages

### Common Error Codes

| Error | Meaning | Solution |
|-------|---------|----------|
| 401 | Unauthorized | Check API key |
| 403 | Forbidden | Check permissions |
| 404 | Not found | Check endpoint URL |
| 429 | Rate limited | Wait and retry |
| 500 | Server error | Try again later |
| 503 | Service unavailable | Check provider status |

### Getting More Help

If you're still stuck:

1. Check the GitHub Issues
2. Search the Discussions
3. Review the documentation
4. Create an issue with:
   - Your config (remove API key)
   - The exact error message
   - Steps to reproduce
   - Your system info

## Debug Mode

To get more detailed error information:

```toml
[app]
debug = true
verbose = true
```

This will show additional logging that can help diagnose issues.

## Resetting Harper

If all else fails, you can reset Harper to a clean state:

1. Delete the config directory (backup first):
   ```bash
   rm -rf config/local.toml
   ```

2. Clear sessions:
   ```bash
   rm -rf sessions/
   ```

3. Rebuild:
   ```bash
   cargo clean && cargo build --release
   ```
