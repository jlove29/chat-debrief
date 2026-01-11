# Save History - Conversation Context Manager

## Overview

**Problem:** Every time you start a new conversation with an AI model, you need to get it up to speed about what you're working on. Keeping everything in one thread pollutes context, and there's no solution for moving between accounts or products (Gemini / AIM).

**Solution:** This tool organizes context within topics by generating topic debriefs from conversation history. These debriefs can be used to prepopulate context at the beginning of new chat threads, allowing you to maintain continuity across conversations and platforms.

## How It Works

1. **Export** conversations from Gemini or AIM using browser bookmarklets
2. **Process** exported transcripts to generate concise debriefs using AI
3. **Reuse** debriefs to bootstrap context in new conversations

---

# Exporting Conversations

The `bookmarklets/` directory contains browser bookmarklets for exporting conversation transcripts from Google AI chat interfaces (Gemini and AIM).

**See [bookmarklets/README.md](bookmarklets/README.md) for complete documentation** on:
- Creating and installing bookmarklets
- Using the universal chat bookmarklet
- Debugging tips
- How the minification process works


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

### Async Research (Optional)

The tool can automatically research open questions and topics from your debriefs:

```bash
# Generate debrief AND run research in one command
cargo run --bin read_files -- <directory_path> --research

# Or run research separately on an existing debrief
cargo run --bin async_researcher <data_directory> <topic_name>
```

Examples:
```bash
# Integrated: process conversations + research
cargo run --bin read_files -- data/hamstring_injury --research

# Standalone: research existing debrief
cargo run --bin async_researcher data hamstring_injury

# Cross-topic analysis (finds connections between topics)
cargo run --bin async_researcher data
```

Research insights are saved to `RESEARCH.md` in the topic directory. See [ASYNC_RESEARCH.md](ASYNC_RESEARCH.md) and [QUICKSTART_RESEARCH.md](QUICKSTART_RESEARCH.md) for details.

## Testing

This project includes comprehensive unit and integration tests.

### Unit Tests

Run unit tests for the library:
```bash
cargo test --lib
```

This tests:
- `debrief.rs`: Prompt building, debrief formatting, and serialization
- `gemini_utils.rs`: Shared API utilities and file formatting
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

Test the async research functionality:
```bash
cargo test test_research_hard_drive -- --ignored --nocapture
cargo test test_research_all_directories -- --ignored --nocapture
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