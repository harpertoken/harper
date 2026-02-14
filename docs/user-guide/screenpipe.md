# Screenpipe Integration

Harper integrates with Screenpipe to search through your screen and audio history, giving AI context about what you've been working on.

## What is Screenpipe?

Screenpipe records your screen and audio, making it searchable. Harper can query this history to help answer questions about your recent work.

## Requirements

1. **Screenpipe installed and running**
   - See [screenpipe](https://github.com/screenpipe/screenpipe) for installation
   - Start screenpipe before using Harper

2. **Screenpipe server running**
   - Default: `http://localhost:3030`
   - Can be changed with `SCREENPIPE_URL` environment variable

## Usage

Harper automatically uses screenpipe when relevant. You can also explicitly ask:

```
search screenpipe for terminal commands
show me what I was working on 30 minutes ago
find that error message from earlier
```

## How It Works

1. Harper detects when screenpipe search would be helpful
2. It queries the screenpipe API
3. Results are displayed with timestamps and source apps

## Configuration

### Environment Variables

```bash
# Change screenpipe URL (default: http://localhost:3030)
export SCREENPIPE_URL="http://localhost:3030"
```

### Search Parameters

You can customize searches:
- **Query**: What to search for
- **Type**: `ocr` (screen text), `audio`, or both
- **Limit**: Number of results (default: 10)

Example in code:
```
[SCREENPIPE terminal error ocr 5]
```

This searches for "terminal error", using OCR, limiting to 5 results.

## Troubleshooting

### Connection Failed

If you see "Could not connect to screenpipe":
1. Ensure screenpipe is running
2. Check the URL is correct
3. Verify no firewall is blocking port 3030

### No Results

* Check screenpipe has recorded content
* Try a different search query
- Verify the correct content type (ocr vs audio)

### Privacy

* Screenpipe only records what you allow
* Harper only searches, never stores recordings
* You can pause screenpipe at any time
