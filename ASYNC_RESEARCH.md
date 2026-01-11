# Async Research Feature

## Overview

The async research feature adds proactive, intelligent research capabilities to chat-debrief. Instead of just summarizing conversations, the system now actively researches open questions, checks for updates, and finds connections between topics.

## Features

### 1. 💡 Proactive Gap Filling
Detects open questions and unresolved issues in your debriefs and automatically researches solutions.

**Example:**
- Debrief mentions: "User encountered error X when trying to configure Y"
- System researches: "How to fix error X in Y configuration"
- Appends solution to debrief before your next session

### 2. 🆕 Novelty/Update Checks
Monitors long-running topics for new developments, breaking changes, or updates.

**Example:**
- Debrief discusses: "Using Rust library X version 1.2"
- System checks: "Latest updates and breaking changes for Rust library X"
- Alerts you to version 2.0 release with migration guide

### 3. 🔗 Cross-Pollination
Analyzes debriefs across different topics to find valuable connections.

**Example:**
- Topic A: "Vector databases for semantic search"
- Topic B: "Rust async programming patterns"
- System researches: "Best Rust vector database clients with async support"

## Usage

### Option 1: Integrated Research (During Debrief Generation)

Run the main processor with the `--research` flag:

```bash
cargo run --bin read_files data/your_topic --research
```

This will:
1. Generate/update the debrief
2. Identify research tasks
3. Perform high-priority research
4. Append insights to the debrief

### Option 2: Standalone Async Research

Run research independently on an existing debrief:

```bash
# Research a specific topic
cargo run --bin async_researcher data hamstring_injury

# Cross-topic analysis across all debriefs
cargo run --bin async_researcher data
```

## How It Works

### Research Pipeline

1. **Task Identification**
   - Analyzes debrief content using Gemini API
   - Identifies open questions, topics needing updates, and cross-topic opportunities
   - Assigns priority scores (1-10) to each task

2. **Task Filtering**
   - Only high-priority tasks (priority ≥ 6) are researched
   - Prevents overwhelming the debrief with low-value information

3. **Research Execution**
   - Each task is researched using the Gemini API
   - Results include findings, confidence score, and sources
   - Only high-confidence results (≥ 6/10) are included

4. **Insight Integration**
   - Research insights are formatted as markdown
   - Appended to debrief under "🔍 Research Insights" section
   - Includes context, findings, sources, and confidence scores

### Research Task Types

```rust
pub enum ResearchTaskType {
    GapFilling,      // Answers open questions
    NoveltyCheck,    // Checks for updates
    CrossPollination, // Finds connections
}
```

### Data Structures

```rust
pub struct ResearchTask {
    pub task_type: ResearchTaskType,
    pub query: String,        // Specific search query
    pub context: String,      // Why this research matters
    pub priority: u8,         // 1-10, higher = more important
}

pub struct ResearchResult {
    pub task: ResearchTask,
    pub findings: String,     // Research findings
    pub confidence: u8,       // 1-10, how confident
    pub sources: Vec<String>, // Reference sources
}
```

## Example Output

When research insights are added to a debrief, they appear like this:

```markdown
---

## 🔍 Research Insights

*The following insights were automatically researched based on open questions and topics in your conversations.*

### 💡 How to fix Rust async runtime error with tokio?

**Context:** User encountered runtime error when spawning async tasks

The error occurs when trying to spawn tasks outside of a tokio runtime context. 
Solution: Wrap your main function with #[tokio::main] or create a runtime manually 
using Runtime::new(). For library code, consider using tokio::spawn only within 
async functions that are already running in a tokio context.

**Sources:**
- Tokio documentation: Runtime creation
- Common async patterns in Rust

*Confidence: 9/10 | Priority: 8/10*

### 🔗 Rust vector database clients with async support

**Context:** Cross-topic connection between vector databases and Rust async patterns

Top options include: qdrant-client (native async), pinecone-sdk (async via reqwest), 
and weaviate-client. The qdrant-client provides the most idiomatic Rust async API 
with full tokio integration.

**Sources:**
- Qdrant Rust client documentation
- Comparison of Rust vector DB clients

*Confidence: 8/10 | Priority: 7/10*
```

