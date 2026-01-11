# Quick Start: Async Research

## 🚀 Getting Started

The async research feature is now fully integrated into chat-debrief! Here's how to use it:

### Method 1: Integrated Research (Recommended)

Process conversations AND run research in one command:

```bash
cargo run --bin read_files data/your_topic --research
```

This will:
1. ✅ Generate/update the debrief from conversation files
2. 🔍 Identify research opportunities
3. 🔬 Perform high-priority research
4. 📝 Append insights to the debrief

### Method 2: Standalone Research

Run research on an existing debrief:

```bash
# Research a specific topic
cargo run --bin async_researcher data your_topic

# Cross-topic analysis (requires 2+ topics with debriefs)
cargo run --bin async_researcher data
```

## 📊 What Gets Researched?

The system automatically identifies:

- **💡 Gap Filling**: Open questions, errors, stuck points
- **🆕 Novelty Checks**: Updates to libraries, frameworks, papers
- **🔗 Cross-Pollination**: Connections between different topics

Only **high-priority** (≥6/10) and **high-confidence** (≥6/10) research is included.

## 📝 Example Output

Research insights are appended to your debrief like this:

```markdown
---

## 🔍 Research Insights

*The following insights were automatically researched...*

### 💡 How to fix Rust async runtime error?

**Context:** User encountered runtime error when spawning tasks

[Detailed findings here...]

*Confidence: 9/10 | Priority: 8/10*
```

## ✅ Tested Features

- ✅ Integrated research with `--research` flag
- ✅ Standalone topic research
- ✅ Cross-topic analysis (needs 2+ debriefs)
- ✅ All unit tests passing
- ✅ Real-world testing on hamstring_injury and hard_drive topics

## 🎯 Real Test Results

**Test 1: hamstring_injury topic**
- Identified: 6 research tasks
- Researched: 5 high-priority tasks
- Added: 5 research insights
- Topics included:
  - Physiological significance of pain types in recovery
  - Distal vs proximal strain rehabilitation protocols
  - Effect of cadence on eccentric load
  - Topical vs oral anti-inflammatories
  - Readiness-to-run criteria

**Test 2: hard_drive topic**
- Identified: 5 research tasks
- Researched: 4 high-priority tasks
- Added: 4 research insights
- Topics included:
  - macOS mount flags for dirty journal bypass
  - Force mount HFS+ read-only on Linux
  - WSL2 HFS+ support
  - macOS mount error 22 troubleshooting

## 🔧 Configuration

Adjust thresholds in `src/researcher.rs`:

```rust
// Minimum priority for research (line ~261)
.filter(|t| t.priority >= 6)

// Minimum confidence for results (line ~276)
if result.confidence >= 6 {
```

## 📚 Full Documentation

See `ASYNC_RESEARCH.md` for complete documentation including:
- Architecture details
- Future enhancements
- Troubleshooting
- Contributing guidelines

## 🎉 Success!

The async research feature is fully implemented and tested. It transforms chat-debrief from a reactive summarizer into a proactive research assistant!
