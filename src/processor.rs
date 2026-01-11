use std::fs;
use std::path::Path;
use std::io::{self, Write};

/// Reads all files from a directory, separating DEBRIEF.md from other files.
/// Creates DEBRIEF.md if it doesn't exist.
/// Skips files that have already been processed (marked with _read suffix).
/// Returns (debrief_contents, other_contents, unread_file_paths)
pub fn read_directory_files(path: &Path) -> io::Result<(String, Vec<String>, Vec<std::path::PathBuf>)> {
    let mut other_contents = Vec::new();
    let mut unread_file_paths = Vec::new();
    
    // Read directory contents
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            // Skip DEBRIEF.md as it's handled separately later
            if entry_path.file_name().unwrap() == "DEBRIEF.md" {
                continue;
            }
            
            // Skip files that have already been processed (contain _read in filename)
            let filename = entry_path.file_name().unwrap().to_string_lossy();
            if filename.contains("_read") {
                eprintln!("Skipping already processed file: {}", filename);
                continue;
            }
            
            // Read file contents
            match fs::read_to_string(&entry_path) {
                Ok(contents) => {
                    other_contents.push(contents);
                    unread_file_paths.push(entry_path.clone());
                },
                Err(e) => {
                    eprintln!("Error reading file {:?}: {}", entry_path.file_name().unwrap(), e);
                }
            }
        }
    }

    // Handle DEBRIEF.md separately
    let debrief_path = path.join("DEBRIEF.md");
    let debrief_contents = if debrief_path.exists() {
        match fs::read_to_string(&debrief_path) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Error reading DEBRIEF.md: {}", e);
                String::new()
            }
        }
    } else {
        println!("DEBRIEF.md not found. Creating it...");
        match fs::File::create(&debrief_path) {
            Ok(mut file) => {
                if let Err(e) = writeln!(file, "# Debrief") {
                    eprintln!("Error writing to DEBRIEF.md: {}", e);
                } else {
                    println!("Successfully created DEBRIEF.md");
                }
            },
            Err(e) => eprintln!("Error creating DEBRIEF.md: {}", e),
        }
        String::from("# Debrief\n")
    };

    Ok((debrief_contents, other_contents, unread_file_paths))
}

/// Writes the updated content to DEBRIEF.md in the specified directory
pub fn write_debrief(path: &Path, content: &str) -> io::Result<()> {
    let debrief_path = path.join("DEBRIEF.md");
    fs::write(&debrief_path, content)?;
    println!("Successfully updated DEBRIEF.md");
    Ok(())
}

/// Marks files as read by renaming them with a _read suffix
/// For example: 1.txt -> 1_read.txt
pub fn mark_files_as_read(file_paths: Vec<std::path::PathBuf>) -> io::Result<()> {
    for path in file_paths {
        if let Some(file_stem) = path.file_stem() {
            if let Some(extension) = path.extension() {
                let new_name = format!(
                    "{}_read.{}",
                    file_stem.to_string_lossy(),
                    extension.to_string_lossy()
                );
                let new_path = path.with_file_name(new_name);
                
                match fs::rename(&path, &new_path) {
                    Ok(_) => {
                        println!("Marked as read: {} -> {}", 
                            path.file_name().unwrap().to_string_lossy(),
                            new_path.file_name().unwrap().to_string_lossy());
                    },
                    Err(e) => {
                        eprintln!("Error renaming file {:?}: {}", path, e);
                    }
                }
            }
        }
    }
    Ok(())
}

