//! Integration tests for fs_engine

use peridot_fs_engine::{ChangeType, FsEngine};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_fs_engine_full_workflow() {
    // Create a temporary project directory
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create engine
    let mut engine = FsEngine::new(project_path).unwrap();

    // Write some files
    engine
        .write_file("src/main.js", "console.log('hello');")
        .unwrap();
    engine
        .write_file("package.json", "{\"name\": \"test\"}")
        .unwrap();
    engine.create_dir("assets").unwrap();
    engine.write_file("assets/README.md", "# Assets").unwrap();

    // Verify files exist
    assert!(project_path.join("src/main.js").exists());
    assert!(project_path.join("package.json").exists());
    assert!(project_path.join("assets").exists());
    assert!(project_path.join("assets/README.md").exists());

    // Check change summary
    let summary = engine.change_summary();
    assert_eq!(summary.len(), 4); // 3 files + 1 directory
    assert_eq!(summary.counts(), (4, 0, 0)); // 4 created, 0 modified, 0 deleted

    // Verify all are marked as created
    for change in summary.changes() {
        assert_eq!(change.change_type, ChangeType::Created);
    }

    // Write same file again (should be unchanged)
    engine
        .write_file("src/main.js", "console.log('hello');")
        .unwrap();
    let summary = engine.change_summary();
    let unchanged = summary.changes_of_type(ChangeType::Unchanged);
    assert_eq!(unchanged.len(), 1);
    assert_eq!(unchanged[0].path.to_str().unwrap(), "src/main.js");

    // Write with different content (should be modified)
    engine
        .write_file("src/main.js", "console.log('world');")
        .unwrap();
    let summary = engine.change_summary();
    let modified = summary.changes_of_type(ChangeType::Modified);
    assert_eq!(modified.len(), 1);
    assert_eq!(modified[0].path.to_str().unwrap(), "src/main.js");
}

#[test]
fn test_path_safety_rejects_traversal() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    let mut engine = FsEngine::new(project_path).unwrap();

    // Attempt to write outside project should fail
    let result = engine.write_file("../outside.txt", "content");
    assert!(result.is_err());

    let result = engine.write_file("foo/../../outside.txt", "content");
    assert!(result.is_err());
}

#[test]
fn test_change_summary_report() {
    let temp_dir = TempDir::new().unwrap();
    let mut engine = FsEngine::new(&temp_dir).unwrap();

    // Create files
    engine.write_file("a.js", "a").unwrap();
    engine.write_file("b.js", "b").unwrap();

    // Modify one
    fs::write(temp_dir.path().join("c.js"), "original").unwrap();
    engine.write_file("c.js", "modified").unwrap();

    // Keep one unchanged
    fs::write(temp_dir.path().join("d.js"), "same").unwrap();
    engine.write_file("d.js", "same").unwrap();

    let summary = engine.change_summary();
    let report = summary.format_report();

    // Verify report contains expected sections
    assert!(
        report.contains("Created (2):"),
        "Expected 'Created (2):' in report:\n{}",
        report
    );
    assert!(
        report.contains("Modified (1):"),
        "Expected 'Modified (1):' in report:\n{}",
        report
    );
    assert!(
        report.contains("Unchanged (1 files)"),
        "Expected 'Unchanged (1 files)' in report:\n{}",
        report
    );
    assert!(
        report.contains("+ a.js"),
        "Expected '+ a.js' in report:\n{}",
        report
    );
    assert!(
        report.contains("+ b.js"),
        "Expected '+ b.js' in report:\n{}",
        report
    );
    assert!(
        report.contains("~ c.js"),
        "Expected '~ c.js' in report:\n{}",
        report
    );
    assert!(
        report.contains("Total: 2 created, 1 modified, 0 deleted"),
        "Expected summary line in report:\n{}",
        report
    );
}

#[test]
fn test_batch_write_files() {
    let temp_dir = TempDir::new().unwrap();
    let mut engine = FsEngine::new(&temp_dir).unwrap();

    let files = vec![
        ("file1.txt".into(), "content1".into()),
        ("file2.txt".into(), "content2".into()),
        ("nested/file3.txt".into(), "content3".into()),
    ];

    let results = engine.write_files(files);

    // All should succeed
    assert_eq!(results.len(), 3);
    for result in results {
        assert!(result.is_ok());
    }

    // Verify files exist
    assert!(temp_dir.path().join("file1.txt").exists());
    assert!(temp_dir.path().join("file2.txt").exists());
    assert!(temp_dir.path().join("nested/file3.txt").exists());
}
