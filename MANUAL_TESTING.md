# Manual Testing Guide for PeridotCode Agent

This guide shows how to test the agent's code modification capabilities on real projects.

## Prerequisites

1. Build peridotcode:
```bash
cargo build --release
```

2. Set up a test project (or use an existing one):
```bash
mkdir test-game
cd test-game
```

## Testing the Agent

### 1. Create a Test Phaser Project

First, create a simple test file to work with:

```bash
mkdir -p src
cat > src/player.js << 'EOF'
class Player {
    constructor(scene, x, y) {
        this.scene = scene;
        this.sprite = scene.add.sprite(x, y, 'player');
        this.speed = 200;
    }

    update(cursors) {
        // TODO: Add movement
    }
}

module.exports = Player;
EOF
```

### 2. Run PeridotCode

From your test project directory:

```bash
/path/to/peridotcode/target/release/peridotcode.exe
```

Or if installed:
```bash
peridotcode
```

### 3. Test Prompts

Once the TUI opens, try these prompts to test the agent:

#### Test 1: Read File Tool
```
"Show me the contents of src/player.js"
```
Expected: Agent should read and display the file contents.

#### Test 2: Modify Code - Add Feature
```
"Add a jump method to the Player class in src/player.js that sets vertical velocity to -300"
```
Expected: Agent should:
1. Read src/player.js
2. Call modify_code tool with the change
3. Save the modified file with the new jump method

#### Test 3: Modify Code - Add Movement
```
"Implement the update method in src/player.js to handle arrow key movement using cursors.left, cursors.right"
```
Expected: Agent should add keyboard movement logic.

#### Test 4: New Game Creation
```
"Create a new platformer game with jumping and enemies"
```
Expected: Agent should use the phaser-2d-starter template.

### 4. Verify Changes

After each modification, check the file:

```bash
cat src/player.js
```

You should see:
- New methods added
- Code properly formatted
- Existing code preserved

### 5. Test File Loading with Context

Create a README with requirements:

```bash
cat > README.md << 'EOF'
# Game Requirements

- Player should have health (100 HP)
- Enemies deal 10 damage
- Collect coins for score
EOF
```

Then prompt:
```
"Based on the README, add health and damage system to the player"
```

The agent should:
1. Load README.md into context
2. Modify src/player.js to add health system

## Testing Workflow

### Full Test Session Example:

```bash
# 1. Create test directory
mkdir -p ~/test-peridot && cd ~/test-peridot

# 2. Run peridotcode
peridotcode

# 3. In the TUI, enter prompts:
# First prompt: "Create a simple platformer game"
# This should scaffold the project

# 4. After scaffolding, test modification:
# "Add double jump ability to the player"

# 5. Check results
ls -la
cat src/scenes/GameScene.js
```

## Debugging

### Check Tool Execution

If tools aren't working, check:

1. Provider configuration is set up:
   - Run setup flow if needed
   - Select OpenRouter provider
   - Enter API key
   - Select model (e.g., openai/gpt-4o-mini)

2. Enable debug logging:
```bash
set RUST_LOG=debug
peridotcode
```

### Verify Tool Registration

The agent should log when tools are used:
```
[DEBUG] Executing tool 'read_file'
[DEBUG] Executing tool 'modify_code'
```

### Check File Safety

The agent respects project boundaries:
- Won't read/write outside the project directory
- Won't overwrite files without tracking

## Advanced Testing

### Test with Conversation History

1. First prompt: "Add a player sprite"
2. Second prompt: "Now make the player move with arrow keys"

The agent should remember context from the first prompt.

### Test File Context Loading

Create multiple files:
```bash
echo "const ENEMY_SPEED = 100;" > src/config.js
echo "class Enemy {}" > src/enemy.js
```

Then: "Make enemies chase the player using the speed from config"

### Test Error Handling

Try invalid requests:
- "Modify nonexistent-file.js" (should error gracefully)
- "Delete everything" (should be rejected by safety checks)

## Expected Output Format

When the agent modifies code, you should see:

```
✓ File read successfully
✓ Code modified successfully

Changes applied to src/player.js:
- Added jump() method
- 15 lines changed

Summary: Added jump functionality with -300 vertical velocity
```

## Cleanup

After testing:
```bash
cd ..
rm -rf test-game
```

## Troubleshooting

### Issue: Agent doesn't respond
- Check if provider is configured
- Verify API key is valid
- Check network connection

### Issue: Files not being modified
- Ensure file paths are relative to project root
- Check file permissions
- Verify you're in the correct directory

### Issue: Changes not applied
- Check the file with `cat` to verify
- Look for error messages in logs
- Ensure the LLM response was parsed correctly

## Test Checklist

- [ ] Create new game from prompt
- [ ] Read existing file
- [ ] Modify existing code
- [ ] Add new method to class
- [ ] Fix bugs in code
- [ ] Handle conversation context
- [ ] Load multiple files
- [ ] Safety boundary enforcement

## Example Test Script

```bash
#!/bin/bash
set -e

# Setup
TEST_DIR="/tmp/peridot-test-$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Create initial file
cat > game.js << 'EOF'
class Game {
    create() {
        console.log('Game created');
    }
}
EOF

# Run peridotcode (this would need interactive input)
echo "Created test project in $TEST_DIR"
echo "Run: cd $TEST_DIR && peridotcode"
echo "Then test with prompt: 'Add an update method to the Game class'"
```

This creates a reproducible test environment.
