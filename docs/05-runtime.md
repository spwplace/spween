# Runtime API

The runtime is where everything comes together. It takes your parsed scene and your game state, and provides a clean API for driving interactive narratives. This guide covers everything you need to integrate spween into your game.

## The Big Picture

Here's the typical flow:

```rust
use spween::{parse, Runtime, EffectHandler, Value};

// 1. Parse your scene file
let scene = parse(source, "scene.scene")?;

// 2. Create a runtime with your game state
let mut runtime = Runtime::new(&scene, my_game);

// 3. Drive the narrative
while !runtime.is_ended() {
    let prose = runtime.current_prose();
    let choices = runtime.available_choices();
    // ... display to player, get input ...
    runtime.select_choice(player_choice)?;
}
```

The runtime manages which passage you're in, evaluates conditions, executes effects, and handles navigation. You just need to show content and handle input.

## The EffectHandler Trait

Before we dive into the runtime, let's fully understand the trait that connects spween to your game:

```rust
pub trait EffectHandler {
    /// Get a variable's current value
    fn get_var(&self, name: &str) -> Value;

    /// Set a variable to a new value
    fn set_var(&mut self, name: &str, value: Value);

    /// Check if a category contains a key (like inventory.sword)
    fn has(&self, category: &str, key: &str) -> bool;

    /// Execute a custom effect command
    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String>;
}
```

Each method serves a specific purpose:
- **`get_var`**: Called when evaluating conditions (`{ gold >= 50 }`) and when modifying variables (`~ gold += 10` needs to know the current value)
- **`set_var`**: Called for assignment effects (`~ visited = true`) and after modification effects compute the new value
- **`has`**: Called for category.key conditions (`{ inventory.sword }`)
- **`call`**: Called for custom effects (`~ play_sound "victory"`)

### A Minimal Implementation

If you're just getting started, here's the simplest working handler:

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
        false  // No inventory system yet
    }

    fn call(&mut self, _name: &str, _args: &[Value]) -> Result<(), String> {
        Ok(())  // Ignore custom effects for now
    }
}
```

This stores all variables in a HashMap and ignores inventory checks and custom effects. It's enough to run simple scenes while you build out your game.

### A Complete Implementation

Here's a more realistic example with proper game systems:

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
    completed_quests: HashSet<String>,

    // Arbitrary flags and counters
    flags: HashMap<String, Value>,

    // External systems (you'd have your own versions)
    audio: AudioSystem,
    events: EventQueue,
}

impl EffectHandler for Game {
    fn get_var(&self, name: &str) -> Value {
        // Check well-known variables first
        match name {
            "health" => return Value::Int(self.health),
            "gold" => return Value::Int(self.gold),
            "level" => return Value::Int(self.level),
            _ => {}
        }

        // Fall back to flags
        self.flags.get(name).cloned().unwrap_or(Value::Null)
    }

    fn set_var(&mut self, name: &str, value: Value) {
        // Handle well-known variables with validation
        match name {
            "health" => {
                if let Some(v) = value.as_int() {
                    self.health = v.clamp(0, 100);  // Cap at 0-100
                }
                return;
            }
            "gold" => {
                if let Some(v) = value.as_int() {
                    self.gold = v.max(0);  // Can't go negative
                }
                return;
            }
            "level" => {
                if let Some(v) = value.as_int() {
                    self.level = v.max(1);  // Minimum level 1
                }
                return;
            }
            _ => {}
        }

        // Store everything else in flags
        self.flags.insert(name.to_string(), value);
    }

    fn has(&self, category: &str, key: &str) -> bool {
        match category {
            "inventory" => self.inventory.contains(key),
            "skills" => self.skills.contains(key),
            "quests" => self.completed_quests.contains(key),
            _ => false
        }
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
        match name {
            "add_item" => {
                let item = args.get(0).and_then(|v| v.as_str())
                    .ok_or("add_item requires an item name")?;
                self.inventory.insert(item.to_string());
                Ok(())
            }
            "remove_item" => {
                let item = args.get(0).and_then(|v| v.as_str())
                    .ok_or("remove_item requires an item name")?;
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
                    .ok_or("complete_quest requires a quest id")?;
                self.completed_quests.insert(quest.to_string());
                self.events.push(format!("quest_complete:{}", quest));
                Ok(())
            }
            _ => {
                // Log unknown effects but don't fail
                // This makes scenes forward-compatible
                eprintln!("Unknown effect: {} {:?}", name, args);
                Ok(())
            }
        }
    }
}
```

