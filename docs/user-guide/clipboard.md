# Clipboard

Harper supports pasted text, image file references, and image clipboard paste from the TUI.

## Overview

Harper only reads clipboard content when you explicitly paste it or use a paste shortcut. Use this when you want to:

- Reference copied text from another application
- Paste a screenshot from the system clipboard
- Drag or paste image file paths from the terminal

## Image Processing

### How It Works

Harper can attach images in two ways:

- Press `Ctrl+Shift+V` in the TUI to read an image from the system clipboard, save it to a temporary PNG file, and insert it as an `@file` reference.
- Drag an image file into the terminal, or paste one or more image file paths. Harper turns supported paths into `@file` references.

This is useful for:

- Analyzing screenshots
- Reviewing diagrams
- Examining UI designs

### Supported Formats

Harper supports the following image formats:

- PNG (preferred)
- JPEG/JPG
- GIF
- WebP
- BMP
- TIFF

### Using Images

1. Copy an image to your clipboard and press `Ctrl+Shift+V` in Harper, or drag an image file into the terminal.
2. Harper inserts an `@file` reference into the input.
3. Send the message with any extra context you want the model to use.

Example workflow:

```
> I took a screenshot of an error message, let me show you
Ctrl+Shift+V
> Can you help me understand what this error means?
```

## Text Processing

### How It Works

Harper can accept pasted text from the terminal paste event. `Ctrl+U` also pastes the internal cut buffer, or reads text from the system clipboard if that buffer is empty. This includes:

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

## Best Practices

### For Images

- Ensure the image is clear and readable
- Include relevant context in your message
- For screenshots, capture the entire relevant area
- Prefer file-path paste or `Ctrl+Shift+V` when the terminal does not support direct image paste

### For Text

- Copy only the relevant portion
- Include any error codes or specific terminology
- Mention the source application if relevant
- Large multiline paste expands the input area up to the TUI limit

### General Tips

1. **Clipboard Read**: Harper reads clipboard content only when you paste text, press `Ctrl+U`, or press `Ctrl+Shift+V`.

2. **Privacy**: Remember that clipboard content is sent to the AI model when you use it. Avoid copying sensitive information like passwords or personal data.

3. **Large Content**: For very large text, consider copying just the relevant sections rather than entire documents.

## Troubleshooting

### Images Not Processing

- For clipboard images, use `Ctrl+Shift+V`, not plain text paste.
- For dragged files, verify the pasted path points to an existing supported image file.
- Check that the image file is not corrupted.

### Text Not Appearing

- Verify text is copied to clipboard
- Try copying again
- If normal terminal paste is blocked by your terminal, use `Ctrl+U` to read clipboard text.

### Large File Issues

- For very large images, consider resizing before attaching
- Text-heavy clipboard content may be truncated in very large cases
