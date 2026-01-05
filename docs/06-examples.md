# Examples

The best way to learn is by seeing complete, working code. This guide presents several examples that you can run, modify, and learn from. Each one demonstrates different aspects of spween while being a functional mini-game.

## Example 1: A Simple Dungeon Crawl

Let's start with a classic: exploring a dungeon, finding treasure, and maybe getting poisoned by mysterious mushrooms. This example shows the core loop of spween—prose, choices, effects, and branching paths.

### The Scene File

Create `dungeon.scene`:

```
---
id: dungeon_entrance
title: The Dark Dungeon
tags: [adventure, starter]
weight: 10
---

=== intro

You stand at the entrance to a dark dungeon. A cold wind blows from
within, carrying the smell of damp stone and ancient secrets.

Your torch flickers uncertainly.

* [Enter the dungeon]
  ~ entered_dungeon = true
  -> entrance_hall

* [This was a bad idea. Turn back.]
  ~ cowardice += 1
  -> END

=== entrance_hall

The entrance hall is lit by pale moonlight streaming through cracks
in the ceiling above. Dust motes dance in the silver beams.

Two passages lead deeper into the darkness. To the left, you hear
the faint sound of dripping water. To the right, a strange glow
pulses rhythmically.

* [Go left, toward the water]
  -> water_room

* [Go right, toward the glow]
  -> glowing_room

* [Search this room first]
  ~ gold += 5
  ~ searched_hall = true
  -> hall_searched

=== hall_searched

Running your hands along the rough stone walls, you find a loose
brick. Behind it: a small pouch containing a few tarnished coins.

Not much, but it's yours now.

* [Go left, toward the water]
  -> water_room

* [Go right, toward the glow]
  -> glowing_room

=== water_room

An underground stream flows through this chamber, cutting a channel
through the ancient stone floor. The water looks remarkably clear.

* [Drink from the stream]
  ~ health += 10
  ~ drank_water = true
  -> drank

* [Wade across and continue deeper]
  -> deep_passage

=== drank

You cup your hands and drink deeply. The water is ice-cold and
refreshing—you feel invigorated, your weariness washing away.

* [Continue deeper into the dungeon]
  -> deep_passage

=== glowing_room

The glow comes from luminescent mushrooms covering the walls in
patches of soft blue-green light. They're beautiful—almost hypnotic.

You've heard of such mushrooms in travelers' tales, but never seen
them yourself. Some are said to be healing. Others... less so.

* [Harvest some mushrooms] { !warned_about_mushrooms }
  ~ has_mushrooms = true
  ~ poisoned = true
  -> mushroom_harvest

* [Better not risk it. Continue on.]
  -> deep_passage

=== mushroom_harvest

You carefully pluck several of the largest specimens. They pulse
with an eerie inner light, warm in your hands.

// The player doesn't know they're poisoned yet...

* [Continue deeper into the dungeon]
  -> deep_passage

=== deep_passage

The passage narrows, forcing you to duck under hanging roots and
squeeze past jutting rocks. Then suddenly—it opens into a small
chamber.

And there, against the far wall, sits a chest.

* [Open the chest] { !poisoned }
  ~ gold += 100
  ~ found_treasure = true
  -> victory

* [Open the chest] { poisoned }
  ~ gold += 100
  ~ found_treasure = true
  ~ collapsed = true
  -> poison_ending

* [Something feels wrong. Leave now.]
  -> exit

=== victory

The chest lid creaks open to reveal a pile of gold coins, glittering
in the light of your torch. You've struck it rich!

Heart pounding with excitement, you gather your fortune and make
your way back through the dungeon. The path out seems shorter
somehow, as if the dungeon itself is satisfied.

You emerge into the fresh night air, richer and wiser.

* [Celebrate your success!]
  -> END

=== poison_ending

You throw open the chest and gasp at the treasure within—more gold
than you've ever seen!

But as you reach for it, the world tilts. Your hands tremble. The
mushrooms... they weren't the healing kind.

You collapse beside your treasure, the gold spilling through your
fingers like water. Your torch gutters and dies.

* [...]
  -> END

=== exit

Something in your gut tells you to leave. Maybe it's instinct. Maybe
it's wisdom. Either way, you retrace your steps and emerge from the
dungeon, blinking in the starlight.

You didn't find treasure, but you're alive. Sometimes that's enough.

* [Leave]
  -> END
```

### The Rust Code