## Creating a Runtime

Once you have a scene and a handler, creating a runtime is straightforward:

```rust
let runtime = Runtime::new(&scene, handler);
```

The runtime borrows the scene (so you can reuse it) and owns the handler. This ownership model lets you get your game state back when the scene ends.

## Checking Runtime State

### Is the Scene Over?

```rust
if runtime.is_ended() {
    println!("Scene complete!");
}
```

### Getting Detailed State

```rust
match runtime.state() {
    RuntimeState::Running(passage_idx) => {
        println!("Currently at passage index {}", passage_idx);
    }
    RuntimeState::Ended => {
        println!("Scene has ended");
    }
}
```

## Working with the Current Passage

### Getting the Passage

```rust
if let Some(passage) = runtime.current_passage() {
    println!("You're in: {}", passage.name);
}
```

### Getting Just the Prose

Most of the time, you just want the text to display:

```rust
if let Some(prose) = runtime.current_prose() {
    println!("{}", prose);
}
```

This returns all the prose content in the current passage as a single string.

## Working with Choices

### Getting All Choices (with Availability)

```rust
let all_choices = runtime.current_choices();
for choice in all_choices {
    if choice.available {
        println!("[{}] {}", choice.index, choice.text);
    } else {
        println!("[{}] {} (locked)", choice.index, choice.text);
    }
}
```

Each `AvailableChoice` contains:
- `index`: The choice's position (use this when selecting)
- `text`: The display text from `[brackets]`
- `available`: Whether the condition passed

### Getting Only Available Choices

If you want to hide locked choices entirely:

```rust
let available = runtime.available_choices();
for choice in available {
    println!("[{}] {}", choice.index, choice.text);
}
```

## Selecting Choices

When the player makes a decision:

```rust
match runtime.select_choice(index) {
    Ok(()) => {
        // Success! Effects ran, navigation happened
    }
    Err(RuntimeError::InvalidChoiceIndex { index, available }) => {
        println!("Invalid choice {} - only {} choices available", index, available);
    }
    Err(RuntimeError::ConditionNotMet) => {
        println!("That choice isn't available right now");
    }
    Err(RuntimeError::EffectError(msg)) => {
        println!("Something went wrong: {}", msg);
    }
    Err(RuntimeError::SceneEnded) => {
        println!("The scene has already ended");
    }
    Err(RuntimeError::UnknownPassage(name)) => {
        println!("Scene tried to navigate to unknown passage: {}", name);
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

## Direct Navigation

Sometimes you need to jump to a passage programmatically—maybe for debugging, or to implement a "skip" feature:

```rust
// Jump to a specific passage
runtime.jump_to("combat")?;

// End the scene immediately
runtime.jump_to("END")?;
```

This bypasses choice selection entirely—no conditions are checked, no effects run.

## Checking Scene Requirements

If your scene has requirements in its frontmatter:

```rust
if runtime.check_scene_requirements() {
    // Player meets all requirements
    // Safe to run this scene
} else {
    // Requirements not met
    // Pick a different scene
}
```

This is useful when building a scene selection system.

## Accessing Your Game State

### Read-Only Access

```rust
let handler = runtime.handler();
let gold = handler.get_var("gold");
println!("Current gold: {:?}", gold);
```

### Mutable Access

Need to modify state from outside the scene? Maybe a timer ticked, or something happened in the game world:

```rust
let handler = runtime.handler_mut();
handler.set_var("time_remaining", Value::Int(30));
```

### Taking Back Ownership

When the scene ends and you need your game state back:

```rust
let final_state = runtime.into_handler();
// runtime is now consumed
// final_state is your Game struct with all changes applied
```

## A Complete Game Loop

Here's a realistic example tying everything together:

```rust
use std::io::{self, Write};

