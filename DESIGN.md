# Processing pipeline for exported conversations

## Overview

This application processes conversation transcripts and generates structured DEBRIEF summaries using the Gemini API. It's designed as a hybrid library + binary crate to support both CLI usage and programmatic integration.

## Architecture

### Project Structure

```
src/
├── lib.rs          # Library entry point, exports public modules
├── main.rs         # Binary entry point, CLI interface
├── gemini.rs       # Gemini API interaction and prompt building
├── processor.rs    # File system operations (read/write/mark)
└── autorater.rs    # AI-powered debrief quality evaluation

tests/
└── integration_test.rs  # End-to-end pipeline tests

testdata/
├── hamstring_injury/    # Test data: medical conversation
└── hard_drive/          # Test data: technical troubleshooting
```

### Module Responsibilities

#### `main.rs`
- CLI argument parsing
- Directory validation
- Orchestrates the pipeline workflow

#### `processor.rs`
- `read_directory_files()`: Reads and categorizes files in a directory
- `write_debrief()`: Writes DEBRIEF.md content
- `mark_files_as_read()`: Renames processed files with `_read` suffix
- `process_files()`: Orchestrates calls to Gemini API

#### `gemini.rs`
- `analyze_files()`: Main API call to Gemini for debrief generation
- `build_prompt()`: Constructs prompts for new or updated debriefs
- `format_files()`: Formats file contents for inclusion in prompts
- `format_debrief_as_markdown()`: Converts structured debrief to markdown
- Defines `Debrief` and `DebriefItem` structs for structured output

#### `autorater.rs`
- `evaluate_debrief()`: Uses Gemini to evaluate debrief quality
- Returns score (1-10), reasoning, and list of issues
- Reusable module for quality assurance in other workflows

## Chrome Bookmarklets

### Overview

The `bookmarklets/` directory provides JavaScript bookmarklets that automate conversation export from Google AI chat interfaces. These bookmarklets eliminate the need for manual copy-pasting of conversations and can be run directly from your browser's url box.

### Available Bookmarklets

#### `chat.js` - Universal Exporter
A smart wrapper that detects the current site and delegates to the appropriate exporter:
- Checks `location.href` for domain
- Calls `downloadGemini()` for `gemini.google.com`
- Calls `downloadAim()` for `google.com/search`
- Shows error message for unsupported sites

#### `gemini.js` - Gemini Chat Exporter
Extracts conversations from Gemini chat interface:
- **Scraping**: Queries `.chat-history` elements
- **Parsing**: Iterates through child elements to extract user/model turns
- **Extraction**: Gets user text and `.model-response-text` content
- **Formatting**: Formats as "User: ...\n\nGemini: ..." with separators
- **Download**: Creates blob and triggers download as `transcript.txt`

#### `aim.js` - AIM Search Exporter
Extracts conversations from Google Search with AIM:
- **Scraping**: Queries `[data-scope-id="turn"]` elements
- **Parsing**: Extracts headings (`[role="heading"][aria-level="2"]`) for user queries
- **Content Extraction**: Processes `[data-subtree="aimc"]` for model responses
  - Handles ordered/unordered lists
  - Extracts headings and text elements
  - Filters out buttons and duplicate content
- **Formatting**: Formats as "User: ...\n\nAIM: ..." with separators
- **Download**: Creates blob and triggers download as `transcript.txt`

### Installation

1. Run the minify script: `python3 bookmarklets/minify.py chat` (or `gemini`/`aim` for specific extractors)
2. Copy the generated `javascript:` URL
3. Create a new bookmark in your browser
4. Paste the code as the bookmark URL
5. Give it a name (e.g., "Download Chat")

### Usage

1. Navigate to a Gemini or AIM conversation page
2. Click the bookmarklet in your bookmarks bar
3. `transcript.txt` downloads automatically
4. Move file to appropriate data directory for processing

### Technical Details

- **DOM Querying**: Uses `querySelector` and `querySelectorAll` for element selection
- **Error Handling**: Validates presence of expected elements before scraping
- **Console Feedback**: Logs extraction status with colored messages
- **Blob API**: Creates downloadable text files client-side
- **Content Cleaning**: Trims whitespace and filters irrelevant elements
- **Minification**: Python script combines and minifies JavaScript into bookmarklet format

## Data Flow

### Input

Conversation transcripts are exported using Chrome bookmarklets (see Chrome Bookmarklets section above).

Directory structure:
```
data/
└── topic1/
    ├── 1.txt
    ├── 2.txt
    ├── 3.txt
    └── DEBRIEF.md
└── topic2/
    ├── 1.txt
    ├── 2.txt
    └── DEBRIEF.md
```

### Processing Workflow

1. **Read Phase** (`processor::read_directory_files`)
   - Scan directory for `.txt` files
   - Separate unread files from `_read` files
   - Read existing `DEBRIEF.md` if present

2. **Analysis Phase** (`gemini::analyze_files`)
   - Build prompt with file contents and existing debrief
   - Call Gemini API with structured output schema
   - Parse JSON response into `Debrief` struct

3. **Write Phase** (`processor::write_debrief`)
   - Convert `Debrief` to markdown format
   - Write to `DEBRIEF.md`

4. **Mark Phase** (`processor::mark_files_as_read`)
   - Rename processed files: `1.txt` → `1_read.txt`
   - Prevents reprocessing on subsequent runs

### Output

`DEBRIEF.md` contains a structured summary focusing on:
- User's needs and current state
- User's actions and progress (not AI recommendations)
- Timeline and context for future conversations

Processed files are marked with `_read` suffix to track what's been analyzed.

## Testing Strategy

### Unit Tests

Each module includes `#[cfg(test)]` sections testing:
- **gemini.rs**: Prompt building, formatting, serialization
- **processor.rs**: File operations, workflow integration
- **autorater.rs**: Prompt construction, response parsing

Run with: `cargo test --lib`

### Integration Tests

End-to-end tests in `tests/integration_test.rs`:
- Create temporary directories
- Copy test data from `testdata/`
- Run full pipeline via `cargo run`
- Verify `DEBRIEF.md` creation and updates
- Use autorater to validate output quality (score ≥ 6/10)
- Marked `#[ignore]` due to API calls and runtime

Run with: `cargo test --test integration_test -- --ignored`

## API Integration

- **Model**: `gemini-3-flash-preview`
- **Authentication**: `GEMINI_API_KEY` environment variable
- **Structured Output**: JSON schema for `Debrief` struct
- **Rate Limiting**: None currently implemented

## Future Enhancements

- Support running on all directories recursively
- Chrome extension for automatic conversation export (bookmarklets currently provide this functionality)
- Configurable models and prompts
- Incremental processing (only new files)
- Rate limiting and retry logic
- Auto-detect and handle different conversation formats from various AI platforms