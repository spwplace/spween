# Effects

Effects are how choices change the world. When a player picks up gold, learns a secret, or makes an enemy—that's effects at work. They're the bridge between narrative choices and actual game state.

## The Basic Idea

Effects are lines starting with `~` inside a choice. They run when the player selects that choice:

```
* [Collect the treasure]
  ~ gold += 100
  ~ found_treasure = true
  -> exit_cave
```

When the player picks "Collect the treasure":
1. Their gold increases by 100
2. A `found_treasure` flag gets set to true
3. The scene navigates to `exit_cave`

Effects always run in order, top to bottom, before navigation happens.

## Types of Effects

### Setting Variables

The simplest effect assigns a value to a variable:

```
~ visited = true
~ player_name = "Hero"
~ difficulty = 3
~ damage_multiplier = 1.5
```

**Syntax:** `~ variable = value`

You can set different types of values:
- **Booleans**: `true`, `false`
- **Integers**: `42`, `-10`, `0`
- **Floats**: `3.14`, `0.5`
- **Strings**: `"text in quotes"`

This calls your `EffectHandler::set_var()` method with the variable name and new value.

### Modifying Variables

Often you want to add to or subtract from a value rather than replacing it entirely:

```
~ gold += 50       // Gain 50 gold
~ health -= 10     // Lose 10 health
~ score += 1       // Increment by 1
~ arrows -= 1      // Use an arrow
```

**Syntax:**
- `~ variable += amount` — add to the variable
- `~ variable -= amount` — subtract from the variable

What happens under the hood: spween first calls `get_var()` to get the current value, performs the arithmetic, then calls `set_var()` with the result.

**Pro tip:** If a variable doesn't exist yet (returns `null`), spween treats it as 0 for modification. So `~ score += 10` on a new game initializes `score` to 10.

### Calling Custom Functions

This is where effects become truly powerful. You can define custom commands that do anything your game needs:

```
~ play_sound "victory"
~ spawn_enemy "goblin" 3
~ trigger_event "boss_defeated"
~ unlock_achievement "treasure_hunter"
```

**Syntax:** `~ function_name arg1 arg2 ...`

Arguments can be:
- **Strings**: `"text"` or bare identifiers like `goblin`
- **Numbers**: `42`, `3.14`
- **Booleans**: `true`, `false`

When spween sees these, it calls your `EffectHandler::call()` method. You decide what each function name means:

```rust
fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
    match name {
        "play_sound" => {
            let sound = args.get(0).and_then(|v| v.as_str()).unwrap_or("default");
            self.audio.play(sound);
            Ok(())
        }
        "spawn_enemy" => {
            let enemy_type = args.get(0).and_then(|v| v.as_str()).unwrap_or("goblin");
            let count = args.get(1).and_then(|v| v.as_int()).unwrap_or(1);
            self.spawn_enemies(enemy_type, count as u32);
            Ok(())
        }
        _ => Ok(()) // Unknown effects silently succeed
    }
}
```

### Alternative Call Syntax

If you prefer, there's also a function-call style syntax:

```
~ call("play_sound", "victory")
~ call("spawn_enemy", "goblin", 3)
```

This is identical to the space-separated syntax—just a matter of taste.

## Stacking Multiple Effects

Real choices often have several effects:

```
* [Buy the legendary sword]
  ~ gold -= 1000
  ~ inventory_legendary_sword = true
  ~ merchant_reputation += 5
  ~ achievement_unlocked "big_spender"
  ~ play_sound "purchase"
  -> purchase_complete
```

All five effects run in sequence before navigating to `purchase_complete`. The order matters—if an earlier effect fails, later ones won't run.

## Implementing Effects in Your Game

Your `EffectHandler` is the bridge between spween's effect system and your actual game:

```rust
impl EffectHandler for MyGame {
    fn get_var(&self, name: &str) -> Value {
        // Used by += and -= to get the current value
        self.variables.get(name).cloned().unwrap_or(Value::Null)
    }

    fn set_var(&mut self, name: &str, value: Value) {
        // Called by = and after += / -= compute the new value
        self.variables.insert(name.to_string(), value);
    }

    fn has(&self, _category: &str, _key: &str) -> bool {
        // Not used for effects, only for conditions
        false
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
        // Called for custom effects like ~ play_sound "victory"
        match name {
            "play_sound" => {
                if let Some(sound) = args.get(0).and_then(|v| v.as_str()) {
                    self.audio.play(sound);
                }
                Ok(())
            }
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
            "trigger_event" => {
                if let Some(event) = args.get(0).and_then(|v| v.as_str()) {
                    self.event_queue.push(event.to_string());
                }
                Ok(())
            }
            _ => {
                // Log unknown effects but don't fail
                eprintln!("Unknown effect: {} {:?}", name, args);
                Ok(())
            }
        }
    }
}
```

## When Effects Run

Understanding the execution order helps you write correct scenes:

1. Player selects a choice
2. Condition is checked (if any)—if it fails, nothing happens
3. **All effects run in order**
4. Navigation happens

