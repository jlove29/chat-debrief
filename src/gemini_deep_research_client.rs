//! Deep Research module using Gemini's Deep Research agent
//!
//! This module provides functionality to perform deep research using the Gemini API's
//! deep research agent. It handles the asynchronous polling workflow required for
//! long-running research tasks.

use genai_rs::{Client, DeepResearchConfig, GenaiError, InteractionStatus, ThinkingSummaries};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Maximum time to wait for research to complete when polling
/// Set to 20 minutes to accommodate batched research with multiple complex topics
const MAX_POLL_DURATION: Duration = Duration::from_secs(1200);

/// Initial delay between polls (will increase with exponential backoff)
const INITIAL_POLL_DELAY: Duration = Duration::from_secs(2);

/// Maximum delay between polls
const MAX_POLL_DELAY: Duration = Duration::from_secs(10);

/// The Deep Research agent identifier
const AGENT_NAME: &str = "deep-research-pro-preview-12-2025";

/// Error type for deep research operations
#[derive(Debug)]
pub enum DeepResearchError {
    /// Polling timed out before completion
    Timeout { interaction_id: String },
    /// The interaction failed or was cancelled
    Failed { interaction_id: String },
    /// An API error occurred
    Api(GenaiError),
}

impl From<GenaiError> for DeepResearchError {
    fn from(err: GenaiError) -> Self {
        DeepResearchError::Api(err)
    }
}

impl std::fmt::Display for DeepResearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeepResearchError::Timeout { interaction_id } => {
                write!(f, "Research timed out (interaction: {})", interaction_id)
            }
            DeepResearchError::Failed { interaction_id } => {
                write!(f, "Research failed (interaction: {})", interaction_id)
            }
            DeepResearchError::Api(e) => write!(f, "API error: {}", e),
        }
    }
}

impl std::error::Error for DeepResearchError {}

/// Result of a deep research operation
#[derive(Debug)]
pub struct DeepResearchResult {
    pub findings: String,
    pub interaction_id: String,
}

/// Performs deep research using the Gemini Deep Research agent
///
/// This function initiates a research task and polls for completion using exponential backoff.
/// Research typically takes 30-120 seconds depending on query complexity.
///
/// # Arguments
/// * `client` - The Gemini API client
/// * `query` - The research query to investigate
///
/// # Returns
/// * `Ok(DeepResearchResult)` - The research findings and interaction ID
/// * `Err(DeepResearchError)` - If the research fails, times out, or encounters an API error
pub async fn perform_deep_research(
    client: &Client,
    query: &str,
) -> Result<DeepResearchResult, DeepResearchError> {
    println!("Starting deep research: {}", query);
    println!("(This may take 30-120 seconds...)");

    // Start the research in background mode with agent configuration
    let result = client
        .interaction()
        .with_agent(AGENT_NAME)
        .with_text(query)
        .with_agent_config(
            DeepResearchConfig::new().with_thinking_summaries(ThinkingSummaries::Auto),
        )
        .with_background(true) // Required for agent interactions
        .with_store_enabled() // Required to retrieve results by interaction ID
        .create()
        .await?;

    let interaction_id = result
        .id
        .as_ref()
        .ok_or_else(|| DeepResearchError::Failed {
            interaction_id: "unknown".to_string(),
        })?
        .clone();

    println!("Research initiated (ID: {})", interaction_id);

    // If already completed (fast response), return immediately
    if result.status == InteractionStatus::Completed {
        println!("Research completed immediately");
        let findings = result.text().unwrap_or_default().to_string();
        return Ok(DeepResearchResult {
            findings,
            interaction_id,
        });
    }

    // Handle unusual initial statuses
    if result.status == InteractionStatus::RequiresAction {
        eprintln!("Research requires action before continuing");
        return Err(DeepResearchError::Failed { interaction_id });
    }

    // Poll for completion
    println!("Polling for completion...");
    let final_response = poll_for_completion(client, &interaction_id).await?;

    let findings = final_response.text().unwrap_or_default().to_string();
    Ok(DeepResearchResult {
        findings,
        interaction_id,
    })
}

/// Cancels a pending interaction to avoid unnecessary charges
async fn cancel_interaction(
    client: &Client,
    interaction_id: &str,
) -> Result<(), DeepResearchError> {
    client.cancel_interaction(interaction_id).await?;
    Ok(())
}

/// Polls for interaction completion with exponential backoff
async fn poll_for_completion(
    client: &Client,
    interaction_id: &str,
) -> Result<genai_rs::InteractionResponse, DeepResearchError> {
    let start = Instant::now();
    let mut delay = INITIAL_POLL_DELAY;
    let mut poll_count = 0;

    loop {
        // Check if we've exceeded the maximum wait time
        if start.elapsed() > MAX_POLL_DURATION {
            // Cancel the pending research to avoid unnecessary charges
            eprintln!("  Timeout reached, cancelling pending research...");
            if let Err(e) = cancel_interaction(client, interaction_id).await {
                eprintln!("  Warning: Failed to cancel interaction: {:?}", e);
            } else {
                eprintln!("  Research cancelled successfully");
            }
            return Err(DeepResearchError::Timeout {
                interaction_id: interaction_id.to_string(),
            });
        }

        // Wait before polling (skip on first iteration)
        if poll_count > 0 {
            sleep(delay).await;
            // Exponential backoff up to maximum
            delay = (delay * 2).min(MAX_POLL_DELAY);
        }
        poll_count += 1;

        // Query the interaction status
        let response = client.get_interaction(interaction_id).await?;

        println!(
            "  Poll #{}: status={:?} (elapsed: {:.1}s)",
            poll_count,
            response.status,
            start.elapsed().as_secs_f64()
        );

        // Check the status
        match response.status {
            InteractionStatus::Completed => return Ok(response),
            InteractionStatus::Failed | InteractionStatus::Cancelled => {
                return Err(DeepResearchError::Failed {
                    interaction_id: interaction_id.to_string(),
                });
            }
            InteractionStatus::InProgress => {
                // Continue polling
            }
            InteractionStatus::RequiresAction => {
                eprintln!("  Note: Interaction requires action (unusual for deep research)");
            }
            _ => {
                // Continue polling on unknown statuses for forward compatibility
                eprintln!("  Unhandled status {:?}, continuing to poll...", response.status);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let timeout_err = DeepResearchError::Timeout {
            interaction_id: "test-123".to_string(),
        };
        assert!(timeout_err.to_string().contains("test-123"));
        assert!(timeout_err.to_string().contains("timed out"));

        let failed_err = DeepResearchError::Failed {
            interaction_id: "test-456".to_string(),
        };
        assert!(failed_err.to_string().contains("test-456"));
        assert!(failed_err.to_string().contains("failed"));
    }
}
