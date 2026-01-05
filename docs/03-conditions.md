# Conditions

Conditions control when choices are available to the player.

## Overview

A choice can have an optional condition. When the condition is false, the choice is either hidden or shown as unavailable (your UI decides).

```
* [Visible choice]
  -> target

* [Conditional choice] { gold >= 100 }
  -> expensive_option
```

## Syntax Options

### Brace Syntax

```
* [Choice text] { condition }
```

Multiple conditions (AND):
```
* [Choice text] { gold >= 100, reputation >= 5 }
```

### When Syntax

```
* [Choice text] when condition
```

Multiple conditions:
```
* [Choice text] when gold >= 100, reputation >= 5
```

Both syntaxes are equivalent. Use whichever you prefer.

## Condition Types

### Variable Comparison

Compare a variable against a value:

```
{ gold >= 100 }      // Greater than or equal
{ health > 0 }       // Greater than
{ level <= 10 }      // Less than or equal
{ attempts < 3 }     // Less than
{ faction == 1 }     // Equal
{ status != 0 }      // Not equal
```

**Operators:**
| Operator | Meaning |
|----------|---------|
| `>=` | Greater than or equal |
| `>` | Greater than |
| `<=` | Less than or equal |
| `<` | Less than |
| `==` | Equal |
| `!=` | Not equal |

**Values:**
- Integers: `100`, `-5`, `0`
- Floats: `3.14`, `0.5`
- Booleans: `true`, `false`
- Strings: `"active"`, `"complete"`

### Has Condition

Check if a category contains a key:

```
{ inventory.sword }      // Has sword in inventory
{ skills.lockpicking }   // Has lockpicking skill
{ flags.door_unlocked }  // Flag is set
```

Format: `category.key`

This calls your `EffectHandler::has(category, key)` method.

### Truthy Check

A bare identifier checks if the variable is "truthy":

```
{ visited_town }         // True if visited_town is truthy
{ quest_complete }       // True if quest_complete is truthy
```

"Truthy" means:
- `true` for booleans
- Non-zero for numbers
- Non-empty for strings
- Always false for `null`

### Negation

Prefix with `!` to negate:

```
{ !inventory.sword }     // Does NOT have sword
{ !visited_town }        // Has NOT visited town
{ !locked }              // locked is falsy
```

## Multiple Conditions

Conditions are AND-ed together. All must be true.

```
{ gold >= 100, inventory.key, !guards_alerted }
```

This choice is available when:
- `gold` is at least 100, AND
- Player has `key` in `inventory`, AND
- `guards_alerted` is falsy

## Frontmatter Requirements

Scenes can have preconditions in the frontmatter:

```yaml
---
id: high_level_quest
title: The Dragon's Lair
requires:
  min:
    level: 10
    gold: 500
  max:
    fear: 50
  has:
    - skills.dragon_lore
    - inventory.fire_resist
  flags:
    - met_sage
  not:
    - quest_failed
---
```

### Require Fields

| Field | Description |
|-------|-------------|
| `min` | Variables must be >= these values |
| `max` | Variables must be <= these values |
| `has` | Must have these category.key pairs |
| `flags` | These variables must be truthy |
| `not` | These variables must be falsy |

Check scene requirements in code:

```rust
if runtime.check_scene_requirements() {
    // Scene can be played
} else {
    // Requirements not met
}
```

## Implementing Conditions

Your `EffectHandler` implementation determines how conditions are evaluated:

```rust
impl EffectHandler for MyGame {
    fn get_var(&self, name: &str) -> Value {
        // Called for variable comparisons and truthy checks
        match name {
            "gold" => Value::Int(self.player.gold),
            "health" => Value::Int(self.player.health),
            "level" => Value::Int(self.player.level),
            _ => self.flags.get(name).cloned().unwrap_or(Value::Null)
        }
    }

    fn has(&self, category: &str, key: &str) -> bool {
        // Called for category.key conditions
        match category {
            "inventory" => self.inventory.contains(key),
            "skills" => self.skills.contains(key),
            "perks" => self.perks.contains(key),
            _ => false
        }
    }

    // ... other methods
}
```

## Runtime Behavior

The runtime provides choice availability info:

```rust
// Get all choices with availability
let choices = runtime.current_choices();
for choice in choices {
    if choice.available {
        println!("[{}] {}", choice.index, choice.text);
    } else {
        println!("[{}] {} (unavailable)", choice.index, choice.text);
    }
}

// Get only available choices
let available = runtime.available_choices();
```

Selecting an unavailable choice returns an error:

```rust
match runtime.select_choice(index) {
    Ok(()) => { /* success */ }
    Err(RuntimeError::ConditionNotMet) => {
        println!("That choice isn't available!");
    }
    Err(e) => { /* other error */ }
}
```

## Examples

### RPG Dialog

```
* [Intimidate them] { strength >= 15 }
  ~ guard_frightened = true
  -> intimidate_success

* [Persuade them] { charisma >= 12 }
  ~ guard_convinced = true
  -> persuade_success

* [Bribe them] { gold >= 50 }
  ~ gold -= 50
  -> bribe_success

* [Fight them]
  -> combat
```

### Quest Prerequisites

```
* [Enter the ancient tomb] { inventory.tomb_key, level >= 5 }
  -> tomb_entrance

* [Read the inscription] { skills.ancient_languages }
  ~ lore_discovered = true
  -> inscription
```

### Story Flags

```
* [Mention what you learned from the sage] { met_sage, !sage_secret_told }
  ~ sage_secret_told = true
  -> reveal_info

* [Ask about the prophecy] { prophecy_started, !prophecy_complete }
  -> prophecy_info
```

### Mutually Exclusive Paths

```
* [Side with the rebels] { !joined_empire }
  ~ joined_rebels = true
  -> rebel_path

* [Side with the empire] { !joined_rebels }
  ~ joined_empire = true
  -> empire_path
```
