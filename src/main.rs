use std::env;
use std::io;
use std::path::Path;

use read_files::processor::{read_directory_files, process_files, write_debrief, mark_files_as_read};
use read_files::researcher;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Check if directory path is provided
    if args.len() < 2 {
        eprintln!("Usage: {} <directory_path> [--research]", args[0]);
        eprintln!("\nOptions:");
        eprintln!("  --research    Run async research after processing debrief");
        std::process::exit(1);
    }

    let dir_path = &args[1];
    let run_research = args.len() > 2 && args[2] == "--research";
    let path = Path::new(dir_path);

    if !path.exists() {
        eprintln!("Error: Directory '{}' does not exist.", dir_path);
        std::process::exit(1);
    }

    if !path.is_dir() {
        eprintln!("Error: '{}' is not a directory.", dir_path);
        std::process::exit(1);
    }

    println!("Reading files from directory: {}", dir_path);

    // Read all files from the directory
    let (debrief_contents, other_contents, unread_file_paths) = read_directory_files(path)?;

    // If there are no new files to process, exit early
    if other_contents.is_empty() {
        println!("No new files to process. All files have already been read.");
        return Ok(());
    }

    // Pass contents to processing function
    let response = match process_files(debrief_contents, other_contents).await {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Error processing files: {}", e);
            std::process::exit(1);
        }
    };

    // Write the response to DEBRIEF.md
    write_debrief(path, &response)?;

    // Mark the processed files as read
    mark_files_as_read(unread_file_paths)?;

    // Optionally run async research
    if run_research {
        println!("\n🔍 Starting async research...");
        let debrief_path = path.join("DEBRIEF.md");
        let topic_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        match researcher::research_and_enhance_debrief(&debrief_path, topic_name).await {
            Ok(_) => println!("✅ Research complete!"),
            Err(e) => eprintln!("⚠️  Research error: {}", e),
        }
    }

    Ok(())
}