// Process the file contents by calling the Gemini API
// Returns the Gemini response to be written to DEBRIEF.md
pub async fn process_files(debrief_contents: String, other_contents: Vec<String>) -> Result<String, genai_rs::GenaiError> {
    eprintln!("Debrief contents length: {} chars", debrief_contents.len());
    eprintln!("Number of other files: {}", other_contents.len());

    // Call the Gemini API
    let response = crate::debrief::analyze_files(debrief_contents, other_contents).await?;
    
    // Print the response
    println!("\n=== Gemini Response ===");
    println!("{}", response);
    println!("======================\n");

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // Helper function to create a temporary test directory
    fn create_test_dir(name: &str) -> PathBuf {
        let test_dir = std::env::temp_dir().join(format!("processor_test_{}", name));
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).unwrap();
        }
        fs::create_dir(&test_dir).unwrap();
        test_dir
    }

    // Helper function to clean up test directory
    fn cleanup_test_dir(path: &Path) {
        if path.exists() {
            fs::remove_dir_all(path).ok();
        }
    }

    #[test]
    fn test_read_directory_files_empty_directory() {
        let test_dir = create_test_dir("empty");
        
        let result = read_directory_files(&test_dir);
        assert!(result.is_ok());
        
        let (debrief_contents, other_contents, unread_paths) = result.unwrap();
        assert_eq!(debrief_contents, "# Debrief\n");
        assert_eq!(other_contents.len(), 0);
        assert_eq!(unread_paths.len(), 0);
        
        // Verify DEBRIEF.md was created
        assert!(test_dir.join("DEBRIEF.md").exists());
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_read_directory_files_with_existing_debrief() {
        let test_dir = create_test_dir("existing_debrief");
        
        // Create DEBRIEF.md with custom content
        let debrief_path = test_dir.join("DEBRIEF.md");
        fs::write(&debrief_path, "# Custom Debrief\n\nExisting content.").unwrap();
        
        let result = read_directory_files(&test_dir);
        assert!(result.is_ok());
        
        let (debrief_contents, _, _) = result.unwrap();
        assert_eq!(debrief_contents, "# Custom Debrief\n\nExisting content.");
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_read_directory_files_with_unread_files() {
        let test_dir = create_test_dir("unread_files");
        
        // Create some test files
        fs::write(test_dir.join("file1.txt"), "Content 1").unwrap();
        fs::write(test_dir.join("file2.txt"), "Content 2").unwrap();
        fs::write(test_dir.join("file3.md"), "Content 3").unwrap();
        
        let result = read_directory_files(&test_dir);
        assert!(result.is_ok());
        
        let (_, other_contents, unread_paths) = result.unwrap();
        assert_eq!(other_contents.len(), 3);
        assert_eq!(unread_paths.len(), 3);
        assert!(other_contents.contains(&"Content 1".to_string()));
        assert!(other_contents.contains(&"Content 2".to_string()));
        assert!(other_contents.contains(&"Content 3".to_string()));
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_read_directory_files_skips_read_files() {
        let test_dir = create_test_dir("skip_read");
        
        // Create unread and already-read files
        fs::write(test_dir.join("file1.txt"), "Unread content").unwrap();
        fs::write(test_dir.join("file2_read.txt"), "Already read").unwrap();
        fs::write(test_dir.join("file3_read.md"), "Also read").unwrap();
        
        let result = read_directory_files(&test_dir);
        assert!(result.is_ok());
        
        let (_, other_contents, unread_paths) = result.unwrap();
        assert_eq!(other_contents.len(), 1);
        assert_eq!(unread_paths.len(), 1);
        assert_eq!(other_contents[0], "Unread content");
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_read_directory_files_skips_debrief() {
        let test_dir = create_test_dir("skip_debrief");
        
        // Create DEBRIEF.md and other files
        fs::write(test_dir.join("DEBRIEF.md"), "Debrief content").unwrap();
        fs::write(test_dir.join("file1.txt"), "File content").unwrap();
        
        let result = read_directory_files(&test_dir);
        assert!(result.is_ok());
        
        let (debrief_contents, other_contents, _) = result.unwrap();
        assert_eq!(debrief_contents, "Debrief content");
        assert_eq!(other_contents.len(), 1);
        assert_eq!(other_contents[0], "File content");
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_write_debrief() {
        let test_dir = create_test_dir("write_debrief");
        
        let content = "# Updated Debrief\n\nNew content here.";
        let result = write_debrief(&test_dir, content);
        assert!(result.is_ok());
        
        // Verify the file was written correctly
        let debrief_path = test_dir.join("DEBRIEF.md");
        assert!(debrief_path.exists());
        let written_content = fs::read_to_string(&debrief_path).unwrap();
        assert_eq!(written_content, content);
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_write_debrief_overwrites_existing() {
        let test_dir = create_test_dir("overwrite_debrief");
        
        // Create initial DEBRIEF.md
        let debrief_path = test_dir.join("DEBRIEF.md");
        fs::write(&debrief_path, "Old content").unwrap();
        
        // Write new content
        let new_content = "New content";
        let result = write_debrief(&test_dir, new_content);
        assert!(result.is_ok());
        
        // Verify it was overwritten
        let written_content = fs::read_to_string(&debrief_path).unwrap();
        assert_eq!(written_content, new_content);
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_mark_files_as_read_single_file() {
        let test_dir = create_test_dir("mark_single");
        
        // Create a test file
        let file_path = test_dir.join("test.txt");
        fs::write(&file_path, "Test content").unwrap();
        
        let result = mark_files_as_read(vec![file_path.clone()]);
        assert!(result.is_ok());
        
        // Verify the file was renamed
        assert!(!file_path.exists());
        assert!(test_dir.join("test_read.txt").exists());
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_mark_files_as_read_multiple_files() {
        let test_dir = create_test_dir("mark_multiple");
        
        // Create multiple test files
        let file1 = test_dir.join("file1.txt");
        let file2 = test_dir.join("file2.md");
        let file3 = test_dir.join("file3.json");
        
        fs::write(&file1, "Content 1").unwrap();
        fs::write(&file2, "Content 2").unwrap();
        fs::write(&file3, "Content 3").unwrap();
        
        let result = mark_files_as_read(vec![file1.clone(), file2.clone(), file3.clone()]);
        assert!(result.is_ok());
        
        // Verify all files were renamed
        assert!(!file1.exists());
        assert!(!file2.exists());
        assert!(!file3.exists());
        assert!(test_dir.join("file1_read.txt").exists());
        assert!(test_dir.join("file2_read.md").exists());
        assert!(test_dir.join("file3_read.json").exists());
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_mark_files_as_read_preserves_content() {
        let test_dir = create_test_dir("mark_preserves");
        
        // Create a test file with content
        let file_path = test_dir.join("data.txt");
        let content = "Important data that should be preserved";
        fs::write(&file_path, content).unwrap();
        
        let result = mark_files_as_read(vec![file_path]);
        assert!(result.is_ok());
        
        // Verify content is preserved
        let renamed_path = test_dir.join("data_read.txt");
        let read_content = fs::read_to_string(&renamed_path).unwrap();
        assert_eq!(read_content, content);
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_mark_files_as_read_empty_list() {
        let result = mark_files_as_read(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_integration_read_write_mark_workflow() {
        let test_dir = create_test_dir("integration");
        
        // Step 1: Create some files
        fs::write(test_dir.join("conv1.txt"), "Conversation 1").unwrap();
        fs::write(test_dir.join("conv2.txt"), "Conversation 2").unwrap();
        
        // Step 2: Read directory
        let (debrief_contents, other_contents, unread_paths) = read_directory_files(&test_dir).unwrap();
        assert_eq!(debrief_contents, "# Debrief\n");
        assert_eq!(other_contents.len(), 2);
        assert_eq!(unread_paths.len(), 2);
        
        // Step 3: Write updated debrief
        let new_debrief = "# Debrief\n\n## Summary\n\nProcessed 2 conversations.";
        write_debrief(&test_dir, new_debrief).unwrap();
        
        // Step 4: Mark files as read
        mark_files_as_read(unread_paths).unwrap();
        
        // Step 5: Read directory again - should only find debrief
        let (debrief_contents2, other_contents2, unread_paths2) = read_directory_files(&test_dir).unwrap();
        assert_eq!(debrief_contents2, new_debrief);
        assert_eq!(other_contents2.len(), 0);
        assert_eq!(unread_paths2.len(), 0);
        
        cleanup_test_dir(&test_dir);
    }
}
