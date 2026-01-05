# Examples

Complete working examples demonstrating spween features.

## Example 1: Simple Text Adventure

A minimal but complete text adventure.

### Scene File (`dungeon.scene`)

```
---
id: dungeon_entrance
title: The Dark Dungeon
tags: [adventure, starter]
weight: 10
---

=== intro

You stand at the entrance to a dark dungeon. A cold wind
blows from within, carrying the smell of damp stone.

Your torch flickers uncertainly.

* [Enter the dungeon]
  ~ entered_dungeon = true
  -> entrance_hall

* [Turn back]
  ~ cowardice += 1
  -> END

=== entrance_hall

The entrance hall is lit by pale moonlight streaming through
cracks in the ceiling. Two passages lead deeper into the dungeon.

To the left, you hear dripping water.
To the right, a faint glow.

* [Go left]
  -> water_room

* [Go right]
  -> glowing_room

* [Search the hall]
  ~ gold += 5
  ~ searched_hall = true
  -> hall_searched

=== hall_searched

You find a few coins scattered in the dust. Not much, but
something.

* [Go left]
  -> water_room

* [Go right]
  -> glowing_room

=== water_room

An underground stream flows through this chamber. The water
looks clean.

* [Drink from the stream]
  ~ health += 10
  ~ drank_water = true
  -> drank

* [Continue deeper]
  -> deep_passage

=== drank

The water is refreshingly cold. You feel invigorated.

* [Continue deeper]
  -> deep_passage

=== glowing_room

The glow comes from luminescent mushrooms growing on the walls.
Beautiful, but you're not sure if they're safe.

* [Harvest mushrooms] { !warned_about_mushrooms }
  ~ has_mushrooms = true
  ~ poisoned = true
  -> mushroom_harvest

* [Leave them alone]
  -> deep_passage

=== mushroom_harvest

You gather several mushrooms. They pulse with an eerie light.

* [Continue deeper]
  -> deep_passage

=== deep_passage

The passage narrows. Up ahead, you see a chest!

* [Open the chest] { !poisoned }
  ~ gold += 100
  ~ found_treasure = true
  -> victory

* [Open the chest] { poisoned }
  ~ gold += 100
  ~ found_treasure = true
  ~ collapsed = true
  -> poison_ending

* [Leave the dungeon]
  -> exit

=== victory

The chest contains gold coins! You've struck it rich!

You make your way back to the surface, treasure in hand.

* [Celebrate!]
  -> END

=== poison_ending

You open the chest and find gold, but the mushroom poison
finally takes hold. You collapse beside your treasure.

* [...]
  -> END

=== exit

You decide you've had enough adventure for one day and
return to the surface.

* [Leave]
  -> END
```

