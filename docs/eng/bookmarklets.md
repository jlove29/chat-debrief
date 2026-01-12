# Bookmarklets - Technical Documentation

This document provides technical details about the bookmarklet implementation for extracting conversation transcripts from Google Gemini and AIM.

## Architecture

### Files

- **`gemini.js`** - Extracts conversations from Gemini chat pages (gemini.google.com)
- **`aim.js`** - Extracts conversations from Google AIM search pages (google.com/search)
- **`chat.js`** - Universal bookmarklet that detects the current page and calls the appropriate extractor
- **`minify.py`** - Python script that converts JavaScript files into minified bookmarklets

### Individual Bookmarklets (gemini.js, aim.js)

Each file contains:
- `scrapeGemini()` / `scrapeAim()` - Extracts conversation turns from the page DOM
- `downloadGeminiFile()` / `downloadAimFile()` - Creates and downloads a text file
- `downloadGemini()` / `downloadAim()` - Main function that orchestrates the extraction

### Universal Bookmarklet (chat.js)

The `chat.js` bookmarklet:
1. Detects which site you're on (Gemini or AIM)
2. Calls the appropriate download function
3. Shows an error if you're on an unsupported site

When you run `python3 minify.py chat`, the script:
1. Reads `gemini.js` and removes the `downloadGemini();` call at the end
2. Reads `aim.js` and removes the `downloadAim();` call at the end
3. Reads `chat.js`
4. Combines all three files into one bookmarklet
5. Minifies the combined code

This ensures all necessary functions are available without duplicate execution.

## Function Naming

To avoid conflicts when combining files, functions are uniquely named:

**gemini.js:**
- `scrapeGemini()` - Scrapes Gemini conversations
- `downloadGeminiFile()` - Downloads the file

**aim.js:**
- `scrapeAim()` - Scrapes AIM conversations
- `downloadAimFile()` - Downloads the file

## Minification Process

The `minify.py` script:

1. **Reads source files** - Loads the JavaScript source code
2. **Removes execution calls** - Strips the final function call (e.g., `downloadGemini();`) to prevent auto-execution when combining files
3. **Combines files** (for `chat` type) - Concatenates gemini.js + aim.js + chat.js
4. **Minifies** - Removes whitespace and comments
5. **Wraps in IIFE** - Wraps in `(function(){...})()` for scope isolation
6. **Adds prefix** - Prepends `javascript:` for bookmarklet format

## DOM Selectors

### Gemini

The Gemini scraper looks for conversation turns using CSS selectors that target the chat interface structure. These may need updating if Google changes the page structure.

### AIM

The AIM scraper targets the search results page structure where AI Mode conversations are displayed.

## Debugging

### Console Testing

To debug a bookmarklet:

1. Open the browser console (F12 or Cmd+Option+J)
2. Copy the bookmarklet code (without the `javascript:` prefix)
3. Paste and run it in the console
4. Check for error messages

### Common Issues

- **"No element found"** - The page structure may have changed. Inspect the page DOM and update the selectors in the `.js` files.
- **"X is not defined"** - A function is missing. This usually happens if the minify script didn't combine files correctly. Regenerate the bookmarklet.
- **Nothing happens** - The bookmarklet may not be properly formatted. Check that it starts with `javascript:` and has no line breaks.

### Updating Selectors

If Google changes their page structure:

1. Inspect the page with browser DevTools
2. Find the new selectors for conversation elements
3. Update the appropriate `.js` file
4. Regenerate the bookmarklet with `minify.py`
5. Test on a live conversation page

## Output Format

The transcript format is designed to be easily parseable:

```
User: [user message]

Gemini/AIM: [model response]

--------------------------------------------------

User: [next user message]

Gemini/AIM: [next model response]

--------------------------------------------------
```

Each turn is separated by a line of dashes, making it easy to split conversations programmatically.

## Testing

### Manual Testing

1. Create a test conversation on Gemini or AIM
2. Generate the bookmarklet: `python3 minify.py chat`
3. Install it in your browser
4. Navigate to the test conversation
5. Click the bookmarklet
6. Verify the downloaded file contains all conversation turns

### Automated Testing

The `test_minify.py` script tests the minification process:

```bash
python3 test_minify.py
```

This verifies that:
- The minify script can read all source files
- Output is properly formatted with `javascript:` prefix
- Combined bookmarklets include all necessary functions

## Future Enhancements

Potential improvements:

1. **Auto-detect page changes** - Monitor for DOM structure changes and alert developers
2. **Filename customization** - Allow users to specify the output filename
3. **Format options** - Support JSON, Markdown, or other output formats
4. **Batch export** - Export multiple conversations at once
5. **Cloud sync** - Automatically upload to a specified location
