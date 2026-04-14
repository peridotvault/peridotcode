# peridot-fs-engine

Safe file system operations for PeridotCode.

## Overview

This crate provides the file system execution layer for PeridotCode with a focus on:

- **Safety**: All paths validated to prevent directory traversal attacks
- **Transparency**: Complete change tracking and reporting
- **Simplicity**: Small, explicit API surface

## Features

### Path Safety

All write operations validate paths stay within the project directory:

```rust
use peridot_fs_engine::FsEngine;

let mut engine = FsEngine::new("./my-project")?;

// These are safe
engine.write_file("src/main.js", "content")?;
engine.write_file("assets/sprite.png", bytes)?;

// These would be rejected
// engine.write_file("../outside.txt", "content")?;  // SafetyViolation error
// engine.write_file("/etc/passwd", "content")?;     // SafetyViolation error
```

### Change Tracking

Every operation is tracked for reporting:

```rust
// Write some files
engine.write_file("README.md", "# Title")?;
engine.write_file("src/main.js", "code")?;

// Get formatted report
let summary = engine.take_change_summary();
println!("{}", summary.format_report());
```

Output:
```text
File Changes:
============

Created (2):
  + README.md
  + src/main.js

Total: 2 created, 0 modified, 0 deleted
```

## Module Structure

| Module | Purpose |
|--------|---------|
| `safety` | Path validation and directory traversal prevention |
| `read` | Safe file reading with size limits |
| `write` | File writing with automatic directory creation |
| `summary` | Change tracking and reporting |
| `operations` | Operation types and batch processing |

## Integration

The fs_engine is used by:

- **template_engine**: For safe scaffold generation
- **core/orchestrator**: For coordinated file operations
- **CLI/TUI**: For displaying change summaries to users

## Safety Model

1. **Canonicalization**: Project root is canonicalized at engine creation
2. **Path Resolution**: All relative paths are resolved against project root
3. **Validation**: Target paths are canonicalized and verified to be within project
4. **Parent Check**: For new files, parent directory must exist within project

## Limitations

- Not a full diff engine (no line-by-line diffs)
- No transaction/rollback support
- Not designed for concurrent access
- 10MB file size limit for reads

See module-level documentation for detailed assumptions and deferred improvements.
