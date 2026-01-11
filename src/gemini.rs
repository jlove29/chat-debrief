use genai_rs::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

const MODEL_NAME: &str = "gemini-3-flash-preview";
const API_KEY_ENV_VAR: &str = "GEMINI_API_KEY";

const DISCLAIMER: &str = "IMPORTANT:
- Your goal is to summarize the user's needs, current progress or state, and anything they might have done or tried in the course of the conversation.
- Your goal is NOT to provide a summary of what Gemini has recommended – only include details of Gemini's responses when they help explain what the user did in the context of the conversation.
- The purpose of this debrief is catch to up future Gemini models on what the user needs and has done, so the user can ask follow up questions and Gemini can make informed responses.";

/// Represents a single section in the debrief
#[derive(Debug, Serialize, Deserialize)]
pub struct DebriefItem {
    pub header: String,
    pub text: String,
}

/// Represents the complete debrief structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Debrief {
    pub items: Vec<DebriefItem>,
}

/// Calls the Gemini API with the provided file contents
pub async fn analyze_files(debrief_contents: String, other_contents: Vec<String>) -> Result<String, genai_rs::GenaiError> {
    // Get API key from environment
    let api_key = std::env::var(API_KEY_ENV_VAR)
        .expect("GEMINI_API_KEY environment variable must be set");

    // Initialize the Gemini client
    let client = Client::new(api_key);

    // Construct the prompt based on whether debrief exists
    let has_existing_debrief = !debrief_contents.trim().is_empty();
    let prompt = build_prompt(debrief_contents, other_contents, has_existing_debrief);

    // Define the JSON schema for structured output
    let schema = json!({
        "type": "object",
        "properties": {
            "items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "header": {"type": "string"},
                        "text": {"type": "string"}
                    },
                    "required": ["header", "text"]
                }
            }
        },
        "required": ["items"]
    });

    // Call the Gemini API with structured output
    println!("Calling Gemini API...");
    let response = client
        .interaction()
        .with_model(MODEL_NAME)
        .with_text(&prompt)
        .with_response_format(schema)
        .create()
        .await?;
    
    // Parse the JSON response into our Debrief struct
    let json_text = response.text().unwrap_or_default();
    let debrief: Debrief = serde_json::from_str(json_text)
        .expect("Failed to parse debrief JSON from Gemini response");
    
    // Convert the structured debrief to markdown
    Ok(format_debrief_as_markdown(&debrief))
}

/// Builds the prompt for the Gemini API
fn build_prompt(debrief_contents: String, other_contents: Vec<String>, has_existing_debrief: bool) -> String {
    let mut prompt = String::new();
    
    if has_existing_debrief {
        // Prompt for updating existing debrief
        prompt.push_str("Here's the current debrief of the user's conversations with Gemini:\n\n");
        prompt.push_str(&debrief_contents);
        prompt.push_str("\n\n");
        
        prompt.push_str("Here are the updates to the user's conversations since this debrief was generated. Note that some of them might contain full conversations, where previous steps in the conversation have already been incorporated into the debrief.\n\n");
        prompt.push_str(&format_files(other_contents));
        prompt.push_str("\n\n");
        
        prompt.push_str("Please rewrite sections of the debrief if there is new information which clarifies, contradicts, or contains important additional details. If you have new information that doesn't fit into the existing debrief, you can add a new section.\n\n");
        prompt.push_str("To update an existing section, repeat the exact header of that section and provide the updated text. To add a new section, create a new header and text. Keep sections that don't need updates unchanged by repeating their header and original text.\n\n");
    } else {
        // Prompt for creating new debrief
        prompt.push_str("Here are the user's conversations with Gemini about this topic.\n\n");
        prompt.push_str(&format_files(other_contents));
        prompt.push_str("\n\n");
        
        prompt.push_str("Your job is to provide a debrief on the user's conversations with Gemini on this topic.\n\n");
    }
    
    prompt.push_str(DISCLAIMER);
    
    prompt
}