```rust
use spween::{parse, Runtime, EffectHandler, Value};
use std::collections::HashMap;
use std::io::{self, Write};

struct GameState {
    health: i64,
    gold: i64,
    flags: HashMap<String, Value>,
}

impl GameState {
    fn new() -> Self {
        Self {
            health: 100,
            gold: 0,
            flags: HashMap::new(),
        }
    }
}

impl EffectHandler for GameState {
    fn get_var(&self, name: &str) -> Value {
        match name {
            "health" => Value::Int(self.health),
            "gold" => Value::Int(self.gold),
            _ => self.flags.get(name).cloned().unwrap_or(Value::Null),
        }
    }

    fn set_var(&mut self, name: &str, value: Value) {
        match name {
            "health" => {
                if let Some(v) = value.as_int() {
                    self.health = v.clamp(0, 100);
                }
            }
            "gold" => {
                if let Some(v) = value.as_int() {
                    self.gold = v.max(0);
                }
            }
            _ => {
                self.flags.insert(name.to_string(), value);
            }
        }
    }

    fn has(&self, _category: &str, _key: &str) -> bool {
        false
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
        // Log effects for debugging
        println!("[Effect: {} {:?}]", name, args);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = include_str!("dungeon.scene");
    let scene = parse(source, "dungeon.scene")?;

    let mut runtime = Runtime::new(&scene, GameState::new());

    println!("\n╔════════════════════════════════════════╗");
    println!("║  {}  ║", scene.meta.title);
    println!("╚════════════════════════════════════════╝\n");

    while !runtime.is_ended() {
        // Show stats
        {
            let state = runtime.handler();
            println!("─── Health: {} │ Gold: {} ───\n", state.health, state.gold);
        }

        // Show prose
        if let Some(prose) = runtime.current_prose() {
            println!("{}\n", prose);
        }

        // Show choices
        let choices = runtime.available_choices();
        if choices.is_empty() {
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

        if let Ok(num) = input.trim().parse::<usize>() {
            if num > 0 && num <= choices.len() {
                runtime.select_choice(num - 1)?;
                println!();
            }
        }
    }

    // Final stats
    let final_state = runtime.into_handler();
    println!("═══════════════════════════════════════");
    println!("  GAME OVER");
    println!("  Final Gold: {}", final_state.gold);
    println!("  Final Health: {}", final_state.health);
    println!("═══════════════════════════════════════\n");

    Ok(())
}
```

### What This Example Teaches

- **Basic scene structure**: Frontmatter, passages, prose, choices, navigation
- **Effects modifying state**: `~ gold += 5`, `~ health += 10`
- **Flags for tracking events**: `~ poisoned = true`
- **Conditional choices**: The chest opening has different outcomes based on the `poisoned` flag
- **Multiple paths through the same story**: Players can go left or right, drink or not, harvest or not
- **A simple game loop**: Display, input, select, repeat

---

## Example 2: The Blacksmith's Shop

This example shows off RPG-style dialogue with skill checks, inventory management, and quest tracking. It's more complex than the dungeon but teaches important patterns.

### The Scene File

Create `blacksmith.scene`:

