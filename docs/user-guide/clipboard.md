# Clipboard Features

Harper provides powerful clipboard integration that allows you to work with both images and text from your system clipboard. This feature enables the AI to analyze and reference content you've copied.

## Overview

Harper can access your clipboard contents and use them as context for conversations. This is particularly useful when you want to:
- Analyze images you've captured
- Reference text from other applications
- Share code snippets or documents

## Image Processing

### How It Works

Harper can process images directly from your clipboard. This is useful for:
- Analyzing screenshots
- Reviewing diagrams
- Examining UI designs
- Reading handwritten notes

### Supported Formats

Harper supports the following image formats:
- PNG (preferred)
- JPEG/JPG
- Other common image formats depending on your system

### Using Images

1. Copy an image to your clipboard (using your system's copy function)
2. Paste or reference it in Harper
3. The AI will analyze the image and provide insights

Example workflow:
```
> I took a screenshot of an error message, let me show you
[paste image from clipboard]
> Can you help me understand what this error means?
```

## Text Processing

### How It Works

Harper can read text from your clipboard to provide context. This includes:
- Copied code snippets
- Error messages
- Documentation
- Any text content

### Using Text

1. Copy text to your clipboard from any application
2. Reference it in Harper

Example:
```
> Here's that error I mentioned
[paste text from clipboard]
> What does this error indicate?
```

Or directly reference clipboard content in your message.

## Best Practices

### For Images

- Ensure the image is clear and readable
- Include relevant context in your message
- For screenshots, capture the entire relevant area

### For Text

- Copy only the relevant portion
- Include any error codes or specific terminology
- Mention the source application if relevant

### General Tips

1. **Clipboard Check**: Harper reads the clipboard when you explicitly reference it or paste content.

2. **Privacy**: Remember that clipboard content is sent to the AI model when you use it. Avoid copying sensitive information like passwords or personal data.

3. **Large Content**: For very large text, consider copying just the relevant sections rather than entire documents.

4. **Format Preservation**: Text formatting may be preserved when copying from applications like code editors or word processors.

## Troubleshooting

### Images Not Processing

- Ensure the image is in a supported format (PNG, JPEG)
- Verify the image is actually in clipboard (try copying again)
- Check that the image file is not corrupted

### Text Not Appearing

- Verify text is copied to clipboard
- Try copying again
- Ensure you're not including non-text content

### Large File Issues

- For very large images, consider resizing before copying
- Text-heavy clipboard content may be truncated in very large cases
