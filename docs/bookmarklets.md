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

### 2. Set Up as Search Engine Shortcut

The fastest way to use the bookmarklet is via Chrome's search engine settings:

1. Copy the entire output (starting with `javascript:`)
2. Go to `chrome://settings/searchEngines` → **Site search** → **Add**
3. Fill in the fields:
   - **Name**: Download Chat
   - **Shortcut**: `chat` (or any short keyword you prefer)
   - **URL**: Paste the `javascript:` code here
4. Click **Add**

### 3. Use the Bookmarklet

1. Navigate to a Gemini or AIM conversation page
2. Press `Cmd + L` (or `Ctrl + L` on Windows/Linux) to focus the address bar
3. Type your shortcut keyword (e.g., `chat`) and press Enter
4. The conversation will be downloaded as `transcript.txt`

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

If you encounter issues, see the [engineering documentation](eng/bookmarklets.md) for detailed debugging information.

## Next Steps

After exporting conversations:

1. Copy the downloaded transcript files to a topic directory in the location where you run the pipeline (e.g., `data/my_topic/`)
2. Run the debrief generator: `cargo run -- data/my_topic`
3. Optionally run research: `cargo run --bin read_files -- data/my_topic --research`
