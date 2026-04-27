# Debugging File Generation Issues

## Quick Test

Run this PowerShell command to test file generation:

```powershell
# 1. Create test directory
$TestDir = "$env:USERPROFILE\test-peridot-$(Get-Random)"
New-Item -ItemType Directory -Path $TestDir -Force | Out-Null
Set-Location $TestDir

# 2. Run with debug logging
$env:RUST_LOG = "debug"
D:\Codingan\antigane\peridotcode\target\release\peridotcode.exe
```

In the TUI, type: **"Create a platformer game"**

## Expected Output in Logs

You should see these log messages:

```
[INFO] Template engine initialized with X templates
[INFO] Available templates: 1
[INFO]   - phaser-2d-starter: Phaser 2D Starter
[INFO] Generating scaffold to: "C:\\Users\\...\\test-peridot-..."
[INFO] Generated 7 files: Created 7 files
[INFO]   Created: .\index.html
[INFO]   Created: .\package.json
[INFO]   Created: .\src\main.js
...
```

## Common Issues & Solutions

### Issue 1: "No templates available"

**Symptom:** Log shows `Available templates: 0`

**Cause:** Template engine can't find templates

**Solution:**
1. Check templates exist:
   ```powershell
   Test-Path "D:\Codingan\antigane\peridotcode\templates\phaser-2d-starter\template.toml"
   ```

2. If running from different directory, templates should be embedded. Check build logs for embedded template loading.

3. Rebuild with templates:
   ```bash
   cargo build --release
   ```

### Issue 2: "Template generation failed"

**Symptom:** Error in logs about template generation

**Cause:** Template engine found but can't generate files

**Solution:**
1. Check output directory permissions
2. Run with debug logging to see exact error
3. Verify template.toml is valid

### Issue 3: Files not written silently

**Symptom:** No error, but no files created

**Cause:** Error being swallowed or wrong output path

**Solution:** 
1. Check the logs for the actual output path
2. Verify the path exists and is writable
3. Look for `[ERROR]` in logs

## Manual Verification

### Step 1: Check Template Engine Directly

Create a test Rust file:

```rust
use peridot_template_engine::{TemplateEngine, TemplateContext};
use std::path::PathBuf;

fn main() {
    // Create engine
    let engine = TemplateEngine::new().expect("Failed to create engine");
    
    // List templates
    let templates = engine.list_templates();
    println!("Found {} templates:", templates.len());
    for t in templates {
        println!("  - {}", t.id.as_ref());
    }
    
    // Try to generate
    let ctx = TemplateContext::from_project("TestGame", None);
    let result = engine.generate_with_auto_select(None, PathBuf::from("./output"), &ctx);
    
    match result {
        Ok(r) => println!("Generated {} files", r.file_count),
        Err(e) => println!("Error: {}", e),
    }
}
```

### Step 2: Check Orchestrator Flow

Add this to your test to trace the full flow:

```rust
use peridot_core::orchestrator::{Orchestrator, OrchestratorConfig};
use peridot_shared::PromptInput;

#[tokio::main]
async fn main() {
    let orch = Orchestrator::new(OrchestratorConfig::default())
        .expect("Failed to create orchestrator");
    
    let input = PromptInput::new("Create a platformer game");
    let result = orch.process_prompt(input).await;
    
    println!("Success: {}", result.success);
    println!("Intent: {}", result.intent.display_name());
    
    if let Some(exec_result) = result.execution_result {
        println!("Files created: {}", exec_result.created_files.len());
        for f in &exec_result.created_files {
            println!("  - {:?}", f);
        }
    }
    
    if let Some(err) = result.error {
        println!("Error: {}", err);
    }
}
```

## Debugging Checklist

- [ ] Templates exist in `templates/phaser-2d-starter/`
- [ ] `template.toml` is valid and readable
- [ ] Running peridotcode from a writable directory
- [ ] No permission errors in logs
- [ ] Template engine reports templates found
- [ ] Output path is correct in logs
- [ ] File changes are tracked

## Enable Full Debug Output

```powershell
$env:RUST_LOG = "trace"
$env:RUST_BACKTRACE = "1"
D:\Codingan\antigane\peridotcode\target\release\peridotcode.exe 2>&1 | Tee-Object -FilePath "debug.log"
```

Then check `debug.log` for detailed execution trace.

## What Should Happen

1. User types: "Create a platformer game"
2. TUI sends to orchestrator
3. Orchestrator classifies as `CreateNewGame`
4. Planner creates plan with steps:
   - load_context
   - select_template
   - generate_scaffold ← Files written HERE
   - write_files
5. `GenerateScaffold` action executes
6. Template engine loads "phaser-2d-starter"
7. Template engine copies files to output directory
8. Files appear in the folder!

## If Still Not Working

Check these specific items:

1. **Is the template engine initialized?**
   - Look for log: `Template engine initialized with X templates`

2. **Is the template found?**
   - Look for log: `Available templates: 1`

3. **Is scaffold generation called?**
   - Look for log: `Generating scaffold to: ...`

4. **Are files being tracked?**
   - Look for logs: `Created: .\filename`

5. **Is there an error?**
   - Search logs for `[ERROR]`

## Working Example

When working correctly, you'll see:

```
[INFO] Template engine initialized with 1 templates
[INFO] Processing prompt: Create a platformer game
[INFO] Classified: Create New Game (100% confidence)
[INFO] Plan: Create new game project
[INFO] Step: Loading project context
[INFO] Step: Selecting template (1 templates available)
[INFO] Step: Generating project scaffold
[INFO] Available templates: 1
[INFO]   - phaser-2d-starter: Phaser 2D Starter
[INFO] Generating scaffold to: "C:\\Users\\..."
[INFO] Generated 7 files: Created 7 files
[INFO]   Created: .\index.html
[INFO]   Created: .\package.json
[INFO]   Created: .\README.md
[INFO]   Created: .\src\main.js
...
[INFO] Step: Writing files to disk
```

Then verify:
```powershell
ls C:\Users\...\test-directory\
# Should show: index.html, package.json, src/, etc.
```