## Configuration

### Environment Variables

```bash
GEMINI_API_KEY=your_api_key_here
```

### Customization

You can adjust thresholds in `researcher.rs`:

```rust
// Minimum priority for research execution
let high_priority_tasks: Vec<_> = tasks.into_iter()
    .filter(|t| t.priority >= 6)  // Adjust this threshold
    .collect();

// Minimum confidence for including results
if result.confidence >= 6 {  // Adjust this threshold
    results.push(result);
}
```

## Architecture

### New Files

- **`src/researcher.rs`** - Core research logic and data structures
- **`src/async_researcher.rs`** - Standalone binary for async research
- **`ASYNC_RESEARCH.md`** - This documentation

### Modified Files

- **`src/lib.rs`** - Exports researcher module
- **`src/main.rs`** - Added `--research` flag
- **`Cargo.toml`** - Added async_researcher binary

### Integration Points

```
┌─────────────────┐
│   main.rs       │
│  (debrief gen)  │
└────────┬────────┘
         │
         ├─ Optional: --research flag
         │
         ▼
┌─────────────────┐
│  researcher.rs  │
│ ┌─────────────┐ │
│ │ Identify    │ │
│ │ Tasks       │ │
│ └──────┬──────┘ │
│        │        │
│ ┌──────▼──────┐ │
│ │ Perform     │ │
│ │ Research    │ │
│ └──────┬──────┘ │
│        │        │
│ ┌──────▼──────┐ │
│ │ Format &    │ │
│ │ Append      │ │
│ └─────────────┘ │
└─────────────────┘
```

## Future Enhancements

### Planned Features

1. **Real Web Search Integration**
   - Currently uses Gemini's knowledge base
   - Future: Integrate with Google Search API, Brave Search, or similar
   - Would provide real-time, up-to-date information

2. **Scheduled Background Processing**
   - Run research as a cron job or background service
   - Automatically update debriefs overnight
   - Email/notify when new insights are found

3. **Research History Tracking**
   - Track which queries have been researched
   - Avoid duplicate research
   - Re-research after time threshold (e.g., monthly for novelty checks)

4. **Confidence Calibration**
   - Learn from user feedback on research quality
   - Adjust confidence thresholds based on accuracy
   - Improve task prioritization over time

5. **Multi-Source Research**
   - Combine results from multiple sources
   - Cross-reference findings
   - Provide more comprehensive insights

6. **Interactive Research Mode**
   - Allow users to approve/reject research tasks before execution
   - Provide feedback on research quality
   - Customize research parameters per topic

## Testing

Run the unit tests:

```bash
cargo test --lib researcher
```

Test on a real debrief:

```bash
# Generate a debrief with research
cargo run --bin read_files data/hamstring_injury --research

# Or run research separately
cargo run --bin async_researcher data hamstring_injury
```

## Performance Considerations

- Each research task makes 1-2 API calls to Gemini
- High-priority filtering reduces API usage
- Cross-pollination analysis scales with number of topics
- Consider rate limiting for large-scale deployments

## Troubleshooting

### Research insights not appearing

1. Check that DEBRIEF.md doesn't already have a "Research Insights" section
2. Verify high-priority tasks were identified (check console output)
3. Ensure research results met confidence threshold (≥ 6/10)

### No research tasks identified

- Debrief may not contain clear open questions or topics
- Try adding more detailed conversation transcripts
- Check that debrief has substantive technical content

### API errors

- Verify GEMINI_API_KEY is set correctly
- Check API quota and rate limits
- Ensure network connectivity

## Contributing

To extend the research capabilities:

1. Add new `ResearchTaskType` variants
2. Implement custom research logic in `perform_research()`
3. Update prompt templates for better task identification
4. Add integration with external APIs (web search, documentation, etc.)

## License

Same as parent project.
