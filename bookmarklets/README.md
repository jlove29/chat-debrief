# Chrome Bookmarklets for Chat Transcript Extraction

This directory contains JavaScript bookmarklets that extract conversation transcripts from Google Gemini and Google AIM (AI Mode) search pages.

## Files

- **`gemini.js`** - Extracts conversations from Gemini chat pages (gemini.google.com)
- **`aim.js`** - Extracts conversations from Google AIM search pages (google.com/search)
- **`chat.js`** - Universal bookmarklet that detects the current page and calls the appropriate extractor
- **`minify.py`** - Python script that converts JavaScript files into minified bookmarklets

## Usage

### Creating Bookmarklets

Run the minify script with the type of bookmarklet you want to create:

```bash
# Create Gemini bookmarklet
python3 minify.py gemini

# Create AIM bookmarklet
python3 minify.py aim

# Create universal chat bookmarklet (combines both)
python3 minify.py chat
```

The script will output a `javascript:` URL that you can copy and save as a browser bookmark.

### Installing a Bookmarklet

1. Run the minify script for your desired type
2. Copy the entire output (starting with `javascript:`)
3. Create a new bookmark in your browser
4. Paste the copied code as the bookmark URL
5. Give it a name (e.g., "Download Chat")

### Using a Bookmarklet

1. Navigate to a Gemini or AIM conversation page
2. Click the bookmarklet in your bookmarks bar
3. The conversation will be downloaded as `transcript.txt`

## How It Works

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

## Debugging

If a bookmarklet doesn't work:

1. Open the browser console (F12 or Cmd+Option+J)
2. Copy the bookmarklet code (without the `javascript:` prefix)
3. Paste and run it in the console
4. Check for error messages

Common issues:
- **"No element found"** - The page structure may have changed
- **"X is not defined"** - A function is missing (regenerate the bookmarklet)
- **Nothing happens** - The bookmarklet may not be properly formatted

## Output Format

Downloaded transcripts are formatted as:

```
User: [user message]

Gemini/AIM: [model response]

--------------------------------------------------

User: [next user message]

Gemini/AIM: [next model response]

--------------------------------------------------
```
