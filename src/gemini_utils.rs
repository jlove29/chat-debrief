use genai_rs::Client;
use serde_json::Value;

/// The Gemini model to use for API calls
pub const MODEL_NAME: &str = "gemini-3-flash-preview";

/// Environment variable name for the Gemini API key
const API_KEY_ENV_VAR: &str = "GEMINI_API_KEY";

/// Formats a list of file contents for inclusion in prompts
/// Each file is labeled as "File N:" followed by its content
pub fn format_files(files: &[String]) -> String {
    files.iter()
        .enumerate()
        .map(|(i, content)| format!("File {}:\n{}\n", i + 1, content))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Creates a Gemini client using the API key from environment variables
pub fn create_client() -> Client {
    let api_key = std::env::var(API_KEY_ENV_VAR)
        .expect("GEMINI_API_KEY environment variable must be set");
    Client::new(api_key)
}

/// Calls the Gemini API with a prompt and JSON schema for structured output
/// Returns the raw JSON response text
pub async fn call_gemini_with_schema(
    model: &str,
    prompt: &str,
    schema: Value,
) -> Result<String, genai_rs::GenaiError> {
    let client = create_client();
    
    let response = client
        .interaction()
        .with_model(model)
        .with_text(prompt)
        .with_response_format(schema)
        .create()
        .await?;
    
    Ok(response.as_text().unwrap_or_default().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_files_empty() {
        let files: Vec<String> = vec![];
        let result = format_files(&files);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_files_single() {
        let files = vec!["Content of file 1".to_string()];
        let result = format_files(&files);
        assert_eq!(result, "File 1:\nContent of file 1\n");
    }

    #[test]
    fn test_format_files_multiple() {
        let files = vec![
            "Content of file 1".to_string(),
            "Content of file 2".to_string(),
        ];
        let result = format_files(&files);
        let expected = "File 1:\nContent of file 1\n\nFile 2:\nContent of file 2\n";
        assert_eq!(result, expected);
    }
}
