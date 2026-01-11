use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::gemini_utils::{self, MODEL_NAME};

const EVALUATION_CRITERIA: &str = "Evaluate this DEBRIEF on the following criteria:
1. Does it accurately summarize the key information from the input files?
2. Does it focus on the user's needs, progress, and actions (not Gemini's recommendations)?
3. Is it well-organized and easy to understand?
4. Does it capture important details without being overly verbose?

Provide:
- A score from 1-10 (10 being excellent)
- Reasoning for your score
- A list of specific issues (empty if none)";

/// Response from the autorater
#[derive(Debug, Serialize, Deserialize)]
pub struct AutoraterResponse {
    pub score: i32,
    pub reasoning: String,
    pub issues: Vec<String>,
}

/// Builds the evaluation prompt for the autorater
fn build_evaluation_prompt(
    input_files: &[String],
    debrief_content: &str,
    context: &str,
) -> String {
    let mut prompt = String::new();
    
    prompt.push_str("You are evaluating the quality of a DEBRIEF summary generated from conversation files.\n\n");
    prompt.push_str(&format!("Context: {}\n\n", context));
    
    prompt.push_str("Input files:\n");
    prompt.push_str(&gemini_utils::format_files(input_files));
    prompt.push_str("\n");
    
    prompt.push_str("Generated DEBRIEF:\n");
    prompt.push_str(debrief_content);
    prompt.push_str("\n\n");
    
    prompt.push_str(EVALUATION_CRITERIA);
    
    prompt
}

/// Uses Gemini to evaluate the quality of a DEBRIEF.md
/// Returns a score from 1-10 and reasoning
pub async fn evaluate_debrief(
    input_files: &[String],
    debrief_content: &str,
    context: &str,
) -> Result<AutoraterResponse, genai_rs::GenaiError> {
    let prompt = build_evaluation_prompt(input_files, debrief_content, context);
    
    let schema = json!({
        "type": "object",
        "properties": {
            "score": {"type": "integer", "minimum": 1, "maximum": 10},
            "reasoning": {"type": "string"},
            "issues": {
                "type": "array",
                "items": {"type": "string"}
            }
        },
        "required": ["score", "reasoning", "issues"]
    });
    
    let json_text = gemini_utils::call_gemini_with_schema(MODEL_NAME, &prompt, schema).await?;
    let autorater_response: AutoraterResponse = serde_json::from_str(&json_text)
        .expect("Failed to parse autorater response");
    
    Ok(autorater_response)
}
