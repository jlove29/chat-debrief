# Save History - Conversation Context Manager

## Overview

**Problem:** Every time you start a new conversation with an AI model, you need to get it up to speed about what you're working on. Keeping everything in one thread pollutes context, and there's no solution for moving between accounts or products (Gemini / AIM).

**Solution:** This tool organizes context within topics by generating topic debriefs from conversation history. These debriefs can be used to prepopulate context at the beginning of new chat threads, allowing you to maintain continuity across conversations and platforms.

## Features

- 📝 **Automated Debrief Generation** - AI-powered summaries of conversation history
- 🔍 **Async Research** - Automatic research on open questions and topics from debriefs
- 📊 **Cross-Topic Analysis** - Find connections between different conversation topics
- 🔖 **Browser Bookmarklets** - Easy export from Gemini and AIM chat interfaces
- ✅ **Quality Evaluation** - Built-in autorater for debrief quality assessment

## Quick Start

### Prerequisites
- Rust and Cargo installed
- `GEMINI_API_KEY` environment variable set

### Basic Usage

```bash
# Generate a debrief from conversation files
cargo run -- data/my_topic

# Generate debrief + run research
cargo run --bin read_files -- data/my_topic --research
```

## Documentation

### User Documentation
- **[Research Guide](docs/research.md)** - Get started with the async research feature
- **[Bookmarklets Guide](docs/bookmarklets.md)** - Export conversations from Gemini/AIM

### Engineering Documentation
- **[Design & Architecture](docs/eng/design.md)** - System architecture and design decisions
- **[Research Deep Dive](docs/eng/research.md)** - Technical details of the research system
- **[Testing Guide](docs/eng/testing.md)** - How to run and write tests


## How It Works

1. **Export** - Use browser bookmarklets to export conversations from Gemini or AIM
2. **Process** - Run the tool to generate AI-powered debriefs from conversation files
3. **Research** (Optional) - Automatically research open questions and find insights
4. **Reuse** - Copy debriefs into new conversations to maintain context

## Project Structure

```
.
├── src/              # Rust source code
├── data/             # Topic directories with conversation files
├── bookmarklets/     # Browser bookmarklets for exporting conversations
├── docs/             # User-facing documentation
└── docs/eng/         # Engineering documentation
```

## Contributing

See [docs/eng/design.md](docs/eng/design.md) for architecture details and [docs/eng/testing.md](docs/eng/testing.md) for testing guidelines.

## License

MIT