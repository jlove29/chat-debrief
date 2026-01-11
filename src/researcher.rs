use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::gemini_utils::{self, MODEL_NAME};
use std::path::Path;
use std::fs;

/// Represents a research task identified from the debrief
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTask {
    pub task_type: ResearchTaskType,
    pub query: String,
    pub context: String,
    pub priority: u8, // 1-10, higher is more important
}

/// Types of research tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResearchTaskType {
    GapFilling,      // Proactive gap filling for open questions
    NoveltyCheck,    // Check for updates on topics
    CrossPollination, // Find connections between topics
}

/// Result of a research task
#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchResult {
    pub task: ResearchTask,
    pub findings: String,
    pub confidence: u8, // 1-10, how confident we are in the findings
    pub sources: Vec<String>,
}

/// Collection of research insights to append to debrief
#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchInsights {
    pub items: Vec<ResearchInsightItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchInsightItem {
    pub header: String,
    pub text: String,
}

/// Identifies open loops and unresolved questions in a debrief
pub async fn identify_research_tasks(
    debrief_content: &str,
    topic_name: &str,
) -> Result<Vec<ResearchTask>, Box<dyn std::error::Error>> {
    let prompt = format!(
        r#"You are analyzing a debrief of conversations to identify research opportunities.

Topic: {}

Debrief Content:
{}

Your task is to identify:
1. **Open Questions/Gaps**: Unresolved questions, errors the user encountered, or topics they were exploring but got stuck on
2. **Topics for Updates**: Specific libraries, frameworks, papers, or technologies mentioned that might have updates
3. **Cross-Topic Connections**: Themes or technologies that could benefit from research connecting different areas

For each research opportunity, provide:
- task_type: "GapFilling", "NoveltyCheck", or "CrossPollination"
- query: A specific, actionable search query
- context: Brief context about why this research would be valuable
- priority: 1-10 (higher = more important/urgent)

Only suggest high-value research tasks. Aim for 3-7 tasks maximum."#,
        topic_name, debrief_content
    );

    let schema = json!({
        "type": "object",
        "properties": {
            "tasks": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "task_type": {
                            "type": "string",
                            "enum": ["GapFilling", "NoveltyCheck", "CrossPollination"]
                        },
                        "query": {"type": "string"},
                        "context": {"type": "string"},
                        "priority": {"type": "integer", "minimum": 1, "maximum": 10}
                    },
                    "required": ["task_type", "query", "context", "priority"]
                }
            }
        },
        "required": ["tasks"]
    });

    println!("Identifying research tasks for topic: {}", topic_name);
    let json_text = gemini_utils::call_gemini_with_schema(MODEL_NAME, &prompt, schema).await?;
    
    #[derive(Deserialize)]
    struct TasksResponse {
        tasks: Vec<TaskData>,
    }
    
    #[derive(Deserialize)]
    struct TaskData {
        task_type: String,
        query: String,
        context: String,
        priority: u8,
    }
    
    let response: TasksResponse = serde_json::from_str(&json_text)?;
    
    let tasks = response.tasks.into_iter().map(|t| {
        let task_type = match t.task_type.as_str() {
            "GapFilling" => ResearchTaskType::GapFilling,
            "NoveltyCheck" => ResearchTaskType::NoveltyCheck,
            "CrossPollination" => ResearchTaskType::CrossPollination,
            _ => ResearchTaskType::GapFilling,
        };
        
        ResearchTask {
            task_type,
            query: t.query,
            context: t.context,
            priority: t.priority,
        }
    }).collect();
    
    Ok(tasks)
}