```
---
id: blacksmith_dialogue
title: The Village Blacksmith
tags: [dialogue, shop, village]
weight: 10
requires:
  min:
    gold: 5
---

=== intro

The rhythmic clang of hammer on steel greets you as you push open
the smithy door. Heat washes over you from the forge.

A broad-shouldered man looks up from his work, wiping sweat from his
brow. His arms are thick with muscle, his apron scorched from
countless sparks.

"Welcome, traveler. What can I do for you?"

* [I'd like to see your weapons]
  -> weapons

* [Show me your armor]
  -> armor

* [Have you heard any news?]
  -> rumors

* [Just looking. I'll be going.]
  -> END

=== weapons

He gestures to a rack along the wall where blades of various sizes
hang ready for inspection. The steel gleams in the forge-light.

"All forged right here. No finer steel in the valley."

* [Buy iron sword - 50 gold] { gold >= 50 }
  ~ gold -= 50
  ~ add_item "iron_sword"
  -> bought_weapon

* [Buy steel sword - 150 gold] { gold >= 150 }
  ~ gold -= 150
  ~ add_item "steel_sword"
  -> bought_weapon

* [Ask about custom work] { skills.persuasion }
  -> custom_work

* [Maybe something else...]
  -> intro

=== armor

He leads you to the back wall where armor pieces hang on wooden
frames. Leather, chainmail, plate—a warrior's dream.

"Protection's worth more than gold when steel's coming at your head."

* [Buy leather armor - 30 gold] { gold >= 30 }
  ~ gold -= 30
  ~ add_item "leather_armor"
  ~ defense += 2
  -> bought_armor

* [Buy chainmail - 100 gold] { gold >= 100 }
  ~ gold -= 100
  ~ add_item "chainmail"
  ~ defense += 5
  -> bought_armor

* [Buy plate armor - 300 gold] { gold >= 300, strength >= 14 }
  ~ gold -= 300
  ~ add_item "plate_armor"
  ~ defense += 8
  -> bought_armor

* [Maybe something else...]
  -> intro

=== bought_weapon

"A fine choice!" He takes the weapon down and wraps it carefully in
oiled cloth. "May it serve you well."

He weighs your coin with a practiced eye and nods, satisfied.

* [Continue shopping]
  -> intro

* [That's all I need. Farewell.]
  -> END

=== bought_armor

"Smart purchase." He helps you check the fit, adjusting straps and
buckles. "That'll keep you breathing when things get ugly."

* [Continue shopping]
  -> intro

* [That's all I need. Farewell.]
  -> END

=== custom_work

His eyes sharpen with interest. You've clearly said the magic words.

"Ah, you know quality when you see it." He leans closer, lowering
his voice. "I can make something special—if you have the materials.
And the coin."

* [Commission a dragonscale blade - 500 gold] { gold >= 500, inventory.dragon_scale }
  ~ gold -= 500
  ~ remove_item "dragon_scale"
  ~ add_item "dragonscale_sword"
  ~ blacksmith_reputation += 10
  -> masterwork_complete

* [What materials do you need?]
  -> materials_info

* [Interesting, but not today]
  -> intro

=== materials_info

"For a truly legendary blade?" He strokes his beard thoughtfully.
"Something extraordinary. Dragon scale. Phoenix feather. Heartstone
from deep in the mountains."

He shrugs. "Bring me something special, and we'll talk about what
I can make from it."

* [I'll keep that in mind]
  -> intro

=== masterwork_complete

The smith works for what feels like hours, sweat pouring down his
face as he coaxes metal and scale into harmony. You watch, transfixed,
as something beautiful takes shape.

Finally, he presents the finished blade. It seems to shimmer with
inner fire, light dancing along its edge.

"My finest work," he says quietly. "Guard it well."

* [Thank him and leave]
  ~ quest_complete "masterwork_sword"
  -> END

=== rumors

He glances around, then leans on his anvil conspiratorially.

"Word from the miners—trouble in the old shaft. Strange sounds at
night. Some say goblins." He spits into the forge. "I say worse."

* [Tell me more about these goblins]
  ~ knows_mine_location = true
  -> mine_info

* [Any other work available?]
  -> other_work

* [Thanks for the warning]
  -> intro

=== mine_info

"East of town, past the split oak. Can't miss the entrance—big
hole in the hillside, cart tracks leading in."

His expression darkens. "If you're thinking of clearing them out...
I'd pay for proof. Ten gold per ear. Just don't get yourself killed."

* [I'll take that bounty]
  ~ quest_start "goblin_bounty"
  ~ bounty_accepted = true
  -> bounty_accepted

* [I'll pass on the monster hunting]
  -> intro

=== bounty_accepted

He grips your forearm in a warrior's clasp.

"Good luck. And don't forget—ten gold per ear. Bring them to me."

* [Time to hunt some goblins]
  -> END

* [First, let me see those weapons]
  -> weapons

=== other_work

"Check the notice board at the tavern. Always someone needing
something done." He returns to his hammer work, the conversation
clearly over.

* [Back to shopping]
  -> intro

* [I should be going]
  -> END
```

### The Rust Code

