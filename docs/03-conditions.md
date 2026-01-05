# Conditions

Conditions are what make your choices feel alive. Instead of showing every option to every player, you can make choices appear only when they make sense—when the player has enough gold, when they've learned a skill, when they haven't already burned that bridge.

This guide covers everything you need to know about writing and combining conditions.

## The Basic Idea

A choice can have an optional condition attached. When the condition is false, the choice either won't appear or will be shown as unavailable (you control this in your UI code).

```
* [This choice is always visible]
  -> somewhere

* [This choice requires gold] { gold >= 100 }
  -> expensive_option
```

The second choice only becomes available when the player has at least 100 gold. That's it—conditions let you gate content behind requirements.

## Two Ways to Write Conditions

spween offers two syntaxes for conditions. They're completely equivalent; use whichever feels more natural to you.

### Brace Syntax

```
* [Choice text] { condition }
```

Multiple conditions (all must be true):
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

Both syntaxes produce identical results. The brace syntax feels more "code-like," while the `when` syntax reads more like natural language. Pick your favorite.

## Types of Conditions

### Comparing Variables

The most common condition compares a variable against a value:

```
{ gold >= 100 }      // Do you have at least 100 gold?
{ health > 0 }       // Are you still alive?
{ level <= 10 }      // Are you level 10 or below?
{ attempts < 3 }     // Have you tried fewer than 3 times?
{ faction == 1 }     // Are you in faction 1?
{ status != 0 }      // Is your status anything other than 0?
```

Here's what each operator means:

| Operator | Meaning | Example |
|----------|---------|---------|
| `>=` | Greater than or equal to | `gold >= 50` — at least 50 gold |
| `>` | Greater than | `health > 0` — more than 0 health |
| `<=` | Less than or equal to | `level <= 10` — level 10 or below |
| `<` | Less than | `attempts < 3` — fewer than 3 attempts |
| `==` | Equals | `faction == 1` — exactly faction 1 |
| `!=` | Not equals | `status != 0` — any status except 0 |

You can compare against different types of values:
- **Integers**: `100`, `-5`, `0`
- **Floats**: `3.14`, `0.5`
- **Booleans**: `true`, `false`
- **Strings**: `"active"`, `"complete"`

### Checking for Items or Abilities

Sometimes you need to check if the player *has* something—an item in their inventory, a learned skill, a completed quest. That's what the "has" condition is for:

```
{ inventory.sword }      // Do you have a sword?
{ skills.lockpicking }   // Can you pick locks?
{ perks.night_vision }   // Do you have night vision?
{ quests.saved_village } // Have you saved the village?
```

The format is `category.key`. When spween sees this, it calls your `EffectHandler::has(category, key)` method, so you can make the categories mean whatever you want in your game.

### Truthy Checks

A bare identifier (just a variable name, no operator) checks if the value is "truthy":

```
{ visited_town }         // Have you visited the town?
{ quest_complete }       // Is the quest complete?
{ door_unlocked }        // Is the door unlocked?
```

What counts as "truthy"?
- `true` is truthy
- Any non-zero number is truthy (`1`, `100`, `-5`)
- Any non-empty string is truthy (`"hello"`)
- `false`, `0`, `""` (empty string), and `null` are all falsy

This is a convenient shorthand. Instead of writing `{ visited_town == true }`, you can just write `{ visited_town }`.

### Negating Conditions

Prefix any condition with `!` to flip it:

```
{ !inventory.sword }     // You DON'T have a sword
{ !visited_town }        // You HAVEN'T visited the town
{ !locked }              // The door is NOT locked
```

Negation is incredibly useful for one-time events or mutually exclusive paths:

```
* [Hear the sage's wisdom] { !heard_sage_wisdom }
  ~ heard_sage_wisdom = true
  -> sage_speaks

* [You've already heard this story.]
  -> END
```

## Combining Multiple Conditions

When you list multiple conditions, they're all ANDed together—every single one must be true:

```
{ gold >= 100, inventory.key, !guards_alerted }
```

This choice is available when:
1. You have at least 100 gold, **AND**
2. You have a key in your inventory, **AND**
3. The guards haven't been alerted

If any condition fails, the whole thing fails.

## Scene-Level Requirements

Besides choice conditions, scenes themselves can have requirements in their frontmatter. These determine whether a scene should even be offered to the player:

```yaml
---
id: dragon_lair
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

### Requirement Fields

| Field | What It Checks |
|-------|----------------|
| `min` | Variables must be **at least** these values |
| `max` | Variables must be **at most** these values |
| `has` | Player must have all these `category.key` items |
| `flags` | These variables must be truthy |
| `not` | These variables must be falsy |

In your code, you can check whether a scene's requirements are met:

```rust
if runtime.check_scene_requirements() {
    // Player qualifies for this scene
} else {
    // Skip it, pick another scene
}
```

This is useful when building a scene selection system that filters available scenes based on player state.

## Making Conditions Work: The EffectHandler

Conditions don't mean anything on their own—your game defines what they mean. When spween evaluates a condition, it calls methods on your `EffectHandler`:

```rust
impl EffectHandler for MyGame {
    fn get_var(&self, name: &str) -> Value {
        // Called for comparisons like { gold >= 50 }
        // and truthy checks like { visited }
        match name {
            "gold" => Value::Int(self.player.gold),
            "health" => Value::Int(self.player.health),
            "level" => Value::Int(self.player.level),
            _ => self.flags.get(name).cloned().unwrap_or(Value::Null)
        }
    }

    fn has(&self, category: &str, key: &str) -> bool {
        // Called for has-checks like { inventory.sword }
        match category {
            "inventory" => self.inventory.contains(key),
            "skills" => self.skills.contains(key),
            "perks" => self.perks.contains(key),
            "quests" => self.completed_quests.contains(key),
            _ => false
        }
    }

    // ... other methods
}
```

This design keeps spween completely agnostic about your game's data model. You might store inventory in a `HashSet`, a database, or a remote server—spween doesn't care. It just calls your methods and trusts your answers.

## Using Conditions in Your Game Loop

The runtime tells you which choices are available:

```rust
// Get ALL choices, with their availability status
let all_choices = runtime.current_choices();
for choice in all_choices {
    if choice.available {
        // Condition passed (or no condition)
        println!("[{}] {}", choice.index, choice.text);
    } else {
        // Condition failed
        println!("[{}] {} (locked)", choice.index, choice.text);
    }
}

// Or get ONLY available choices
let available = runtime.available_choices();
for choice in available {
    println!("[{}] {}", choice.index, choice.text);
}
```

You decide how to handle unavailable choices in your UI:
- Hide them entirely
- Show them grayed out
- Show them with a hint about what's needed

If a player somehow tries to select an unavailable choice:

```rust
match runtime.select_choice(index) {
    Ok(()) => {
        // Success!
    }
    Err(RuntimeError::ConditionNotMet) => {
        // They tried to pick a locked choice
        println!("That option isn't available right now.");
    }
    Err(e) => {
        // Some other error
    }
}
```

## Practical Examples

Let's look at some realistic uses of conditions.

### RPG Skill Checks

```
=== locked_door

The ornate door is sealed with an ancient mechanism.

* [Pick the lock] { skills.lockpicking }
  ~ picked_lock = true
  -> door_opened

* [Force it open] { strength >= 16 }
  ~ door_broken = true
  ~ make_noise
  -> door_opened

* [Use the skeleton key] { inventory.skeleton_key }
  ~ skeleton_key_uses -= 1
  -> door_opened

* [Look for another way]
  -> search_area
```

### Resource Gates

```
* [Buy health potion] { gold >= 25 }
  ~ gold -= 25
  ~ potions += 1
  -> shop

* [Rest at the inn] { gold >= 10 }
  ~ gold -= 10
  ~ health = 100
  ~ fatigue = 0
  -> morning

* [Leave]
  -> street
```

### Story Flags

```
* [Mention what the sage told you] { met_sage, !told_about_sage }
  ~ told_about_sage = true
  -> reveal_info

* [Ask about the prophecy] { prophecy_started, !prophecy_complete }
  -> prophecy_details
```

### Mutually Exclusive Paths

```
* [Side with the rebels] { !joined_empire }
  ~ joined_rebels = true
  -> rebel_path

* [Side with the empire] { !joined_rebels }
  ~ joined_empire = true
  -> empire_path

* [Remain neutral] { !joined_rebels, !joined_empire }
  -> neutral_path
```

### One-Time Events

```
* [Open the treasure chest] { !chest_opened }
  ~ chest_opened = true
  ~ gold += 500
  -> treasure_found

* [The chest is empty]
  -> room_description
```

## Summary

Conditions let you create dynamic, responsive narratives where player state shapes what's possible. Remember:

- Use `{ condition }` or `when condition` on choices
- Compare variables with `>=`, `>`, `<=`, `<`, `==`, `!=`
- Check for items/abilities with `category.key`
- Use bare identifiers for truthy checks
- Negate with `!`
- Combine multiple conditions with commas (they're ANDed)
- Implement `get_var()` and `has()` in your `EffectHandler` to make conditions work

Next up: [Effects](04-effects.md) — learn how to modify game state when choices are selected.
