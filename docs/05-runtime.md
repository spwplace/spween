# Runtime API

The runtime executes parsed scenes and manages state transitions.

## Overview

```rust
use spween::{parse, Runtime, EffectHandler, Value};

// Parse scene
let scene = parse(source, "scene.scene")?;

// Create runtime with your game state
let mut runtime = Runtime::new(&scene, my_game);

// Interact with the scene
while !runtime.is_ended() {
    let prose = runtime.current_prose();
    let choices = runtime.available_choices();
    runtime.select_choice(chosen_index)?;
}
```

## The EffectHandler Trait

You must implement this trait to connect spween to your game:

```rust
pub trait EffectHandler {
    /// Get a variable's value
    fn get_var(&self, name: &str) -> Value;

    /// Set a variable's value
    fn set_var(&mut self, name: &str, value: Value);

    /// Check if category has key (for `category.key` conditions)
    fn has(&self, category: &str, key: &str) -> bool;

    /// Execute a custom effect
    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String>;
}
```

### Minimal Implementation

```rust
use std::collections::HashMap;
use spween::{EffectHandler, Value};

struct SimpleGame {
    vars: HashMap<String, Value>,
}

impl EffectHandler for SimpleGame {
    fn get_var(&self, name: &str) -> Value {
        self.vars.get(name).cloned().unwrap_or(Value::Null)
    }

    fn set_var(&mut self, name: &str, value: Value) {
        self.vars.insert(name.to_string(), value);
    }

    fn has(&self, _category: &str, _key: &str) -> bool {
        false
    }

    fn call(&mut self, _name: &str, _args: &[Value]) -> Result<(), String> {
        Ok(())
    }
}
```

### Full Implementation

```rust
use std::collections::{HashMap, HashSet};
use spween::{EffectHandler, Value};

struct Game {
    // Core stats
    health: i64,
    gold: i64,
    level: i64,

    // Collections
    inventory: HashSet<String>,
    skills: HashSet<String>,
    quests_complete: HashSet<String>,

    // Flags and counters
    flags: HashMap<String, Value>,

    // External systems
    audio: AudioSystem,
    events: EventQueue,
}

impl EffectHandler for Game {
    fn get_var(&self, name: &str) -> Value {
        // Check core stats first
        match name {
            "health" => return Value::Int(self.health),
            "gold" => return Value::Int(self.gold),
            "level" => return Value::Int(self.level),
            _ => {}
        }

        // Then check flags
        self.flags.get(name).cloned().unwrap_or(Value::Null)
    }

    fn set_var(&mut self, name: &str, value: Value) {
        // Handle core stats
        match name {
            "health" => {
                if let Some(v) = value.as_int() {
                    self.health = v.clamp(0, 100);
                }
                return;
            }
            "gold" => {
                if let Some(v) = value.as_int() {
                    self.gold = v.max(0);
                }
                return;
            }
            "level" => {
                if let Some(v) = value.as_int() {
                    self.level = v.max(1);
                }
                return;
            }
            _ => {}
        }

        // Store in flags
        self.flags.insert(name.to_string(), value);
    }

    fn has(&self, category: &str, key: &str) -> bool {
        match category {
            "inventory" => self.inventory.contains(key),
            "skills" => self.skills.contains(key),
            "quests" => self.quests_complete.contains(key),
            _ => false
        }
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
        match name {
            "add_item" => {
                let item = args.get(0).and_then(|v| v.as_str())
                    .ok_or("add_item requires item name")?;
                self.inventory.insert(item.to_string());
                Ok(())
            }
            "remove_item" => {
                let item = args.get(0).and_then(|v| v.as_str())
                    .ok_or("remove_item requires item name")?;
                self.inventory.remove(item);
                Ok(())
            }
            "play_sound" => {
                if let Some(sound) = args.get(0).and_then(|v| v.as_str()) {
                    self.audio.play(sound);
                }
                Ok(())
            }
            "complete_quest" => {
                let quest = args.get(0).and_then(|v| v.as_str())
                    .ok_or("complete_quest requires quest id")?;
                self.quests_complete.insert(quest.to_string());
                self.events.push(format!("quest_complete:{}", quest));
                Ok(())
            }
            _ => {
                // Log unknown effects but don't fail
                eprintln!("Unknown effect: {} {:?}", name, args);
                Ok(())
            }
        }
    }
}
```

## Runtime Methods

### Creating a Runtime