```rust
use spween::{parse, Runtime, EffectHandler, Value};
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

struct RPGGame {
    gold: i64,
    strength: i64,
    defense: i64,
    inventory: HashSet<String>,
    skills: HashSet<String>,
    flags: HashMap<String, Value>,
    quests: HashSet<String>,
}

impl RPGGame {
    fn new() -> Self {
        let mut game = Self {
            gold: 200,
            strength: 12,
            defense: 0,
            inventory: HashSet::new(),
            skills: HashSet::new(),
            flags: HashMap::new(),
            quests: HashSet::new(),
        };
        // Starting skills and items
        game.skills.insert("persuasion".to_string());
        game.inventory.insert("dragon_scale".to_string());
        game
    }
}

impl EffectHandler for RPGGame {
    fn get_var(&self, name: &str) -> Value {
        match name {
            "gold" => Value::Int(self.gold),
            "strength" => Value::Int(self.strength),
            "defense" => Value::Int(self.defense),
            _ => self.flags.get(name).cloned().unwrap_or(Value::Null),
        }
    }

    fn set_var(&mut self, name: &str, value: Value) {
        match name {
            "gold" => {
                if let Some(v) = value.as_int() {
                    self.gold = v;
                }
            }
            "strength" => {
                if let Some(v) = value.as_int() {
                    self.strength = v;
                }
            }
            "defense" => {
                if let Some(v) = value.as_int() {
                    self.defense = v;
                }
            }
            _ => {
                self.flags.insert(name.to_string(), value);
            }
        }
    }

    fn has(&self, category: &str, key: &str) -> bool {
        match category {
            "inventory" => self.inventory.contains(key),
            "skills" => self.skills.contains(key),
            "quests" => self.quests.contains(key),
            _ => false,
        }
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
        match name {
            "add_item" => {
                if let Some(item) = args.get(0).and_then(|v| v.as_str()) {
                    println!("\n  ✓ Acquired: {}", item);
                    self.inventory.insert(item.to_string());
                }
                Ok(())
            }
            "remove_item" => {
                if let Some(item) = args.get(0).and_then(|v| v.as_str()) {
                    self.inventory.remove(item);
                }
                Ok(())
            }
            "quest_start" => {
                if let Some(quest) = args.get(0).and_then(|v| v.as_str()) {
                    println!("\n  ★ Quest started: {}", quest);
                    self.quests.insert(quest.to_string());
                }
                Ok(())
            }
            "quest_complete" => {
                if let Some(quest) = args.get(0).and_then(|v| v.as_str()) {
                    println!("\n  ★ Quest complete: {}", quest);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = include_str!("blacksmith.scene");
    let scene = parse(source, "blacksmith.scene")?;

    let mut runtime = Runtime::new(&scene, RPGGame::new());

    println!("\n═══ {} ═══\n", scene.meta.title);

    while !runtime.is_ended() {
        // Show stats
        {
            let state = runtime.handler();
            println!("Gold: {} | Strength: {} | Defense: {}",
                state.gold, state.strength, state.defense);
            if !state.inventory.is_empty() {
                println!("Inventory: {:?}", state.inventory);
            }
            println!();
        }

        // Show prose
        if let Some(prose) = runtime.current_prose() {
            println!("{}\n", prose);
        }

        // Show choices
        let choices = runtime.available_choices();
        if choices.is_empty() {
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

        if let Ok(num) = input.trim().parse::<usize>() {
            if num > 0 && num <= choices.len() {
                runtime.select_choice(num - 1)?;
                println!();
            }
        }
    }

    println!("\n═══ Session Complete ═══\n");
    Ok(())
}
```

### What This Example Teaches

- **Skill checks**: The "custom work" option only appears if you have persuasion
- **Inventory management**: Using `add_item` and `remove_item` custom effects
- **Multiple requirements**: Plate armor requires both gold AND strength
- **Quest tracking**: Starting and completing quests with custom effects
- **Returning to previous nodes**: Several choices loop back to `intro`
- **Scene requirements in frontmatter**: The scene requires at least 5 gold

---

## Example 3: Building a Scene Selection System

Real games have multiple scenes. This example shows how to manage a library of scenes and select appropriate ones based on player state.

