use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use read_files::autorater;
/// Helper to create a temporary test directory
fn create_temp_dir(name: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_dir = std::env::temp_dir().join(format!("integration_test_{}_{}", name, timestamp));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).unwrap();
    }
    fs::create_dir(&temp_dir).unwrap();
    temp_dir
}

/// Helper to clean up test directory
fn cleanup_dir(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path).ok();
    }
}

/// Helper to copy a file from testdata to target directory
fn copy_testdata_file(testdata_dir: &str, filename: &str, target_dir: &Path) {
    let source = PathBuf::from("testdata")
        .join(testdata_dir)
        .join(filename);
    let dest = target_dir.join(filename);
    fs::copy(&source, &dest).unwrap();
}

/// Helper to run the binary on a directory
fn run_pipeline(dir: &Path) -> bool {
    let output = Command::new("cargo")
        .args(&["run", "--", dir.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");
    
    output.status.success()
}

/// Helper to read DEBRIEF.md content
fn read_debrief(dir: &Path) -> String {
    let debrief_path = dir.join("DEBRIEF.md");
    fs::read_to_string(&debrief_path).unwrap()
}



/// Parameterized integration test for a testdata directory
#[tokio::main]
async fn test_pipeline_for_directory(testdata_dir: &str) {
    let test_dir = create_temp_dir(testdata_dir);
    
    // Step 1: Copy 1.txt and 2.txt
    copy_testdata_file(testdata_dir, "1.txt", &test_dir);
    copy_testdata_file(testdata_dir, "2.txt", &test_dir);
    
    // Step 2: Run the pipeline
    let success = run_pipeline(&test_dir);
    assert!(success, "First pipeline run failed for {}", testdata_dir);
    
    // Step 3: Check that DEBRIEF.md exists and has content
    let debrief_path = test_dir.join("DEBRIEF.md");
    assert!(debrief_path.exists(), "DEBRIEF.md was not created for {}", testdata_dir);
    
    let first_debrief_content = read_debrief(&test_dir);
    assert!(!first_debrief_content.is_empty(), "DEBRIEF.md is empty for {}", testdata_dir);
    assert!(first_debrief_content.len() > 50, "DEBRIEF.md content seems too short for {}", testdata_dir);
    
    println!("First DEBRIEF.md for {} ({} bytes):\n{}\n", testdata_dir, first_debrief_content.len(), first_debrief_content);
    
    // Autorater evaluation of first debrief
    let file1_content = fs::read_to_string(test_dir.join("1_read.txt")).unwrap();
    let file2_content = fs::read_to_string(test_dir.join("2_read.txt")).unwrap();
    let first_eval = autorater::evaluate_debrief(
        &[file1_content.clone(), file2_content.clone()],
        &first_debrief_content,
        "Initial debrief creation from files 1 and 2",
    ).await.expect("Failed to evaluate first debrief");
    
    println!("Autorater evaluation (first run): Score {}/10", first_eval.score);
    println!("Reasoning: {}", first_eval.reasoning);
    if !first_eval.issues.is_empty() {
        println!("Issues: {:?}", first_eval.issues);
    }
    
    assert!(first_eval.score >= 6, "First debrief quality too low: {}/10. Reasoning: {}", first_eval.score, first_eval.reasoning);
    
    // Verify files were marked as read
    assert!(test_dir.join("1_read.txt").exists(), "1.txt was not marked as read");
    assert!(test_dir.join("2_read.txt").exists(), "2.txt was not marked as read");
    assert!(!test_dir.join("1.txt").exists(), "1.txt still exists (should be renamed)");
    assert!(!test_dir.join("2.txt").exists(), "2.txt still exists (should be renamed)");
    
    // Step 4: Copy 3.txt
    copy_testdata_file(testdata_dir, "3.txt", &test_dir);
    
    // Step 5: Run the pipeline again
    let success = run_pipeline(&test_dir);
    assert!(success, "Second pipeline run failed for {}", testdata_dir);
    
    // Step 6: Check that DEBRIEF.md has different content
    let second_debrief_content = read_debrief(&test_dir);
    assert!(!second_debrief_content.is_empty(), "DEBRIEF.md is empty after second run for {}", testdata_dir);
    assert_ne!(
        first_debrief_content, 
        second_debrief_content, 
        "DEBRIEF.md content did not change after processing 3.txt for {}", 
        testdata_dir
    );
    
    println!("Second DEBRIEF.md for {} ({} bytes):\n{}\n", testdata_dir, second_debrief_content.len(), second_debrief_content);
    
    // Autorater evaluation of second debrief (updated with file 3)
    let file3_content = fs::read_to_string(test_dir.join("3_read.txt")).unwrap();
    let second_eval = autorater::evaluate_debrief(
        &[file1_content, file2_content, file3_content],
        &second_debrief_content,
        "Updated debrief after adding file 3",
    ).await.expect("Failed to evaluate second debrief");
    
    println!("Autorater evaluation (second run): Score {}/10", second_eval.score);
    println!("Reasoning: {}", second_eval.reasoning);
    if !second_eval.issues.is_empty() {
        println!("Issues: {:?}", second_eval.issues);
    }
    
    assert!(second_eval.score >= 6, "Second debrief quality too low: {}/10. Reasoning: {}", second_eval.score, second_eval.reasoning);
    
    // Verify 3.txt was marked as read
    assert!(test_dir.join("3_read.txt").exists(), "3.txt was not marked as read");
    assert!(!test_dir.join("3.txt").exists(), "3.txt still exists (should be renamed)");
    
    // Step 7: Cleanup
    cleanup_dir(&test_dir);
}

#[test]
#[ignore] // Ignore by default since this requires API key and makes real API calls
fn test_integration_hamstring_injury() {
    test_pipeline_for_directory("hamstring_injury");
}

#[test]
#[ignore] // Ignore by default since this requires API key and makes real API calls
fn test_integration_hard_drive() {
    test_pipeline_for_directory("hard_drive");
}

#[test]
#[ignore] // Run both directories in sequence
fn test_integration_all_directories() {
    test_pipeline_for_directory("hamstring_injury");
    test_pipeline_for_directory("hard_drive");
}
