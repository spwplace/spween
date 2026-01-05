# Getting Started with spween

Welcome! You're about to create your first interactive scene. By the end of this guide, you'll have a working text adventure running in your terminal—and you'll understand exactly how it works.

## What is spween?

spween is a Domain-Specific Language (DSL) for writing branching narratives. If you've ever played a choose-your-own-adventure book, a visual novel, or a dialogue-heavy RPG, you've experienced what spween helps you create.

Think of spween as a specialized mini-language designed for one thing: interactive stories. It handles all the fiddly parts—parsing your scene files, managing which passage you're in, checking if choices are available, executing effects—so you can focus on the creative work.

**What spween gives you:**
- A clean syntax for writing scenes with choices and consequences
- Automatic parsing into a structured format your code can use
- Built-in support for conditions ("only show this choice if the player has 50 gold")
- Built-in support for effects ("when they pick this choice, subtract 10 health")
- A runtime that manages navigation between passages

**What you provide:**
- Your game state (variables, inventory, quest flags, etc.)
- How to respond to effects (what does "play_sound" actually do in your game?)
- The UI (how do you display prose and choices to the player?)

This separation is intentional—spween handles the narrative logic while staying completely agnostic about your game engine, rendering system, or data storage.

## Installation

Add spween to your `Cargo.toml`:

```toml
[dependencies]
spween = "0.1"
```

## Your First Scene

Let's create something simple but complete. Create a file called `hello.scene`:

```
---
id: hello_world
title: Hello World
weight: 10
---

=== intro

Welcome to spween! This is your first interactive scene.

What would you like to do?

* [Say hello]
  -> greeting

* [Stay silent]
  -> silent

=== greeting

"Hello!" you say cheerfully.

The world seems a little brighter somehow.

* [Continue]
  -> END

=== silent

You remain quiet, observing your surroundings in thoughtful silence.

Sometimes words aren't necessary.

* [Continue]
  -> END
```

Don't worry if this looks unfamiliar—let's break it down piece by piece.

### The Frontmatter

```
---
id: hello_world
title: Hello World
weight: 10
---
```

The section between `---` markers is called *frontmatter*. It's YAML-formatted metadata about your scene:

- **`id`**: A unique identifier for this scene. Your code uses this to reference the scene.
- **`title`**: A human-readable name. You might display this in a scene selection screen.
- **`weight`**: Used for weighted random selection when multiple scenes are available. Higher numbers = more likely to be chosen.

### Passages

```
=== intro
```

Lines starting with `===` mark the beginning of a *passage*—a named section of your scene. The first passage (`intro` in this case) is where the scene begins when a player starts it.

Think of passages like scenes in a play or chapters in a book. Each one has a name and contains content that gets shown to the player.

### Prose

```
Welcome to spween! This is your first interactive scene.

What would you like to do?
```

Plain text becomes *prose*—the narrative content that gets displayed to the player. You can have multiple paragraphs; blank lines are preserved.

### Choices

```
* [Say hello]
  -> greeting
```

Lines starting with `*` are *choices*—decision points where the player picks what happens next.

- The text in `[brackets]` is what the player sees as the option
- The `->` arrow indicates navigation: when the player selects this choice, go to the `greeting` passage

### Navigation

- `-> passage_name` jumps to that passage
- `-> END` is special—it ends the scene entirely

## Running Your Scene

Here's the Rust code to bring your scene to life:

```rust
use spween::{parse, Runtime, EffectHandler, Value};
use std::io::{self, Write};

// Our game state—minimal for now, we'll expand it later
struct Game;

impl EffectHandler for Game {
    fn get_var(&self, _name: &str) -> Value {
        Value::Null
    }

    fn set_var(&mut self, _name: &str, _value: Value) {
        // We're not tracking any variables yet
    }

    fn has(&self, _category: &str, _key: &str) -> bool {
        false
    }

    fn call(&mut self, _name: &str, _args: &[Value]) -> Result<(), String> {
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and parse the scene file
    let source = std::fs::read_to_string("hello.scene")?;
    let scene = parse(&source, "hello.scene")?;

    // Create the runtime with our game state
    let mut runtime = Runtime::new(&scene, Game);

    println!("=== {} ===\n", scene.meta.title);

    // The game loop
    while !runtime.is_ended() {
        // Display the current passage's prose
        if let Some(prose) = runtime.current_prose() {
            println!("{}\n", prose);
        }

        // Display available choices
        let choices = runtime.available_choices();
        for choice in &choices {
            println!("  {}. {}", choice.index + 1, choice.text);
        }

        // Get player input
        print!("\n> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // Parse input and select choice
        if let Ok(num) = input.trim().parse::<usize>() {
            if num > 0 && num <= choices.len() {
                if let Err(e) = runtime.select_choice(num - 1) {
                    println!("Error: {}\n", e);
                }
            }
        }

        println!();
    }

    println!("[Scene ended]");
    Ok(())
}
```

Run it with `cargo run`, and you'll see something like:

```
=== Hello World ===

Welcome to spween! This is your first interactive scene.

What would you like to do?

  1. Say hello
  2. Stay silent

> 1

"Hello!" you say cheerfully.

The world seems a little brighter somehow.

  1. Continue

> 1

[Scene ended]
```

Congratulations! You just built an interactive narrative.

## Understanding the EffectHandler

You might be wondering about that `EffectHandler` trait we implemented. This is how spween talks to your game.

When spween needs to:
- **Check a variable's value** (for conditions like `{ gold >= 50 }`), it calls `get_var()`
- **Change a variable** (for effects like `~ gold -= 10`), it calls `set_var()`
- **Check if something exists** (for conditions like `{ inventory.sword }`), it calls `has()`
- **Run a custom command** (for effects like `~ play_sound "victory"`), it calls `call()`

Our minimal implementation above doesn't track any state, which is fine for a simple scene with no conditions or effects. In real games, you'd connect these methods to your actual game state.

## What's Next?

Now that you have the basics working, you're ready to explore more:

1. **[DSL Syntax](02-dsl-syntax.md)** — Learn the full syntax: frontmatter options, prose formatting, all the ways to write choices
2. **[Conditions](03-conditions.md)** — Make choices appear only when certain conditions are met
3. **[Effects](04-effects.md)** — Modify game state when choices are selected
4. **[Runtime API](05-runtime.md)** — Advanced runtime features and integration patterns
5. **[Examples](06-examples.md)** — Complete working examples you can learn from

## Quick Reference

Here's a cheat sheet for everything we covered:

```
---                     # Start frontmatter
id: scene_id            # Required: unique identifier
title: Scene Title      # Required: display name
weight: 10              # Optional: selection weight (default: 10)
---                     # End frontmatter

=== passage_name        # Start a passage (first one is entry point)

Prose text here.        # Narrative content shown to player
More prose.             # Blank lines preserved

* [Choice text]         # A choice the player can select
  -> target_passage     # Where to go when selected

* [Another choice]      # Multiple choices allowed
  -> END                # Special target that ends the scene
```

You now have everything you need to start creating interactive stories. Have fun!