```
* [Open the chest]
  ~ chest_opened = true        // Step 1: Set flag
  ~ gold += random_gold        // Step 2: Add gold
  ~ play_sound "chest_open"    // Step 3: Play sound
  ~ check_for_trap             // Step 4: Maybe trigger trap
  -> chest_contents            // Step 5: Navigate
```

## Error Handling

Call effects can return errors, which propagate up to the runtime:

```rust
fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
    match name {
        "consume_item" => {
            let item = args.get(0).and_then(|v| v.as_str())
                .ok_or("consume_item requires an item name")?;

            if !self.inventory.contains(item) {
                return Err(format!("Can't consume {}: you don't have it!", item));
            }

            self.inventory.remove(item);
            Ok(())
        }
        _ => Ok(())
    }
}
```

In your game loop, you can catch these errors:

```rust
match runtime.select_choice(index) {
    Ok(()) => {
        // Everything worked
    }
    Err(RuntimeError::EffectError(msg)) => {
        // An effect failed
        println!("Something went wrong: {}", msg);
    }
    Err(e) => {
        // Other error (invalid index, condition not met, etc.)
    }
}
```

**Design choice:** Should unknown effects fail silently or error? In the examples above, we return `Ok(())` for unknowns—this makes scenes forward-compatible (they'll work even if you haven't implemented every effect yet). But you could choose to error on unknowns for stricter validation.

## Practical Patterns

Let's look at common ways effects get used in real games.

### Resource Management

```
* [Buy health potion] { gold >= 25 }
  ~ gold -= 25
  ~ potions += 1
  -> shop

* [Rest at the inn] { gold >= 10 }
  ~ gold -= 10
  ~ health = 100     // Set to full
  ~ fatigue = 0      // Reset fatigue
  ~ advance_time 8   // 8 hours pass
  -> morning
```

### Quest Tracking

```
* [Accept the dragon slayer quest]
  ~ quest_dragon_started = true
  ~ add_quest "dragon_slayer"
  ~ journal_entry "The village elder asked me to slay the dragon..."
  -> quest_details

* [Report your success]
  ~ quest_dragon_complete = true
  ~ remove_quest "dragon_slayer"
  ~ gold += 500
  ~ reputation_village += 25
  ~ play_sound "quest_complete"
  -> reward_scene
```

### Story Flags and Consequences

```
* [Tell her the truth]
  ~ told_truth_to_sarah = true
  ~ sarah_trust += 20
  ~ trigger_event "sarah_learns_truth"
  -> truth_reaction

* [Lie to protect her]
  ~ lied_to_sarah = true
  ~ sarah_trust -= 5          // Small hit now...
  ~ pending_revelation = true  // ...bigger consequence later
  -> lie_reaction
```

### Combat and Action

```
* [Attack the goblin]
  ~ combat_start "goblin"
  ~ player_initiative = true
  -> combat_round

* [Victory!]
  ~ enemies_defeated += 1
  ~ xp += 50
  ~ loot_drop "goblin"
  ~ play_sound "victory_fanfare"
  -> loot_screen
```

### Environment Changes

```
* [Pull the lever]
  ~ lever_a_pulled = true
  ~ door_north_open = true
  ~ door_south_open = false
  ~ play_sound "mechanism"
  ~ rumble_effect
  -> lever_result

* [Light your torch]
  ~ torches -= 1
  ~ room_lit = true
  ~ reveal_hidden "secret_door"
  ~ play_sound "torch_ignite"
  -> lit_room
```

### Conversation Tracking

Many games track what topics have been discussed:

```
* [Ask about the war] { !asked_war }
  ~ asked_war = true
  -> war_story

* [Ask about their family] { !asked_family }
  ~ asked_family = true
  -> family_story

* [I should go] { asked_war, asked_family }
  ~ conversation_exhausted "elder"
  -> farewell
```

## Tips for Writing Good Effects

1. **Use descriptive names**: `quest_dragon_complete` tells you more than `qd1`. You'll thank yourself when debugging.

2. **Group related effects**: If several effects are logically one action, keep them together with a comment:

   ```
   * [Buy the sword]
     // Transaction
     ~ gold -= 100
     ~ merchant_gold += 100
     // Inventory
     ~ add_item "iron_sword"
     ~ equip_weapon "iron_sword"
     // Feedback
     ~ play_sound "purchase"
     -> shop
   ```

3. **Mind the order**: Effects that depend on each other should be ordered correctly. If `check_for_trap` might kill the player, put it after `gold += 100` so they at least get the gold first (or after, if you're mean).

4. **Return meaningful errors**: When a call effect fails, a clear error message helps debugging enormously.

5. **Consider idempotency**: When possible, effects that run twice should be safe. `~ opened_chest = true` is idempotent—setting it twice is fine. But `~ gold += 100` isn't—running it twice gives 200 gold. This matters if you ever need to replay or retry.

## Summary

Effects are your tools for making choices matter:

- **Set variables** with `~ var = value`
- **Modify variables** with `~ var += amount` or `~ var -= amount`
- **Call custom functions** with `~ function_name arg1 arg2`
- Effects run **in order**, **before navigation**
- Implement `set_var()` and `call()` in your `EffectHandler`
- `call()` can return errors that propagate to your game loop

Next up: [Runtime API](05-runtime.md) — learn how to integrate spween deeply into your game.
