# Exporting Conversations with Bookmarklets

Browser bookmarklets allow you to easily export conversation transcripts from Google Gemini and Google AIM (AI Mode) search pages.

## Quick Start

### 1. Create the Bookmarklet

Navigate to the `bookmarklets/` directory and run:

```bash
cd bookmarklets
python3 minify.py chat
```

This will output a `javascript:` URL that you can save as a bookmark.

### 2. Install the Bookmarklet

1. Copy the entire output (starting with `javascript:`)
2. Create a new bookmark in your browser
3. Paste the copied code as the bookmark URL
4. Give it a name (e.g., "Download Chat")

### 3. Use the Bookmarklet

1. Navigate to a Gemini or AIM conversation page
2. Click the bookmarklet in your bookmarks bar
3. The conversation will be downloaded as `transcript.txt`

## Available Bookmarklets

You can create three types of bookmarklets:

```bash
# Universal bookmarklet (recommended) - works on both Gemini and AIM
python3 minify.py chat

# Gemini-only bookmarklet
python3 minify.py gemini

# AIM-only bookmarklet
python3 minify.py aim
```

The universal `chat` bookmarklet automatically detects which site you're on and uses the appropriate extractor.

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

## Troubleshooting

### Bookmarklet doesn't work

1. Make sure you're on a Gemini or AIM conversation page
2. Try refreshing the page and clicking the bookmarklet again
3. Check that you copied the entire `javascript:` URL when creating the bookmark

### Nothing downloads

1. Check your browser's download settings
2. Make sure pop-ups aren't blocked for the site
3. Try opening the browser console (F12) to see if there are any errors

For more detailed debugging information, see [docs/eng/bookmarklets.md](eng/bookmarklets.md).

## Next Steps

After exporting conversations:

1. Save the transcript files to a topic directory (e.g., `data/my_topic/`)
2. Run the debrief generator: `cargo run -- data/my_topic`
3. Optionally run research: `cargo run --bin read_files -- data/my_topic --research`
