# Save History - Conversation Context Manager

## Overview

**Problem:** Every time you start a new conversation with an AI model, you need to get it up to speed about what you're working on. Keeping everything in one thread pollutes context, and there's no solution for moving between accounts or products (Gemini / AIM).

**Solution:** This tool organizes context within topics by generating topic debriefs from conversation history. These debriefs can be used to prepopulate context at the beginning of new chat threads, allowing you to maintain continuity across conversations and platforms.

## How It Works

1. **Export** conversations from Gemini or AIM using Chrome DevTools snippets
2. **Process** exported transcripts to generate concise debriefs using AI
3. **Reuse** debriefs to bootstrap context in new conversations

---

# Chrome Snippets for Exporting Conversations

The `chrome_snippets/` directory contains JavaScript snippets that can be run in Chrome DevTools to automatically export conversation transcripts from Google AI chat interfaces.

## Available Snippets

### `chat.js` - Universal Exporter
Automatically detects which Google AI service you're using and runs the appropriate exporter:
- Detects `gemini.google.com` → runs Gemini exporter
- Detects `google.com/search` → runs AIM exporter
- Shows error for unsupported sites

### `gemini.js` - Gemini Chat Exporter
Exports conversations from `gemini.google.com`:
- Scrapes `.chat-history` elements
- Extracts user queries and Gemini responses
- Downloads as `transcript.txt` with formatted turns

### `aim.js` - AIM Search Exporter
Exports conversations from Google Search with AIM:
- Scrapes `[data-scope-id="turn"]` elements
- Extracts user queries and AIM responses
- Handles lists, headings, and structured content
- Downloads as `transcript.txt` with formatted turns

## Installing Chrome Snippets

1. **Open Chrome DevTools**
   - Press `F12` or `Cmd+Option+I` (Mac) / `Ctrl+Shift+I` (Windows/Linux)
   - Or right-click on the page and select "Inspect"

2. **Navigate to Sources Panel**
   - Click the "Sources" tab in DevTools
   - Look for the "Snippets" tab in the left sidebar
   - If you don't see it, click the `>>` button to find it

3. **Create a New Snippet**
   - Click "+ New snippet" at the bottom of the Snippets pane
   - Give it a descriptive name (e.g., "Export Chat")

4. **Copy Snippet Code**
   - Open the desired snippet file from `chrome_snippets/`
   - Copy the entire contents
   - Paste into the snippet editor in DevTools

5. **Save the Snippet**
   - Press `Cmd+S` (Mac) / `Ctrl+S` (Windows/Linux)
   - Or right-click the snippet name and select "Save"

## Using the Snippets

1. Navigate to a Google AI chat page (Gemini or AIM)
2. Open DevTools (`F12`)
3. Go to Sources → Snippets
4. Right-click your snippet and select "Run" (or press `Cmd+Enter` / `Ctrl+Enter`)
5. The conversation will be downloaded as `transcript.txt`
6. Move the file to your data directory for processing

**Tip:** For the universal exporter (`chat.js`), you only need one snippet that works on both platforms.

# Running the pipeline

### Prerequisites
- Rust and Cargo installed
- `GEMINI_API_KEY` environment variable set (required for API calls)

### Running
Run the utility using `cargo run` followed by the directory path you want to read:
```bash
cargo run -- <directory_path>
```

Example:
```bash
cargo run -- data/coder_agent_vim
```

## Testing

This project includes comprehensive unit and integration tests.

### Unit Tests

Run unit tests for the library:
```bash
cargo test --lib
```

This tests:
- `gemini.rs`: Prompt building, file formatting, and serialization
- `processor.rs`: File reading, debrief writing, and file marking
- `autorater.rs`: Prompt building and response serialization

### Integration Tests

Integration tests validate the entire pipeline using test data in `testdata/`:
```bash
cargo test --test integration_test -- --ignored
```

**Note:** Integration tests are marked with `#[ignore]` because they:
- Make real API calls to Gemini
- Require `GEMINI_API_KEY` to be set
- Take longer to run (30-60 seconds per test)

Run a specific integration test:
```bash
cargo test test_integration_hamstring_injury -- --ignored --nocapture
cargo test test_integration_hard_drive -- --ignored --nocapture
```

## Autorater Module

The `autorater` module provides AI-powered evaluation of generated DEBRIEF content. It's available as a reusable library component.

### Usage

```rust
use read_files::autorater;

#[tokio::main]
async fn main() {
    let input_files = vec![
        "File 1 content".to_string(),
        "File 2 content".to_string(),
    ];
    let debrief_content = "# Summary\n\nGenerated debrief...";
    
    let result = autorater::evaluate_debrief(
        &input_files,
        debrief_content,
        "Context for evaluation",
    ).await.expect("Failed to evaluate");
    
    println!("Score: {}/10", result.score);
    println!("Reasoning: {}", result.reasoning);
    for issue in result.issues {
        println!("Issue: {}", issue);
    }
}
```

### Evaluation Criteria

The autorater evaluates debriefs on:
1. Accuracy in summarizing key information from input files
2. Focus on user needs, progress, and actions (not AI recommendations)
3. Organization and clarity
4. Appropriate level of detail without verbosity

Returns a score (1-10), reasoning, and a list of specific issues.