```rust
let runtime = Runtime::new(&scene, handler);
```

The runtime borrows the scene and owns the handler.

### State Checking

```rust
// Is the scene finished?
if runtime.is_ended() {
    println!("Scene complete!");
}

// Get current state
match runtime.state() {
    RuntimeState::Running(passage_idx) => {
        println!("At passage {}", passage_idx);
    }
    RuntimeState::Ended => {
        println!("Scene ended");
    }
}
```

### Current Passage

```rust
// Get the current passage
if let Some(passage) = runtime.current_passage() {
    println!("Passage: {}", passage.name);
}

// Get just the prose text
if let Some(prose) = runtime.current_prose() {
    println!("{}", prose);
}
```

### Choices

```rust
// All choices with availability status
let all_choices = runtime.current_choices();
for choice in all_choices {
    println!("[{}] {} (available: {})",
        choice.index,
        choice.text,
        choice.available
    );
}

// Only available choices
let available = runtime.available_choices();
for choice in available {
    println!("[{}] {}", choice.index, choice.text);
}
```

### Selecting Choices

```rust
match runtime.select_choice(index) {
    Ok(()) => {
        // Choice executed successfully
    }
    Err(RuntimeError::InvalidChoiceIndex { index, available }) => {
        println!("Invalid choice {} (only {} available)", index, available);
    }
    Err(RuntimeError::ConditionNotMet) => {
        println!("That choice isn't available right now");
    }
    Err(RuntimeError::EffectError(msg)) => {
        println!("Effect failed: {}", msg);
    }
    Err(RuntimeError::SceneEnded) => {
        println!("Scene has already ended");
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

### Direct Navigation

Jump to a specific passage (bypasses choice selection):

```rust
runtime.jump_to("combat")?;  // Jump to "combat" passage
runtime.jump_to("END")?;     // End the scene
```

### Scene Requirements

Check if the scene's preconditions are met:

```rust
if runtime.check_scene_requirements() {
    // Player meets requirements for this scene
} else {
    // Skip this scene
}
```

### Accessing the Handler

```rust
// Read-only access
let handler = runtime.handler();
println!("Gold: {:?}", handler.get_var("gold"));

// Mutable access (for external state changes)
let handler = runtime.handler_mut();
handler.set_var("health", Value::Int(100));

// Take ownership of handler when done
let final_state = runtime.into_handler();
```

## Game Loop Example

```rust
fn run_scene(scene: &Scene, game: &mut Game) -> Result<(), Box<dyn Error>> {
    let mut runtime = Runtime::new(scene, game);

    while !runtime.is_ended() {
        // Clear screen
        print!("\x1B[2J\x1B[H");

        // Show prose
        if let Some(prose) = runtime.current_prose() {
            println!("{}\n", prose);
        }

        // Show choices
        let choices = runtime.available_choices();
        if choices.is_empty() {
            println!("[Press Enter to continue]");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            break;
        }

        for choice in &choices {
            println!("  {}. {}", choice.index + 1, choice.text);
        }

        // Get input
        print!("\n> ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        // Parse and select
        if let Ok(num) = input.trim().parse::<usize>() {
            if num > 0 && num <= choices.len() {
                if let Err(e) = runtime.select_choice(num - 1) {
                    println!("Error: {}", e);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        }
    }

    // Scene ended - get final state
    *game = runtime.into_handler();
    Ok(())
}
```

## Error Types

```rust
pub enum RuntimeError {
    /// Tried to navigate to unknown passage
    UnknownPassage(String),

    /// No current passage (shouldn't happen normally)
    NoCurrentPassage,

    /// Choice index out of bounds
    InvalidChoiceIndex { index: usize, available: usize },

    /// Tried to select a choice whose condition wasn't met
    ConditionNotMet,

    /// A call effect returned an error
    EffectError(String),

    /// Tried to interact with an ended scene
    SceneEnded,
}
```

## Thread Safety

The `Runtime` itself is not `Send` or `Sync` because it holds mutable state. If you need thread-safe scene execution:

1. Run the runtime on a single thread
2. Use channels to communicate state changes
3. Or wrap your handler in appropriate synchronization primitives

## Performance Tips

1. **Reuse scenes**: Parse once, run multiple times
2. **Clone sparingly**: Effects clone choice data internally
3. **Keep handlers lightweight**: Avoid expensive operations in `get_var`/`set_var`
4. **Batch state changes**: Update external systems in `call()` efficiently
