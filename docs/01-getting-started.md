# Getting Started with spween

This guide walks you through creating your first interactive scene with spween.

## What is spween?

spween is a Domain-Specific Language (DSL) for writing branching narratives. Think of it as a simple scripting language for interactive fiction, dialogue trees, or any choice-based content in games.

**spween handles:**
- Parsing scene files into a structured format
- Managing passage navigation
- Evaluating conditions on choices
- Executing effects when choices are selected

**You handle:**
- Storing and retrieving game state (variables, inventory, etc.)
- Implementing custom effects (play sounds, spawn enemies, etc.)
- Rendering the text and choices to your UI

## Installation

Add spween to your `Cargo.toml`:

```toml
[dependencies]
spween = "0.1"
```

## Your First Scene

Create a file called `hello.scene`:

```
---
id: hello_world
title: Hello World
weight: 10
---

=== intro

Welcome to spween! This is your first interactive scene.

* [Say hello]
  -> greeting

* [Stay silent]
  -> silent

=== greeting

"Hello!" you say cheerfully.

The world seems a little brighter.

* [Continue]
  -> END

=== silent

You remain quiet, observing your surroundings.

* [Continue]
  -> END
```

Let's break this down:

### Frontmatter

```
---
id: hello_world
title: Hello World
weight: 10
---
```

The section between `---` markers is YAML frontmatter containing metadata:
- `id`: Unique identifier for this scene
- `title`: Human-readable name
- `weight`: Selection weight (higher = more likely when randomly choosing scenes)

### Passages

```
=== intro
```

Passages are named sections starting with `===`. The first passage is where the scene begins.

### Prose

```
Welcome to spween! This is your first interactive scene.
```

Plain text becomes prose - the narrative content shown to the player.

### Choices

```
* [Say hello]
  -> greeting
```

Lines starting with `*` are choices. The text in `[brackets]` is what the player sees. The `->` indicates where to navigate when selected.

### Navigation

- `-> passage_name` - Jump to another passage
- `-> END` - End the scene

## Running Your Scene

```rust
use spween::{parse, Runtime, EffectHandler, Value};

// Minimal game state - we'll expand this later
struct Game;

impl EffectHandler for Game {
    fn get_var(&self, _name: &str) -> Value { Value::Null }
    fn set_var(&mut self, _name: &str, _value: Value) {}
    fn has(&self, _category: &str, _key: &str) -> bool { false }
    fn call(&mut self, _name: &str, _args: &[Value]) -> Result<(), String> { Ok(()) }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and parse
    let source = std::fs::read_to_string("hello.scene")?;
    let scene = parse(&source, "hello.scene")?;

    // Create runtime
    let mut runtime = Runtime::new(&scene, Game);

    // Game loop
    while !runtime.is_ended() {
        // Show current prose
        if let Some(prose) = runtime.current_prose() {
            println!("\n{}\n", prose);
        }

        // Show choices
        let choices = runtime.available_choices();
        for choice in &choices {
            println!("  {}. {}", choice.index + 1, choice.text);
        }

        // Get player input (simplified)
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let choice_num: usize = input.trim().parse().unwrap_or(1);

        // Select choice (0-indexed)
        if let Err(e) = runtime.select_choice(choice_num - 1) {
            println!("Invalid choice: {}", e);
        }
    }

    println!("\n[Scene ended]");
    Ok(())
}
```

## What's Next?

Now that you have a basic scene running:

1. **[DSL Syntax](02-dsl-syntax.md)** - Learn the full syntax
2. **[Conditions](03-conditions.md)** - Make choices conditional
3. **[Effects](04-effects.md)** - Modify game state
4. **[Runtime API](05-runtime.md)** - Advanced runtime control

## Quick Reference

```
---                     # Start frontmatter
id: scene_id            # Required: unique identifier
title: Scene Title      # Required: display name
weight: 10              # Optional: selection weight (default: 10)
cooldown: 5             # Optional: cooldown turns (default: 5)
---                     # End frontmatter

=== passage_name        # Start a passage

Prose text here.        # Narrative content

* [Choice text]         # A choice
  -> target             # Navigation

* [Another choice]      # Multiple choices allowed
  -> END                # End the scene
```