### Rust Code

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
        println!("[Effect: {} {:?}]", name, args);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = include_str!("dungeon.scene");
    let scene = parse(source, "dungeon.scene")?;

    let mut runtime = Runtime::new(&scene, GameState::new());

    println!("=== {} ===\n", scene.meta.title);

    while !runtime.is_ended() {
        // Show stats
        let state = runtime.handler();
        println!("[Health: {} | Gold: {}]\n", state.health, state.gold);

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
    println!("\n=== Game Over ===");
    println!("Final gold: {}", final_state.gold);
    println!("Final health: {}", final_state.health);

    Ok(())
}
```

## Example 2: RPG Dialogue

A more complex dialogue with skill checks and inventory.

### Scene File (`blacksmith.scene`)

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

The blacksmith looks up from his anvil as you approach.
Sweat glistens on his muscular arms.

"What can I do for you, traveler?"

* [Browse weapons]
  -> weapons

* [Browse armor]
  -> armor

* [Ask about rumors]
  -> rumors

* [Leave]
  -> END

=== weapons

He gestures to a rack of weapons along the wall.

"Fine steel, all of it. Made right here."

* [Buy iron sword (50 gold)] { gold >= 50 }
  ~ gold -= 50
  ~ add_item "iron_sword"
  -> bought_weapon

* [Buy steel sword (150 gold)] { gold >= 150 }
  ~ gold -= 150
  ~ add_item "steel_sword"
  -> bought_weapon

* [Ask about custom work] { skills.persuasion }
  -> custom_work

* [Back]
  -> intro

=== armor

Suits of armor line the back wall, from simple leather
to gleaming plate.

* [Buy leather armor (30 gold)] { gold >= 30 }
  ~ gold -= 30
  ~ add_item "leather_armor"
  ~ defense += 2
  -> bought_armor

* [Buy chainmail (100 gold)] { gold >= 100 }
  ~ gold -= 100
  ~ add_item "chainmail"
  ~ defense += 5
  -> bought_armor

* [Buy plate armor (300 gold)] { gold >= 300, strength >= 14 }
  ~ gold -= 300
  ~ add_item "plate_armor"
  ~ defense += 8
  -> bought_armor

* [Back]
  -> intro

=== bought_weapon

"A fine choice! May it serve you well."

He wraps the weapon carefully.

* [Continue shopping]
  -> intro

* [Leave]
  -> END

=== bought_armor

"That'll keep you safe out there."

He helps you with the fitting.

* [Continue shopping]
  -> intro

* [Leave]
  -> END

=== custom_work

His eyes light up with interest.

"Ah, you know quality when you see it. I can make something
special, if you've got the coin... and the materials."

* [Commission a masterwork sword (500 gold)] { gold >= 500, inventory.dragon_scale }
  ~ gold -= 500
  ~ remove_item "dragon_scale"
  ~ add_item "dragonscale_sword"
  ~ blacksmith_reputation += 10
  -> masterwork_complete

* [Ask what materials you'd need]
  -> materials_info

* [Back]
  -> intro

=== materials_info

"For a truly legendary blade, I'd need something extraordinary.
Dragon scale, phoenix feather, that sort of thing."

He shrugs. "Bring me something special and we'll talk."

* [Back]
  -> intro

=== masterwork_complete

The blacksmith works for hours, finally presenting you with
a magnificent blade that seems to shimmer with inner fire.

"My finest work. Guard it well."

* [Thank him]
  ~ quest_complete "masterwork_sword"
  -> END

=== rumors

He leans in conspiratorially.

"Word is there's trouble brewing in the old mine. Goblins,
some say. Worse, say others."

* [Ask about the mine]
  ~ knows_mine_location = true
  -> mine_info

* [Ask about other work]
  -> other_work

* [Back]
  -> intro

=== mine_info

"East of town, past the old oak. Can't miss it."

He pauses. "If you're thinking of clearing it out, I'd pay
good coin for goblin ears. Proof of the deed."

* [Accept bounty]
  ~ quest_start "goblin_bounty"
  ~ bounty_accepted = true
  -> bounty_accepted

* [Decline]
  -> intro

=== bounty_accepted

"Excellent! Ten gold per ear. Don't get yourself killed."

* [Leave to hunt goblins]
  -> END

* [Continue shopping first]
  -> intro

=== other_work

"There's always work for someone handy with a blade. Check
the notice board at the tavern."

* [Back]
  -> intro
```

### Rust Code

```rust
use spween::{parse, Runtime, EffectHandler, Value};
use std::collections::{HashMap, HashSet};

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
        // Starting equipment and skills
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
                    println!("[Acquired: {}]", item);
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
                    println!("[Quest started: {}]", quest);
                    self.quests.insert(quest.to_string());
                }
                Ok(())
            }
            "quest_complete" => {
                if let Some(quest) = args.get(0).and_then(|v| v.as_str()) {
                    println!("[Quest complete: {}]", quest);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

// Run with the same game loop as Example 1
```

## Example 3: Scene Selection System

Managing multiple scenes with requirements and weights.