/// Performs web research for a given task (simulated for now)
/// In a real implementation, this would use a web search API
pub async fn perform_research(
    task: &ResearchTask,
) -> Result<ResearchResult, Box<dyn std::error::Error>> {
    // For now, we'll use Gemini to simulate research
    // In production, this would integrate with web search APIs
    let prompt = format!(
        r#"You are a research assistant helping to answer questions and find information.

Research Query: {}
Context: {}
Task Type: {:?}

Based on your knowledge, provide:
1. Key findings that answer the query or provide relevant information
2. Specific details, solutions, or insights
3. Any caveats or limitations

Be specific and actionable. If you don't have current information, acknowledge that and provide the best available guidance."#,
        task.query, task.context, task.task_type
    );

    let schema = json!({
        "type": "object",
        "properties": {
            "findings": {"type": "string"},
            "confidence": {"type": "integer", "minimum": 1, "maximum": 10},
            "sources": {
                "type": "array",
                "items": {"type": "string"}
            }
        },
        "required": ["findings", "confidence", "sources"]
    });

    println!("Researching: {}", task.query);
    let json_text = gemini_utils::call_gemini_with_schema(MODEL_NAME, &prompt, schema).await?;
    
    #[derive(Deserialize)]
    struct ResearchData {
        findings: String,
        confidence: u8,
        sources: Vec<String>,
    }
    
    let data: ResearchData = serde_json::from_str(&json_text)?;
    
    Ok(ResearchResult {
        task: task.clone(),
        findings: data.findings,
        confidence: data.confidence,
        sources: data.sources,
    })
}

/// Formats research results as markdown to append to debrief
pub fn format_research_insights(results: &[ResearchResult]) -> String {
    if results.is_empty() {
        return String::new();
    }
    
    let mut output = String::from("\n\n---\n\n## 🔍 Research Insights\n\n");
    output.push_str("*The following insights were automatically researched based on open questions and topics in your conversations.*\n\n");
    
    for result in results {
        let task_emoji = match result.task.task_type {
            ResearchTaskType::GapFilling => "💡",
            ResearchTaskType::NoveltyCheck => "🆕",
            ResearchTaskType::CrossPollination => "🔗",
        };
        
        output.push_str(&format!("### {} {}\n\n", task_emoji, result.task.query));
        output.push_str(&format!("**Context:** {}\n\n", result.task.context));
        output.push_str(&format!("{}\n\n", result.findings));
        
        if !result.sources.is_empty() {
            output.push_str("**Sources:**\n");
            for source in &result.sources {
                output.push_str(&format!("- {}\n", source));
            }
            output.push_str("\n");
        }
        
        output.push_str(&format!("*Confidence: {}/10 | Priority: {}/10*\n\n", 
            result.confidence, result.task.priority));
    }
    
    output
}

/// Runs async research on a debrief and appends insights
pub async fn research_and_enhance_debrief(
    debrief_path: &Path,
    topic_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the current debrief
    let debrief_content = fs::read_to_string(debrief_path)?;
    
    // Check if research has already been performed (avoid duplicates)
    if debrief_content.contains("## 🔍 Research Insights") {
        println!("Research insights already present in debrief. Skipping...");
        return Ok(());
    }
    
    // Identify research tasks
    let tasks = identify_research_tasks(&debrief_content, topic_name).await?;
    
    if tasks.is_empty() {
        println!("No research tasks identified.");
        return Ok(());
    }
    
    println!("Identified {} research tasks", tasks.len());
    
    // Perform research on high-priority tasks (priority >= 6)
    let high_priority_tasks: Vec<_> = tasks.into_iter()
        .filter(|t| t.priority >= 6)
        .collect();
    
    if high_priority_tasks.is_empty() {
        println!("No high-priority research tasks found.");
        return Ok(());
    }
    
    println!("Performing research on {} high-priority tasks...", high_priority_tasks.len());
    
    let mut results = Vec::new();
    for task in high_priority_tasks {
        match perform_research(&task).await {
            Ok(result) => {
                // Only include high-confidence results (>= 6)
                if result.confidence >= 6 {
                    results.push(result);
                }
            }
            Err(e) => {
                eprintln!("Error researching task '{}': {}", task.query, e);
            }
        }
    }
    
    if results.is_empty() {
        println!("No high-confidence research results to add.");
        return Ok(());
    }
    
    // Format and append research insights
    let insights = format_research_insights(&results);
    let enhanced_debrief = format!("{}{}", debrief_content, insights);
    
    // Write back to file
    fs::write(debrief_path, enhanced_debrief)?;
    println!("✓ Added {} research insights to debrief", results.len());
    
    Ok(())
}