```rust
use spween::{parse, Runtime, EffectHandler, Value, Scene};
use rand::Rng;
use std::error::Error;

struct SceneManager {
    scenes: Vec<Scene>,
}

impl SceneManager {
    fn load_all(paths: &[&str]) -> Result<Self, Box<dyn Error>> {
        let mut scenes = Vec::new();
        for path in paths {
            let source = std::fs::read_to_string(path)?;
            let scene = parse(&source, path)?;
            scenes.push(scene);
        }
        Ok(Self { scenes })
    }

    /// Get scenes that match a tag and whose requirements are met
    fn available_scenes<'a, H: EffectHandler>(
        &'a self,
        handler: &H,
        tag: &str
    ) -> Vec<&'a Scene> {
        self.scenes.iter()
            .filter(|scene| {
                // Must have the requested tag
                scene.meta.tags.iter().any(|t| t == tag)
            })
            .filter(|scene| {
                // Must meet requirements (simplified check)
                // In a real implementation, you'd check scene.meta.requires
                true
            })
            .collect()
    }

    /// Randomly select a scene using weights
    fn select_random<'a, H: EffectHandler>(
        &'a self,
        handler: &H,
        tag: &str
    ) -> Option<&'a Scene> {
        let available = self.available_scenes(handler, tag);
        if available.is_empty() {
            return None;
        }

        // Calculate total weight
        let total_weight: u32 = available.iter()
            .map(|s| s.meta.weight)
            .sum();

        // Random selection
        let mut roll = rand::thread_rng().gen_range(0..total_weight);
        for scene in available {
            if roll < scene.meta.weight {
                return Some(scene);
            }
            roll -= scene.meta.weight;
        }

        available.last().copied()
    }
}

// Example usage in a game loop:
fn game_loop() -> Result<(), Box<dyn Error>> {
    // Load all scenes
    let manager = SceneManager::load_all(&[
        "scenes/tavern_brawl.scene",
        "scenes/merchant_encounter.scene",
        "scenes/mysterious_stranger.scene",
        "scenes/goblin_ambush.scene",
    ])?;

    let mut game = MyGame::new();

    // Game turn loop
    for turn in 0..10 {
        println!("\n=== Turn {} ===\n", turn + 1);

        // Select a random encounter for this turn
        let tag = if turn % 3 == 0 { "combat" } else { "social" };

        if let Some(scene) = manager.select_random(&game, tag) {
            println!("Starting scene: {}", scene.meta.title);

            let mut runtime = Runtime::new(scene, game);

            // Run the scene...
            while !runtime.is_ended() {
                // (normal game loop here)
            }

            // Get game state back
            game = runtime.into_handler();
        } else {
            println!("No {} scenes available", tag);
        }
    }

    Ok(())
}
```

### What This Example Teaches

- **Loading multiple scenes**: Building a scene library
- **Tag-based filtering**: Finding scenes appropriate for the situation
- **Weighted random selection**: Using scene weights for variety
- **Scene requirements**: Only offering scenes the player qualifies for
- **Passing state between scenes**: Using `into_handler()` to continue

---

## Tips for Writing Great Scenes

After building several examples, here are patterns that work well:

### 1. Start Strong

Your first passage sets the tone. Make it evocative:

```
// Good
=== intro
Rain hammers the cobblestones as you duck into the tavern's warmth.
The door slams behind you, cutting off the storm's fury.

// Less engaging
=== intro
You are in a tavern. It is raining outside.
```

### 2. Make Choices Feel Meaningful

Each choice should feel distinct. Avoid options that are just cosmetic:

```
// Good: different approaches with different consequences
* [Sneak past the guards] { skills.stealth }
  ~ sneaked_in = true
  -> inside_undetected

* [Bribe the guards] { gold >= 50 }
  ~ gold -= 50
  ~ guards_bribed = true
  -> inside_noticed

* [Fight your way in]
  ~ guards_hostile = true
  -> combat

// Less engaging: same outcome, different words
* [Go through the door]
  -> next_room
* [Enter the room]
  -> next_room
* [Walk inside]
  -> next_room
```

### 3. Use Conditions Thoughtfully

Don't lock everything behind conditions. Give players fallback options:

```
// Good: always at least one available choice
* [Use magic] { skills.magic }
  -> magic_solution

* [Use strength] { strength >= 15 }
  -> strength_solution

* [Look for another way]  // Always available
  -> creative_solution

// Frustrating: might have NO available choices
* [Use magic] { skills.magic }
  -> magic_solution

* [Use strength] { strength >= 15 }
  -> strength_solution
```

### 4. Show Consequences

Let players see that their choices mattered:

```
=== return_to_village

{ saved_village }
The villagers cheer as you approach. Children run to greet you,
and the elder presents you with a medallion.

{ !saved_village }
The village is quiet. Burned buildings still smolder. The survivors
stare at you with hollow eyes as you pass.

// (Using prose conditionals would require custom implementation,
// but you can achieve this with separate passages)
```

### 5. Test All Paths

Play through every combination. Use `jump_to()` for quick testing:

```rust
// Debug helper
runtime.jump_to("boss_fight")?;  // Skip directly to test this passage
```

---

## Where to Go from Here

You now have all the pieces to build interactive narratives with spween. Here are some project ideas:

1. **A murder mystery** where clues unlock new dialogue options
2. **A trading simulation** with resource management and reputation
3. **A visual novel** with relationship tracking
4. **A roguelike adventure** with random encounter selection
5. **A dialogue system** for an existing game

Whatever you build, remember: the best interactive fiction makes players feel like their choices matter. spween gives you the tools—the stories are yours to tell.

Happy writing!
