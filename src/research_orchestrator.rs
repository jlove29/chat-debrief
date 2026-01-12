use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::gemini_utils::{self, MODEL_NAME};
use crate::gemini_deep_research_client;
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

/// Performs batched deep research for multiple tasks using a single Deep Research API call
///
/// This function combines all research tasks into a single comprehensive query,
/// which is more efficient than making multiple sequential API calls.
pub async fn perform_batch_research(
    tasks: &[ResearchTask],
) -> Result<ResearchResult, Box<dyn std::error::Error>> {
    if tasks.is_empty() {
        return Err("No tasks to research".into());
    }
    
    // Create the Gemini client
    let client = gemini_utils::create_client();
    
    // Build a comprehensive research query that includes all tasks
    let mut query = String::from("Please conduct comprehensive research on the following topics:\n\n");
    
    for (i, task) in tasks.iter().enumerate() {
        query.push_str(&format!(
            "{}. **{}**\n   Query: {}\n   Context: {}\n\n",
            i + 1,
            match task.task_type {
                ResearchTaskType::GapFilling => "Gap Filling",
                ResearchTaskType::NoveltyCheck => "Novelty Check",
                ResearchTaskType::CrossPollination => "Cross-Pollination",
            },
            task.query,
            task.context
        ));
    }
    
    query.push_str(
        r#"For each topic, provide:
1. Specific, actionable information and findings
2. Relevant details, solutions, or insights
3. Sources and citations where applicable
4. Any caveats or limitations

Organize your response with clear sections for each research topic.
Be thorough and evidence-based."#
    );

    println!("Starting batched deep research for {} tasks...", tasks.len());
    
    // Perform deep research using the Deep Research agent
    let result = gemini_deep_research_client::perform_deep_research(&client, &query).await?;
    
    // Extract sources from the findings
    let sources = extract_sources(&result.findings);
    
    // Estimate confidence based on the depth of the response
    let confidence = estimate_confidence(&result.findings);
    
    // Create a synthetic task that represents all the batched tasks
    let batch_task = ResearchTask {
        task_type: ResearchTaskType::GapFilling,
        query: format!("Batch research: {} tasks", tasks.len()),
        context: format!("Combined research for: {}", 
            tasks.iter()
                .map(|t| t.query.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        ),
        priority: tasks.iter().map(|t| t.priority).max().unwrap_or(5),
    };
    
    Ok(ResearchResult {
        task: batch_task,
        findings: result.findings,
        confidence,
        sources,
    })
}

/// Performs deep research for a single task (kept for backwards compatibility)
///
/// Note: For multiple tasks, use perform_batch_research instead for better efficiency.
pub async fn perform_research(
    task: &ResearchTask,
) -> Result<ResearchResult, Box<dyn std::error::Error>> {
    perform_batch_research(&[task.clone()]).await
}

/// Extracts source references from research findings
/// This is a simple heuristic that looks for URLs and citation patterns
fn extract_sources(findings: &str) -> Vec<String> {
    let mut sources = Vec::new();
    
    // Look for URLs
    for word in findings.split_whitespace() {
        if word.starts_with("http://") || word.starts_with("https://") {
            sources.push(word.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != ':').to_string());
        }
    }
    
    // If no URLs found, indicate that sources are embedded in the research
    if sources.is_empty() {
        sources.push("Deep Research synthesis (multiple sources)".to_string());
    }
    
    sources
}

/// Estimates confidence based on the depth and quality of findings
fn estimate_confidence(findings: &str) -> u8 {
    // Simple heuristic: longer, more detailed responses indicate higher confidence
    let length = findings.len();
    
    if length > 2000 {
        9 // Very comprehensive research
    } else if length > 1000 {
        8 // Detailed research
    } else if length > 500 {
        7 // Good research
    } else if length > 200 {
        6 // Basic research
    } else {
        5 // Minimal research
    }
}

/// Formats research results as markdown to append to debrief
pub fn format_research_insights(results: &[ResearchResult]) -> String {
    if results.is_empty() {
        return String::new();
    }
    
    let mut output = String::from("\n\n---\n\n## Research Insights\n\n");
    output.push_str("*The following insights were automatically researched based on open questions and topics in your conversations.*\n\n");
    
    for result in results {
        let task_emoji = match result.task.task_type {
            ResearchTaskType::GapFilling => "**Gap Filling**",
            ResearchTaskType::NoveltyCheck => "**Novelty Check**",
            ResearchTaskType::CrossPollination => "**Cross-Pollination**",
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

/// Runs async research on a debrief and saves insights to RESEARCH.md
pub async fn research_and_enhance_debrief(
    debrief_path: &Path,
    topic_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the current debrief
    let debrief_content = fs::read_to_string(debrief_path)?;
    
    // Determine the research file path (same directory as debrief)
    let research_path = debrief_path.parent()
        .ok_or("Invalid debrief path")?
        .join("RESEARCH.md");
    
    // Check if research has already been performed (avoid duplicates)
    if research_path.exists() {
        let existing_research = fs::read_to_string(&research_path)?;
        if existing_research.contains("## Research Insights") {
            println!("Research insights already present in RESEARCH.md. Skipping...");
            return Ok(());
        }
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
    
    // Batch all research tasks into a single Deep Research call
    let result = perform_batch_research(&high_priority_tasks).await?;
    let results = vec![result];
    
    if results.is_empty() {
        println!("No high-confidence research results to add.");
        return Ok(());
    }
    
    // Format research insights
    let insights = format_research_insights(&results);
    
    // Write to RESEARCH.md file
    fs::write(&research_path, insights)?;
    println!("✓ Added {} research insights to RESEARCH.md", results.len());
    
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
        assert!(output.contains("## Research Insights"));
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