/// Analyzes all debriefs in the data directory for cross-pollination opportunities
pub async fn analyze_cross_topic_connections(
    data_dir: &Path,
) -> Result<Vec<ResearchTask>, Box<dyn std::error::Error>> {
    let mut all_topics = Vec::new();
    
    // Read all topic directories and their debriefs
    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let debrief_path = path.join("DEBRIEF.md");
            if debrief_path.exists() {
                let topic_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let content = fs::read_to_string(&debrief_path)?;
                all_topics.push((topic_name, content));
            }
        }
    }
    
    if all_topics.len() < 2 {
        println!("Need at least 2 topics for cross-pollination analysis");
        return Ok(Vec::new());
    }
    
    // Build a prompt with all topics
    let mut prompt = String::from(
        "You are analyzing multiple conversation topics to find valuable connections.\n\n"
    );
    
    for (name, content) in &all_topics {
        prompt.push_str(&format!("## Topic: {}\n{}\n\n", name, content));
    }
    
    prompt.push_str(
        r#"Identify 3-5 high-value cross-pollination opportunities where:
- Concepts from one topic could inform or solve problems in another
- Technologies discussed separately could be combined
- Similar patterns or challenges appear across topics

For each opportunity, provide a specific research query that would bridge the topics."#
    );
    
    let schema = json!({
        "type": "object",
        "properties": {
            "tasks": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "context": {"type": "string"},
                        "priority": {"type": "integer", "minimum": 1, "maximum": 10}
                    },
                    "required": ["query", "context", "priority"]
                }
            }
        },
        "required": ["tasks"]
    });
    
    println!("Analyzing cross-topic connections across {} topics...", all_topics.len());
    let json_text = gemini_utils::call_gemini_with_schema(MODEL_NAME, &prompt, schema).await?;
    
    #[derive(Deserialize)]
    struct CrossTopicResponse {
        tasks: Vec<CrossTopicTask>,
    }
    
    #[derive(Deserialize)]
    struct CrossTopicTask {
        query: String,
        context: String,
        priority: u8,
    }
    
    let response: CrossTopicResponse = serde_json::from_str(&json_text)?;
    
    let tasks = response.tasks.into_iter().map(|t| {
        ResearchTask {
            task_type: ResearchTaskType::CrossPollination,
            query: t.query,
            context: t.context,
            priority: t.priority,
        }
    }).collect();
    
    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_research_insights_empty() {
        let results: Vec<ResearchResult> = vec![];
        let output = format_research_insights(&results);
        assert_eq!(output, "");
    }

    #[test]
    fn test_format_research_insights_single() {
        let task = ResearchTask {
            task_type: ResearchTaskType::GapFilling,
            query: "How to fix error X?".to_string(),
            context: "User encountered error X".to_string(),
            priority: 8,
        };
        
        let result = ResearchResult {
            task: task.clone(),
            findings: "Solution: Do Y and Z".to_string(),
            confidence: 9,
            sources: vec!["Documentation".to_string()],
        };
        
        let output = format_research_insights(&[result]);
        assert!(output.contains("## 🔍 Research Insights"));
        assert!(output.contains("How to fix error X?"));
        assert!(output.contains("Solution: Do Y and Z"));
        assert!(output.contains("Documentation"));
        assert!(output.contains("Confidence: 9/10"));
    }

    #[test]
    fn test_research_task_type_serialization() {
        let task = ResearchTask {
            task_type: ResearchTaskType::NoveltyCheck,
            query: "Test query".to_string(),
            context: "Test context".to_string(),
            priority: 5,
        };
        
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("NoveltyCheck"));
        
        let deserialized: ResearchTask = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.task_type, ResearchTaskType::NoveltyCheck);
        assert_eq!(deserialized.priority, 5);
    }
}