/// Formats the file contents for inclusion in the prompt
fn format_files(files: Vec<String>) -> String {
    files.iter()
        .enumerate()
        .map(|(i, content)| format!("File {}:\n{}\n", i + 1, content))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Converts a structured Debrief into markdown format
fn format_debrief_as_markdown(debrief: &Debrief) -> String {
    debrief.items.iter()
        .map(|item| format!("### {}\n\n{}\n", item.header, item.text))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_files_empty() {
        let files = vec![];
        let result = format_files(files);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_files_single() {
        let files = vec!["Content of first file".to_string()];
        let result = format_files(files);
        assert_eq!(result, "File 1:\nContent of first file\n");
    }

    #[test]
    fn test_format_files_multiple() {
        let files = vec![
            "First file content".to_string(),
            "Second file content".to_string(),
            "Third file content".to_string(),
        ];
        let result = format_files(files);
        let expected = "File 1:\nFirst file content\n\nFile 2:\nSecond file content\n\nFile 3:\nThird file content\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_debrief_as_markdown_empty() {
        let debrief = Debrief { items: vec![] };
        let result = format_debrief_as_markdown(&debrief);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_debrief_as_markdown_single_item() {
        let debrief = Debrief {
            items: vec![DebriefItem {
                header: "Introduction".to_string(),
                text: "This is the introduction text.".to_string(),
            }],
        };
        let result = format_debrief_as_markdown(&debrief);
        assert_eq!(result, "### Introduction\n\nThis is the introduction text.\n");
    }

    #[test]
    fn test_format_debrief_as_markdown_multiple_items() {
        let debrief = Debrief {
            items: vec![
                DebriefItem {
                    header: "Section 1".to_string(),
                    text: "Content for section 1.".to_string(),
                },
                DebriefItem {
                    header: "Section 2".to_string(),
                    text: "Content for section 2.".to_string(),
                },
            ],
        };
        let result = format_debrief_as_markdown(&debrief);
        let expected = "### Section 1\n\nContent for section 1.\n\n### Section 2\n\nContent for section 2.\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_prompt_new_debrief() {
        let debrief_contents = String::new();
        let other_contents = vec!["Conversation 1".to_string()];
        let result = build_prompt(debrief_contents, other_contents, false);
        
        assert!(result.contains("Here are the user's conversations with Gemini about this topic."));
        assert!(result.contains("File 1:\nConversation 1\n"));
        assert!(result.contains("Your job is to provide a debrief"));
        assert!(result.contains(DISCLAIMER));
        assert!(!result.contains("Here's the current debrief"));
    }

    #[test]
    fn test_build_prompt_existing_debrief() {
        let debrief_contents = "### Existing Section\n\nExisting content.".to_string();
        let other_contents = vec!["New conversation".to_string()];
        let result = build_prompt(debrief_contents.clone(), other_contents, true);
        
        assert!(result.contains("Here's the current debrief"));
        assert!(result.contains(&debrief_contents));
        assert!(result.contains("Here are the updates to the user's conversations"));
        assert!(result.contains("File 1:\nNew conversation\n"));
        assert!(result.contains("Please rewrite sections of the debrief"));
        assert!(result.contains(DISCLAIMER));
        assert!(!result.contains("Your job is to provide a debrief"));
    }

    #[test]
    fn test_build_prompt_structure() {
        let debrief_contents = String::new();
        let other_contents = vec!["Test".to_string()];
        let result = build_prompt(debrief_contents, other_contents, false);
        
        // Verify the disclaimer is at the end
        assert!(result.ends_with(DISCLAIMER));
    }

    #[test]
    fn test_debrief_item_serialization() {
        let item = DebriefItem {
            header: "Test Header".to_string(),
            text: "Test text content.".to_string(),
        };
        
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("Test Header"));
        assert!(json.contains("Test text content."));
    }

    #[test]
    fn test_debrief_deserialization() {
        let json = r#"{"items":[{"header":"Title","text":"Body"}]}"#;
        let debrief: Debrief = serde_json::from_str(json).unwrap();
        
        assert_eq!(debrief.items.len(), 1);
        assert_eq!(debrief.items[0].header, "Title");
        assert_eq!(debrief.items[0].text, "Body");
    }

    #[test]
    fn test_debrief_roundtrip() {
        let original = Debrief {
            items: vec![
                DebriefItem {
                    header: "Header 1".to_string(),
                    text: "Text 1".to_string(),
                },
                DebriefItem {
                    header: "Header 2".to_string(),
                    text: "Text 2".to_string(),
                },
            ],
        };
        
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Debrief = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.items.len(), 2);
        assert_eq!(deserialized.items[0].header, "Header 1");
        assert_eq!(deserialized.items[1].text, "Text 2");
    }
}
