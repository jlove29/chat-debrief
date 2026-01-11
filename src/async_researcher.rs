use std::env;
use std::path::Path;
use read_files::researcher;

/// Async researcher binary - runs background research on debriefs
/// 
/// Usage:
///   cargo run --bin async_researcher <data_directory> [topic_name]
///   
/// If topic_name is provided, researches that specific topic.
/// If omitted, performs cross-pollination analysis across all topics.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <data_directory> [topic_name]", args[0]);
        eprintln!("\nExamples:");
        eprintln!("  {} ./data                    # Cross-topic analysis", args[0]);
        eprintln!("  {} ./data hamstring_injury   # Research specific topic", args[0]);
        std::process::exit(1);
    }

    let data_dir = Path::new(&args[1]);
    
    if !data_dir.exists() || !data_dir.is_dir() {
        eprintln!("Error: '{}' is not a valid directory", args[1]);
        std::process::exit(1);
    }

    if args.len() >= 3 {
        // Research a specific topic
        let topic_name = &args[2];
        let topic_path = data_dir.join(topic_name);
        
        if !topic_path.exists() || !topic_path.is_dir() {
            eprintln!("Error: Topic directory '{}' not found", topic_name);
            std::process::exit(1);
        }
        
        let debrief_path = topic_path.join("DEBRIEF.md");
        
        if !debrief_path.exists() {
            eprintln!("Error: DEBRIEF.md not found in topic '{}'", topic_name);
            std::process::exit(1);
        }
        
        println!("🔍 Starting async research for topic: {}", topic_name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        match researcher::research_and_enhance_debrief(&debrief_path, topic_name).await {
            Ok(_) => {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("✅ Research complete!");
            }
            Err(e) => {
                eprintln!("❌ Error during research: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Perform cross-pollination analysis
        println!("🔗 Analyzing cross-topic connections...");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        match researcher::analyze_cross_topic_connections(data_dir).await {
            Ok(tasks) => {
                if tasks.is_empty() {
                    println!("No cross-pollination opportunities found.");
                } else {
                    println!("\n📊 Found {} cross-pollination opportunities:\n", tasks.len());
                    
                    for (i, task) in tasks.iter().enumerate() {
                        println!("{}. {} (Priority: {}/10)", i + 1, task.query, task.priority);
                        println!("   Context: {}\n", task.context);
                    }
                    
                    // Optionally perform research on high-priority tasks
                    let high_priority: Vec<_> = tasks.into_iter()
                        .filter(|t| t.priority >= 7)
                        .collect();
                    
                    if !high_priority.is_empty() {
                        println!("\n🔬 Researching {} high-priority connections...\n", high_priority.len());
                        
                        for task in high_priority {
                            match researcher::perform_research(&task).await {
                                Ok(result) => {
                                    println!("✓ {}", task.query);
                                    println!("  Confidence: {}/10", result.confidence);
                                    if !result.sources.is_empty() {
                                        println!("  Sources: {}", result.sources.join(", "));
                                    }
                                    println!();
                                }
                                Err(e) => {
                                    eprintln!("✗ Error researching '{}': {}", task.query, e);
                                }
                            }
                        }
                    }
                }
                
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("✅ Cross-topic analysis complete!");
            }
            Err(e) => {
                eprintln!("❌ Error during analysis: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
