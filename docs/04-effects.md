# Effects

Effects modify game state when a choice is selected.

## Overview

Effects are lines starting with `~` inside a choice:

```
* [Collect the treasure]
  ~ gold += 100
  ~ found_treasure = true
  -> exit
```

Effects execute in order, before navigation.

## Effect Types

### Set Variable

Assign a value to a variable:

```
~ visited = true
~ player_name = "Hero"
~ difficulty = 3
~ multiplier = 1.5
```

**Syntax:** `~ variable = value`

**Value types:**
- Boolean: `true`, `false`
- Integer: `42`, `-10`, `0`
- Float: `3.14`, `0.5`
- String: `"text here"`

### Modify Variable

Add or subtract from a numeric variable:

```
~ gold += 50       // Add 50
~ health -= 10     // Subtract 10
~ score += 1       // Increment
```

**Syntax:**
- `~ variable += amount` (add)
- `~ variable -= amount` (subtract)

If the variable doesn't exist or is null, it's treated as 0.

### Call Effect

Invoke a custom function in your game:

```
~ play_sound "victory"
~ spawn_enemy "goblin" 3
~ trigger_event "boss_defeated"
```

**Syntax:** `~ function_name arg1 arg2 ...`

Arguments can be:
- Strings: `"text"` or bare identifiers like `goblin`
- Numbers: `42`, `3.14`
- Booleans: `true`, `false`

### Call with Parentheses

Alternative syntax for calls:

```
~ call("play_sound", "victory")
~ call("spawn_enemy", "goblin", 3)
```

**Syntax:** `~ call("function_name", arg1, arg2, ...)`

## Multiple Effects

Stack multiple effects in a choice:

```
* [Buy the legendary sword]
  ~ gold -= 1000
  ~ inventory_legendary_sword = true
  ~ merchant_reputation += 5
  ~ achievement_unlocked "big_spender"
  -> purchase_complete
```

Effects execute top-to-bottom.

## Implementing Effects

Your `EffectHandler` processes all effects:

```rust
impl EffectHandler for MyGame {
    fn get_var(&self, name: &str) -> Value {
        // Used by Modify to get current value
        self.variables.get(name).cloned().unwrap_or(Value::Null)
    }

    fn set_var(&mut self, name: &str, value: Value) {
        // Called by Set and Modify effects
        self.variables.insert(name.to_string(), value);
    }

    fn has(&self, _category: &str, _key: &str) -> bool {
        // Not used for effects, only conditions
        false
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
        // Called for custom effects
        match name {
            "play_sound" => {
                if let Some(Value::String(sound)) = args.get(0) {
                    self.audio.play(sound);
                }
                Ok(())
            }
            "spawn_enemy" => {
                let enemy_type = args.get(0)
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let count = args.get(1)
                    .and_then(|v| v.as_int())
                    .unwrap_or(1);
                self.spawn_enemies(enemy_type, count as u32);
                Ok(())
            }
            "trigger_event" => {
                if let Some(Value::String(event)) = args.get(0) {
                    self.events.trigger(event);
                }
                Ok(())
            }
            _ => Err(format!("Unknown effect: {}", name))
        }
    }
}
```

## Effect Execution Flow

When a choice is selected:

1. Condition is checked (if any)
2. All effects execute in order
3. Navigation occurs

```
* [Open the chest]
  ~ chest_opened = true        // 1. Set flag
  ~ gold += 50                 // 2. Add gold
  ~ play_sound "chest_open"    // 3. Play sound
  ~ check_for_trap             // 4. Custom logic
  -> chest_contents            // 5. Navigate
```

## Error Handling

If a call effect returns an error, it propagates:

```rust
fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
    match name {
        "consume_item" => {
            let item = args.get(0).and_then(|v| v.as_str())
                .ok_or("consume_item requires item name")?;

            if !self.inventory.contains(item) {
                return Err(format!("Item not found: {}", item));
            }

            self.inventory.remove(item);
            Ok(())
        }
        _ => Ok(()) // Unknown effects succeed silently
    }
}
```

In the runtime:

```rust
match runtime.select_choice(index) {
    Ok(()) => { /* success */ }
    Err(RuntimeError::EffectError(msg)) => {
        println!("Effect failed: {}", msg);
    }
    Err(e) => { /* other error */ }
}
```

## Common Patterns

### Resource Management

```
* [Buy health potion] { gold >= 25 }
  ~ gold -= 25
  ~ potions += 1
  -> shop

* [Rest at inn] { gold >= 10 }
  ~ gold -= 10
  ~ health = 100
  ~ fatigue = 0
  -> morning
```

### Quest Tracking

```
* [Accept the quest]
  ~ quest_dragon_started = true
  ~ quest_log_add "dragon_slayer"
  -> quest_details

* [Complete delivery]
  ~ quest_delivery_complete = true
  ~ gold += 100
  ~ reputation_merchants += 10
  ~ quest_log_remove "delivery"
  -> delivery_done
```

### Story Flags

```
* [Tell the truth]
  ~ told_truth = true
  ~ trust_sarah += 2
  -> truth_reaction

* [Lie to protect them]
  ~ lied_to_sarah = true
  ~ sarah_believes_lie = true
  -> lie_reaction
```

### Combat Results

```
* [Attack the goblin]
  ~ combat_start "goblin"
  ~ player_attacks_first = true
  -> combat

* [Victory!]
  ~ enemies_defeated += 1
  ~ xp += 50
  ~ loot_drop "goblin"
  ~ play_sound "victory"
  -> loot_screen
```

### Environmental Changes

```
* [Pull the lever]
  ~ lever_pulled = true
  ~ door_a_open = true
  ~ door_b_open = false
  ~ play_sound "mechanism"
  -> lever_result

* [Light the torch]
  ~ room_lit = true
  ~ torches -= 1
  ~ reveal_hidden_door = true
  -> lit_room
```

### Dialogue Tracking

```
* [Ask about the war]
  ~ asked_about_war = true
  ~ dialogue_exhausted_war = true
  -> war_story

* [Ask about family] { !dialogue_exhausted_family }
  ~ asked_about_family = true
  ~ dialogue_exhausted_family = true
  -> family_story
```

## Tips

1. **Use descriptive names**: `quest_dragon_complete` > `qd1`

2. **Group related effects**: Put all effects for one logical action together

3. **Consider order**: Effects that depend on each other should be ordered correctly

4. **Handle errors gracefully**: Return meaningful error messages from `call()`

5. **Keep effects idempotent when possible**: Running the same effect twice shouldn't break things