```rust
use spween::{parse, Runtime, EffectHandler, Value, Scene};
use rand::Rng;

struct SceneManager {
    scenes: Vec<Scene>,
}

impl SceneManager {
    fn load_scenes(paths: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut scenes = Vec::new();
        for path in paths {
            let source = std::fs::read_to_string(path)?;
            let scene = parse(&source, path)?;
            scenes.push(scene);
        }
        Ok(Self { scenes })
    }

    fn select_scene<H: EffectHandler>(&self, handler: &H, context: &str) -> Option<&Scene> {
        // Filter scenes by tag and requirements
        let eligible: Vec<_> = self.scenes.iter()
            .filter(|s| s.meta.tags.iter().any(|t| t == context))
            .filter(|s| {
                // Check requirements using a temporary runtime
                let runtime = Runtime::new(s, DummyHandler(handler));
                runtime.check_scene_requirements()
            })
            .collect();

        if eligible.is_empty() {
            return None;
        }

        // Weighted random selection
        let total_weight: u32 = eligible.iter().map(|s| s.meta.weight).sum();
        let mut roll = rand::thread_rng().gen_range(0..total_weight);

        for scene in eligible {
            if roll < scene.meta.weight {
                return Some(scene);
            }
            roll -= scene.meta.weight;
        }

        eligible.last().copied()
    }
}

// Helper to check requirements without modifying state
struct DummyHandler<'a, H>(&'a H);

impl<'a, H: EffectHandler> EffectHandler for DummyHandler<'a, H> {
    fn get_var(&self, name: &str) -> Value { self.0.get_var(name) }
    fn set_var(&mut self, _: &str, _: Value) {}
    fn has(&self, cat: &str, key: &str) -> bool { self.0.has(cat, key) }
    fn call(&mut self, _: &str, _: &[Value]) -> Result<(), String> { Ok(()) }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = SceneManager::load_scenes(&[
        "scenes/tavern_brawl.scene",
        "scenes/merchant_visit.scene",
        "scenes/mysterious_stranger.scene",
    ])?;

    let game = MyGame::new();

    // Select a random "tavern" scene
    if let Some(scene) = manager.select_scene(&game, "tavern") {
        println!("Selected: {}", scene.meta.title);
        let mut runtime = Runtime::new(scene, game);
        // ... run the scene
    }

    Ok(())
}
```

## Example 4: Save/Load State

Serializing game state between sessions.

```rust
use serde::{Serialize, Deserialize};
use spween::{EffectHandler, Value};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct SaveableGame {
    variables: HashMap<String, SaveableValue>,
    inventory: Vec<String>,
    // Add other persistent state
}

#[derive(Serialize, Deserialize)]
enum SaveableValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl From<Value> for SaveableValue {
    fn from(v: Value) -> Self {
        match v {
            Value::Null => SaveableValue::Null,
            Value::Bool(b) => SaveableValue::Bool(b),
            Value::Int(i) => SaveableValue::Int(i),
            Value::Float(f) => SaveableValue::Float(f),
            Value::String(s) => SaveableValue::String(s.to_string()),
        }
    }
}

impl From<SaveableValue> for Value {
    fn from(v: SaveableValue) -> Self {
        match v {
            SaveableValue::Null => Value::Null,
            SaveableValue::Bool(b) => Value::Bool(b),
            SaveableValue::Int(i) => Value::Int(i),
            SaveableValue::Float(f) => Value::Float(f),
            SaveableValue::String(s) => Value::from(s),
        }
    }
}

impl SaveableGame {
    fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    fn load(path: &str) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }
}

// Convert your game state to/from SaveableGame for persistence
```

## Tips for Writing Good Scenes

1. **Start strong**: The first passage sets the tone
2. **Give meaningful choices**: Each choice should feel different
3. **Use conditions sparingly**: Too many locked choices frustrates players
4. **Show consequences**: Let effects be visible in the narrative
5. **Test all paths**: Make sure every combination works
6. **Keep passages focused**: One scene or decision per passage
7. **Use consistent naming**: `snake_case` for IDs and variables
