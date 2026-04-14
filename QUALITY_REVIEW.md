# Quality and Consistency Review

## Review Date: 2026-01-14

## 1. Architecture Consistency

### ✅ Strengths
- Clear crate boundaries following workspace structure
- Shared types in `peridot-shared` prevent circular dependencies
- Template engine properly isolated from orchestration
- FS engine provides safety guarantees for all file operations

### ⚠️ Issues Found

#### 1.1 Inconsistent Naming Conventions
- `peridot-fs-engine` (kebab) vs `fs_engine` directory (snake)
- `peridot-command-runner` vs `command_runner` directory
- Some modules use snake_case, some use single words

#### 1.2 Core Crate Dependencies
- `Orchestrator` holds `fs_engine` field but never uses it (dead code)
- Template engine does its own file operations instead of using fs_engine
- Skills system has no integration point in orchestrator (just a TODO comment)

#### 1.3 Error Handling Inconsistencies
- Some places use `anyhow::Result` (CLI), some use `PeridotResult` (crates)
- Error messages vary in format and detail level

## 2. Duplication Removal

### Removed Duplications

#### 2.1 Change Tracking Types
- `ChangeType`, `FileChange` defined in both `fs_engine` and re-exported in `template_engine`
- Template engine should use fs_engine types directly, not re-export

#### 2.2 Path Validation
- `validate_project_path` exists in fs_engine but also re-implemented concepts elsewhere
- All path validation should go through fs_engine

#### 2.3 Run Instructions
- `ExecutionResult.instructions` vs `RunInstructions` from command_runner
- Need to consolidate these concepts

## 3. Naming Improvements

### Proposed Renames

| Current | Proposed | Reason |
|---------|----------|--------|
| `fs_engine` | Keep, but uniform | Already good, just ensure consistency |
| `command_runner` | `command` | Shorter, clearer |
| `template_engine` | `templates` | Consistent with Rust conventions |
| `skill` trait method `apply` | `install` | Clearer intent (applies skill to project) |
| `OrchestratorResult` | `SessionResult` | Clearer that it's per-session |
| `ExecutionResult` | `GenerationResult` | More specific to code generation |

## 4. Crate Boundary Issues

### 4.1 Template Engine → FS Engine
- Template engine should use `FsEngine` for all writes
- Currently does its own file operations

### 4.2 Core → Skills
- Orchestrator has `Action::AddSkill` but no skill registry integration
- Skills are stubbed but not wired into the system

### 4.3 Command Runner Integration
- Command runner is created but not used for dev server execution
- Instructions are provided but commands aren't run

## 5. Documentation Gaps

### Missing Documentation
- No architecture diagram in docs/
- No example of adding a custom skill
- No troubleshooting guide for common errors
- Missing CHANGELOG

### Outdated Documentation
- PRD mentions Godot but MVP is Phaser-only
- MVP doc doesn't mention skills foundation
- README has placeholder sections

## 6. Known Gaps (Current vs PRD)

### PRD Requirements Not Yet Implemented

1. **Intent Parsing** (Section 18.2)
   - Currently basic keyword matching
   - Needs more sophisticated classification

2. **Context Awareness** (Section 18.7)
   - Orchestrator reads context but doesn't use it for decisions
   - No file content analysis before modification

3. **Diff Preview** (Section 18.5)
   - Change summary exists but no diff preview
   - PRD mentions "diff preview sebelum apply"

4. **Skill System Foundation** (Section 18.8)
   - Skills defined but not integrated
   - No "add skill" command implementation

5. **Error Recovery** (Section 19)
   - Basic error handling present
   - No recovery path for failed operations

### MVP Requirements Implemented ✅

- [x] Rust workspace
- [x] CLI entrypoint
- [x] Terminal UI shell
- [x] User prompt input
- [x] Basic orchestration
- [x] Template selection
- [x] Phaser 2D starter template
- [x] Project scaffolding
- [x] Generated file summary
- [x] Run instructions

## 7. Proposed Next Three Steps

### Step 1: Integrate FS Engine into Template Engine (High Priority)
**Why**: Ensures all file operations go through safety-checked layer
**What**: Replace direct file writes in `renderer.rs` with `FsEngine` calls
**Impact**: Consistent safety, change tracking, error handling

### Step 2: Wire Skills into Orchestrator (High Priority)
**Why**: Completes the skill system foundation
**What**: Add skill registry to orchestrator, implement `Action::AddSkill` handling
**Impact**: Skills become functional, not just stubs

### Step 3: Implement Intent Classification Improvement (Medium Priority)
**Why**: Current classification is too basic for useful prompts
**What**: Add proper intent parsing with keyword extraction, genre detection
**Impact**: Better template selection, more accurate planning

## 8. Code Quality Issues

### 8.1 Unused Code
- `Orchestrator.fs_engine` field (never used)
- `SkillRegistry::new_const` (doesn't work as intended)
- Some unused imports in tests

### 8.2 TODO Comments Without Issues
Many TODOs lack tracking. Should create issues for:
- Dev server execution
- Config file parsing
- Skill application logic
- Multi-engine support

### 8.3 Test Coverage
- Skills crate: good coverage (25 tests)
- FS engine: good coverage (11 tests)
- Core: minimal tests
- Template engine: minimal tests
- CLI: no tests

## 9. Consistency Checklist

- [x] All crates use workspace dependencies
- [x] All crates use `PeridotResult` for errors (except CLI)
- [ ] All file operations go through `FsEngine` (template engine needs fixing)
- [x] All skills implement `Debug`
- [x] All public APIs have documentation
- [ ] All TODOs reference GitHub issues (needs addressing)

## 10. Recommendations Summary

1. **Immediate**: Fix template engine to use fs_engine
2. **Short-term**: Wire skills into orchestrator
3. **Medium-term**: Improve intent classification
4. **Ongoing**: Create issues for all TODOs, add tests for core/template
