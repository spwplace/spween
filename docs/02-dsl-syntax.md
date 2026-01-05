# DSL Syntax Reference

Complete reference for the spween scene file format.

## File Structure

A spween scene file has two parts:

```
---
[frontmatter]
---

[passages]
```

## Frontmatter

YAML metadata between `---` delimiters.

### Required Fields

```yaml
---
id: unique_scene_id
title: Human Readable Title
---
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique identifier, used for scene selection |
| `title` | string | Display name for the scene |

### Optional Fields

```yaml
---
id: my_scene
title: My Scene
tags: [combat, main_quest]
weight: 15
cooldown: 3
requires:
  min: { gold: 100 }
  has: [inventory.sword]
---
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `tags` | list | `[]` | Categories for filtering scenes |
| `weight` | int | `10` | Selection probability weight |
| `cooldown` | int | `5` | Minimum turns before scene can repeat |
| `requires` | object | none | Preconditions (see [Conditions](03-conditions.md)) |

### Custom Fields

Any additional YAML fields are stored in `scene.meta.custom`:

```yaml
---
id: my_scene
title: My Scene
author: Jane Doe
difficulty: hard
---
```

Access in code:
```rust
for (key, value) in &scene.meta.custom {
    println!("{}: {}", key, value);
}
```

## Passages

Passages are named sections containing prose and choices.

### Basic Passage

```
=== passage_name

Prose content goes here. This can span
multiple lines and paragraphs.

More prose after a blank line.
```

- Passage names must be valid identifiers: `[a-zA-Z_][a-zA-Z0-9_]*`
- The first passage is the entry point
- Common convention: name the first passage `intro`

### Prose

Any text that isn't a choice, effect, or navigation becomes prose:

```
=== intro

This is prose. It's the narrative text shown to the player.

You can have multiple paragraphs. Blank lines are preserved
in the output.

Numbers like 42 and punctuation work fine!
```

### Comments

Lines starting with `//` are comments:

```
=== intro

// This won't appear in the output
The player sees this.

// TODO: add more flavor text
```

## Choices

Choices let the player make decisions.

### Basic Choice

```
* [Choice text]
  -> target_passage
```

- `*` marks a choice
- `[text]` is displayed to the player
- `-> target` navigates on selection

### Multiple Choices

```
=== intro

What do you do?

* [Go north]
  -> north

* [Go south]
  -> south

* [Stay here]
  -> END
```

### Choice with Effects

```
* [Buy the sword]
  ~ gold -= 50
  ~ inventory_sword = true
  -> shop_complete
```

Effects execute before navigation. See [Effects](04-effects.md).

### Conditional Choices

```
* [Use lockpick] { skills.lockpicking }
  -> unlocked

* [Force the door] when strength >= 15
  ~ door_broken = true
  -> forced
```

Two syntax options:
- `{ condition }` - Brace syntax
- `when condition` - Keyword syntax

See [Conditions](03-conditions.md).

### Choice without Navigation

If no `->` is specified, the scene ends after the choice:

```
* [Leave]
  ~ left_early = true
  // Scene ends here
```

## Navigation

### Jump to Passage

```
-> passage_name
```

### End Scene

```
-> END
```

`END` is a special target that ends the scene.

### Navigation in Choices

Navigation is typically the last line of a choice:

```
* [Enter the cave]
  ~ explored_cave = true
  ~ torches -= 1
  -> cave_entrance
```

## Indentation

Effects and navigation under a choice should be indented:

```
* [Choice text]
  ~ effect_one = true
  ~ effect_two = true
  -> target
```

Indentation uses 2 spaces or 1 tab. Consistency within a file is recommended.

## Complete Example

```
---
id: merchant_encounter
title: The Traveling Merchant
tags: [random, commerce]
weight: 10
cooldown: 5
requires:
  min: { gold: 10 }
---

=== intro

A merchant's wagon blocks the road ahead. The driver waves
as you approach.

"Fine goods for sale! Take a look?"

* [Browse the wares]
  -> browse

* [Ask about the road ahead]
  -> ask_road

* [Continue on your way]
  ~ merchant_ignored = true
  -> END

=== browse

The merchant displays their goods:
- Health potion: 25 gold
- Map fragment: 50 gold
- Lucky charm: 100 gold

* [Buy health potion] { gold >= 25 }
  ~ gold -= 25
  ~ has_potion = true
  -> bought

* [Buy map fragment] { gold >= 50 }
  ~ gold -= 50
  ~ has_map = true
  -> bought

* [Buy lucky charm] { gold >= 100 }
  ~ gold -= 100
  ~ luck += 1
  -> bought

* [Nothing, thanks]
  -> END

=== bought

"Pleasure doing business!" The merchant tips their hat.

* [Continue]
  -> END

=== ask_road

"Bandits about three miles north. Stick to the forest
path if you want to avoid them."

* [Thank them and leave]
  ~ knows_bandit_location = true
  -> END

* [Browse wares]
  -> browse
```