fn run_scene(scene: &Scene, game: Game) -> Result<Game, Box<dyn Error>> {
    let mut runtime = Runtime::new(scene, game);

    println!("\n=== {} ===\n", scene.meta.title);

    while !runtime.is_ended() {
        // Show current stats
        {
            let state = runtime.handler();
            println!("[Health: {} | Gold: {}]",
                state.health,
                state.gold
            );
        }

        // Show prose
        if let Some(prose) = runtime.current_prose() {
            println!("\n{}\n", prose);
        }

        // Show choices
        let choices = runtime.available_choices();
        if choices.is_empty() {
            // No choices = end of content
            println!("[Press Enter to continue]");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            break;
        }

        for choice in &choices {
            println!("  {}. {}", choice.index + 1, choice.text);
        }

        // Get input
        print!("\n> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // Parse and select
        match input.trim().parse::<usize>() {
            Ok(num) if num > 0 && num <= choices.len() => {
                if let Err(e) = runtime.select_choice(num - 1) {
                    println!("\nError: {}\n", e);
                }
            }
            _ => {
                println!("\nPlease enter a number between 1 and {}\n", choices.len());
            }
        }
    }

    println!("\n=== Scene Complete ===\n");

    // Return the game state with all changes
    Ok(runtime.into_handler())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load scene
    let source = std::fs::read_to_string("adventure.scene")?;
    let scene = parse(&source, "adventure.scene")?;

    // Create initial game state
    let game = Game::new();

    // Run the scene
    let final_game = run_scene(&scene, game)?;

    // Continue with the modified game state...
    println!("Final gold: {}", final_game.gold);

    Ok(())
}
```

## Error Types

Here's a complete reference of what can go wrong:

```rust
pub enum RuntimeError {
    /// Tried to navigate to a passage that doesn't exist
    UnknownPassage(String),

    /// No current passage (shouldn't happen in normal use)
    NoCurrentPassage,

    /// Choice index was out of bounds
    InvalidChoiceIndex {
        index: usize,
        available: usize,
    },

    /// Tried to select a choice whose condition wasn't met
    ConditionNotMet,

    /// A call effect returned an error
    EffectError(String),

    /// Tried to interact with an ended scene
    SceneEnded,
}
```

## Tips for Integration

### Reuse Parsed Scenes

Parsing is relatively expensive. Parse once, run many times:

```rust
// Good: parse once
let scene = parse(&source, "scene.scene")?;
for _ in 0..10 {
    let runtime = Runtime::new(&scene, Game::new());
    // ...
}

// Bad: parsing in a loop
for _ in 0..10 {
    let scene = parse(&source, "scene.scene")?;  // Wasteful!
    let runtime = Runtime::new(&scene, Game::new());
    // ...
}
```

### Keep Handlers Lightweight

The `get_var` and `set_var` methods get called frequently. Keep them fast:

```rust
// Good: direct field access and HashMap lookup
fn get_var(&self, name: &str) -> Value {
    match name {
        "health" => Value::Int(self.health),
        _ => self.flags.get(name).cloned().unwrap_or(Value::Null)
    }
}

// Bad: expensive computation on every access
fn get_var(&self, name: &str) -> Value {
    // Don't do database queries here!
    self.database.query(name)
}
```

### Handle Navigation Errors Gracefully

If your scene has a typo in a passage name, you'll get `UnknownPassage`. Consider logging these for debugging:

```rust
Err(RuntimeError::UnknownPassage(name)) => {
    eprintln!("BUG: Scene tried to navigate to '{}' which doesn't exist", name);
    // Maybe fall back to ending the scene?
}
```

## Summary

The runtime is your interface to interactive narratives:

- Create with `Runtime::new(&scene, handler)`
- Check state with `is_ended()` and `state()`
- Get content with `current_prose()` and `current_choices()`
- Drive interaction with `select_choice(index)`
- Access game state with `handler()`, `handler_mut()`, `into_handler()`

Next up: [Examples](06-examples.md) — see complete working games that tie everything together.
