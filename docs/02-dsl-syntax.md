# DSL Syntax Reference

This guide covers everything you can write in a spween scene file. Think of it as your complete reference—bookmark it and come back whenever you need to look something up.

## The Shape of a Scene File

Every spween scene has the same basic structure:

```
---
[frontmatter - metadata about the scene]
---

[passages - the actual content]
```

The frontmatter tells spween *about* your scene (its ID, title, requirements). The passages contain the actual narrative content—the prose, choices, and effects that make up your interactive story.

Let's explore each part in detail.

## Frontmatter

The frontmatter section sits between two `---` delimiters at the very top of your file. It uses YAML format for structured metadata.

### The Required Fields

Every scene needs at least these two fields:

```yaml
---
id: unique_scene_id
title: Human Readable Title
---
```

| Field | What It's For |
|-------|---------------|
| `id` | A unique identifier used by your code to reference this scene. Use `snake_case`—no spaces, just lowercase letters, numbers, and underscores. |
| `title` | A nice name for humans to read. This might appear in scene selection menus or save files. Spaces and special characters are fine here. |

### Optional Fields

You can add more metadata to control how your scene behaves:

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

| Field | Default | What It Does |
|-------|---------|--------------|
| `tags` | `[]` | Categories for filtering scenes. Your code can say "give me a random scene with the `tavern` tag." |
| `weight` | `10` | Selection probability when randomly choosing between scenes. Higher = more likely to be picked. |
| `cooldown` | `5` | How many "turns" (however you define them) before this scene can repeat. Prevents the same scene from appearing too often. |
| `requires` | none | Preconditions that must be met before this scene can be selected. See [Conditions](03-conditions.md) for the full syntax. |

### Custom Fields

Need to store extra information? Any YAML fields beyond the standard ones get preserved in `scene.meta.custom`:

```yaml
---
id: my_scene
title: My Scene
author: Jane Doe
difficulty: hard
mood: tense
---
```

In your code:
```rust
for (key, value) in &scene.meta.custom {
    println!("{}: {}", key, value);
    // Prints: author: Jane Doe, difficulty: hard, mood: tense
}
```

This is handy for editor tooling, debugging, or any game-specific metadata you want to track.

## Passages

After the frontmatter comes the heart of your scene: passages. Each passage is a named section that can contain prose, choices, and effects.

### Creating a Passage

```
=== passage_name

Content goes here...
```

The `===` marker starts a new passage. The name that follows must be a valid identifier—letters, numbers, and underscores, starting with a letter or underscore.

**Important:** The first passage in your file is the entry point. When a player starts the scene, they begin there. By convention, most scene authors name it `intro`, but you can call it whatever you like.

### What Goes in a Passage

A passage can contain:
- **Prose** — Narrative text shown to the player
- **Choices** — Decision points with `*`
- **Effects** — State changes with `~`
- **Navigation** — Jumps with `->`
- **Comments** — Notes to yourself with `//`

Let's look at each one.

## Prose

Any text that isn't a special marker becomes prose—the narrative content your player reads:

```
=== intro

This is prose. It's the story text, the dialogue, the descriptions—
everything the player reads.

You can have multiple paragraphs. Blank lines between them are
preserved in the output.

Numbers like 42 and punctuation work just fine!
Even "quoted text" appears as you'd expect.
```

Prose is straightforward: write what you want the player to see.

## Comments

Sometimes you want to leave notes for yourself (or other writers) that won't appear in the game:

```
=== intro

// This is a comment - players never see it
The player sees this text.

// TODO: add more atmospheric description here
// NOTE: this passage leads to the boss fight
```

Lines starting with `//` are ignored during parsing. Use them freely to annotate your scenes.

## Choices

Choices are what make your narrative interactive. They give players agency—a chance to shape the story.

### Basic Choice Syntax

```
* [Choice text goes here]
  -> target_passage
```

- The `*` marks this line as a choice
- Text in `[brackets]` is what the player sees
- The `-> target` line (indented underneath) specifies where to go when selected

### Multiple Choices

Most passages offer several options:

```
=== intro

You stand at a crossroads. Three paths stretch before you.

* [Take the northern path]
  -> north

* [Take the eastern path]
  -> east

* [Take the southern path]
  -> south

* [Sit down and rest]
  -> rest
```

Players will see all choices and pick one.

### Choices with Effects

Choices become more interesting when they *do* things:

```
* [Buy the sword]
  ~ gold -= 50
  ~ inventory_sword = true
  -> shop_complete
```

Here, selecting "Buy the sword" subtracts 50 gold, sets a flag, and then navigates to the next passage. Effects (the `~` lines) execute before navigation.

We'll cover effects in detail in [Effects](04-effects.md).

### Conditional Choices

Some choices should only appear (or be selectable) under certain circumstances:

```
* [Use your lockpick] { skills.lockpicking }
  -> unlocked

* [Force the door open] { strength >= 15 }
  ~ door_broken = true
  -> forced_open

* [Look for another way in]
  -> search_around
```

The `{ condition }` syntax specifies requirements. The first choice only appears if the player has the lockpicking skill. The second requires strength of at least 15.

There's also a `when` keyword syntax that some authors prefer:

```
* [Force the door open] when strength >= 15
  -> forced_open
```

Both forms work identically. See [Conditions](03-conditions.md) for the full condition syntax.

### Choices Without Navigation

If a choice doesn't specify a `->` target, the scene ends after selecting it:

```
* [Walk away forever]
  ~ left_early = true
  // No -> here, so the scene ends
```

## Navigation

Navigation controls the flow between passages.

### Jumping to a Passage

```
-> passage_name
```

This takes the player to the named passage. You can use it inside choices (the common case) or on its own in a passage.

### Ending the Scene

```
-> END
```

`END` is a special target that finishes the scene. The runtime's `is_ended()` method will return `true`.

### Navigation Placement

Navigation is typically the last thing in a choice:

```
* [Enter the mysterious cave]
  ~ explored_cave = true    // Effect 1
  ~ torches -= 1            // Effect 2
  -> cave_entrance          // Then navigate
```

Effects run first, in order, then navigation happens.

## Indentation

Indentation groups effects and navigation under their choice. Use either 2 spaces or 1 tab—just be consistent within a file:

```
* [This is the choice text]
  ~ effect_one = true
  ~ effect_two = true
  -> target_passage
```

The indented lines belong to the choice above them.

## A Complete Example

Let's put it all together with a realistic scene:

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

A merchant's wagon blocks the road ahead. Colorful fabrics and
glinting trinkets catch your eye. The driver notices your interest
and waves with a friendly smile.

"Fine goods for sale, traveler! Care to take a look?"

* [Browse the wares]
  -> browse

* [Ask about the road ahead]
  -> ask_road

* [Continue on your way]
  ~ merchant_ignored = true
  -> END

=== browse

The merchant spreads out their goods with practiced flair:
- Health potion: 25 gold
- Map fragment: 50 gold
- Lucky charm: 100 gold

// Note: these choices only appear if player can afford them
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

* [Nothing catches my eye]
  -> END

=== bought

"Pleasure doing business!" The merchant tips their hat with a grin.

* [Continue on your way]
  -> END

=== ask_road

"Bandits about three miles north. Nasty bunch." The merchant lowers
their voice. "But if you stick to the forest path, you can avoid them
entirely. Just watch for the old oak—turn east there."

* [Thank them and leave]
  ~ knows_bandit_location = true
  -> END

* [Maybe I'll browse those wares first]
  -> browse
```

This scene demonstrates:
- Frontmatter with tags, weight, cooldown, and requirements
- Multiple passages with different purposes
- Conditional choices that check player gold
- Effects that modify game state
- Navigation between passages and to END
- Comments explaining the author's intent

## Quick Reference

```
---                           # Start frontmatter
id: scene_id                  # Required: unique identifier
title: Display Name           # Required: human-readable name
tags: [tag1, tag2]            # Optional: categories
weight: 10                    # Optional: selection probability
cooldown: 5                   # Optional: turns before repeat
requires:                     # Optional: preconditions
  min: { var: value }
  has: [category.key]
---                           # End frontmatter

=== passage_name              # Start a passage

Prose text here.              # Narrative content

// This is a comment          # Not shown to players

* [Choice text]               # A choice
  ~ variable = value          # Effect (optional, can have multiple)
  -> target                   # Navigation (optional)

* [Another choice] { cond }   # Conditional choice
  -> somewhere

-> passage_name               # Direct navigation
-> END                        # End the scene
```

You now know everything about spween's syntax. Next, let's dive deeper into [Conditions](03-conditions.md) to make your choices truly dynamic.
