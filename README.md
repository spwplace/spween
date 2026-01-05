# spween

A game-agnostic DSL for narrative/choice-based content in Rust.

spween lets you write branching narratives in a simple markup language, then execute them in your game with full control over how conditions are evaluated and effects are applied.

## Quick Example

```
---
id: tavern_encounter
title: The Mysterious Stranger
weight: 10
---

=== intro

A hooded figure sits alone in the corner of the tavern.
They gesture for you to approach.

* [Approach cautiously] { perception >= 12 }
  ~ perception_used = true
  -> cautious

* [Approach boldly]
  -> bold

* [Ignore them]
  -> END

=== cautious

You notice a glint of steel beneath their cloak. A weapon,
but kept hidden. They mean no immediate harm.

* [Sit down]
  ~ stranger_trust += 1
  -> conversation

=== bold

You stride over confidently. The stranger seems amused.

* [Sit down]
  -> conversation
```

## Features

- **Simple DSL**: Write scenes in an intuitive markup format
- **Game-agnostic**: Define your own variables, tags, and effects
- **Conditions**: Gate choices on game state (`{ gold >= 100 }`)
- **Effects**: Modify state when choices are selected (`~ gold -= 50`)
- **Custom calls**: Hook into your game logic (`~ play_sound "dramatic"`)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
spween = "0.1"
```

## Basic Usage

```rust
use spween::{parse, Runtime, EffectHandler, Value};
use std::collections::HashMap;

// 1. Implement EffectHandler for your game state
struct MyGame {
    vars: HashMap<String, Value>,
}

impl EffectHandler for MyGame {
    fn get_var(&self, name: &str) -> Value {
        self.vars.get(name).cloned().unwrap_or(Value::Null)
    }

    fn set_var(&mut self, name: &str, value: Value) {
        self.vars.insert(name.to_string(), value);
    }

    fn has(&self, category: &str, key: &str) -> bool {
        // Check if player has item, skill, etc.
        false
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
        // Handle custom effects like "play_sound", "spawn_enemy", etc.
        Ok(())
    }
}

// 2. Parse a scene
let source = std::fs::read_to_string("tavern.scene")?;
let scene = parse(&source, "tavern.scene")?;

// 3. Run it
let game = MyGame { vars: HashMap::new() };
let mut runtime = Runtime::new(&scene, game);

// 4. Get current prose and choices
println!("{}", runtime.current_prose().unwrap());
for choice in runtime.available_choices() {
    println!("  [{}] {}", choice.index, choice.text);
}

// 5. Select a choice
runtime.select_choice(0)?;
```

## Documentation

- [Getting Started](docs/01-getting-started.md) - First steps with spween
- [DSL Syntax](docs/02-dsl-syntax.md) - Complete language reference
- [Conditions](docs/03-conditions.md) - Conditional choice visibility
- [Effects](docs/04-effects.md) - Modifying game state
- [Runtime API](docs/05-runtime.md) - Executing scenes
- [Examples](docs/06-examples.md) - Complete working examples

## License

MIT
