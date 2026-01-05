//! Runtime for executing spween scenes.
//!
//! The runtime evaluates conditions and executes effects through a user-provided
//! `EffectHandler` trait implementation.

use smol_str::SmolStr;

use crate::{
    ast::*,
    error::RuntimeError,
    Value,
};

/// Trait for handling game-specific effects and state.
///
/// Implement this trait to connect spween to your game's state system.
///
/// # Example
///
/// ```ignore
/// struct MyGame {
///     variables: HashMap<String, Value>,
///     inventory: HashSet<String>,
/// }
///
/// impl EffectHandler for MyGame {
///     fn get_var(&self, name: &str) -> Value {
///         self.variables.get(name).cloned().unwrap_or(Value::Null)
///     }
///
///     fn set_var(&mut self, name: &str, value: Value) {
///         self.variables.insert(name.to_string(), value);
///     }
///
///     fn has(&self, category: &str, key: &str) -> bool {
///         if category == "inventory" {
///             self.inventory.contains(key)
///         } else {
///             false
///         }
///     }
///
///     fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
///         match name {
///             "damage" => { /* handle damage */ Ok(()) }
///             _ => Err(format!("Unknown function: {}", name))
///         }
///     }
/// }
/// ```
pub trait EffectHandler {
    /// Get the value of a variable.
    fn get_var(&self, name: &str) -> Value;

    /// Set the value of a variable.
    fn set_var(&mut self, name: &str, value: Value);

    /// Check if a category has a specific key/tag.
    fn has(&self, category: &str, key: &str) -> bool;

    /// Call a custom effect function.
    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String>;
}

/// The current state of a running scene.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeState {
    /// Scene is running, at the given passage index
    Running(usize),
    /// Scene has ended
    Ended,
}

/// A choice that is currently available to the player.
#[derive(Debug, Clone)]
pub struct AvailableChoice {
    /// Index of this choice in the passage's choice list
    pub index: usize,
    /// Display text for the choice
    pub text: SmolStr,
    /// Whether the choice's condition is currently met
    pub available: bool,
}

/// Runtime for executing a spween scene.
pub struct Runtime<'scene, H: EffectHandler> {
    scene: &'scene Scene,
    handler: H,
    state: RuntimeState,
    passage_index: std::collections::HashMap<SmolStr, usize>,
}

impl<'scene, H: EffectHandler> Runtime<'scene, H> {
    /// Create a new runtime for the given scene.
    ///
    /// Starts at the first passage (usually "intro").
    pub fn new(scene: &'scene Scene, handler: H) -> Self {
        let passage_index: std::collections::HashMap<SmolStr, usize> = scene
            .passages
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name.clone(), i))
            .collect();

        Self {
            scene,
            handler,
            state: if scene.passages.is_empty() {
                RuntimeState::Ended
            } else {
                RuntimeState::Running(0)
            },
            passage_index,
        }
    }

    /// Get a reference to the effect handler.
    pub fn handler(&self) -> &H {
        &self.handler
    }

    /// Get a mutable reference to the effect handler.
    pub fn handler_mut(&mut self) -> &mut H {
        &mut self.handler
    }

    /// Consume the runtime and return the handler.
    pub fn into_handler(self) -> H {
        self.handler
    }

    /// Get the current runtime state.
    pub fn state(&self) -> &RuntimeState {
        &self.state
    }

    /// Check if the scene has ended.
    pub fn is_ended(&self) -> bool {
        matches!(self.state, RuntimeState::Ended)
    }

    /// Get the current passage, if any.
    pub fn current_passage(&self) -> Option<&Passage> {
        match self.state {
            RuntimeState::Running(idx) => self.scene.passages.get(idx),
            RuntimeState::Ended => None,
        }
    }

    /// Get the prose text for the current passage.
    pub fn current_prose(&self) -> Option<String> {
        self.current_passage().map(|p| {
            p.content
                .iter()
                .filter_map(|c| match c {
                    PassageContent::Prose(prose) => Some(prose.text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        })
    }

    /// Get the choices available in the current passage.
    ///
    /// Returns all choices with their availability status based on conditions.
    pub fn current_choices(&self) -> Vec<AvailableChoice> {
        let Some(passage) = self.current_passage() else {
            return Vec::new();
        };

        passage
            .content
            .iter()
            .filter_map(|c| match c {
                PassageContent::Choice(choice) => Some(choice),
                _ => None,
            })
            .enumerate()
            .map(|(i, choice)| {
                let available = self.evaluate_condition(&choice.condition);
                AvailableChoice {
                    index: i,
                    text: choice.text.clone(),
                    available,
                }
            })
            .collect()
    }

    /// Get only the available choices (those whose conditions are met).
    pub fn available_choices(&self) -> Vec<AvailableChoice> {
        self.current_choices()
            .into_iter()
            .filter(|c| c.available)
            .collect()
    }

    /// Select a choice by its index.
    ///
    /// This executes the choice's effects and navigates to the target passage.
    pub fn select_choice(&mut self, choice_index: usize) -> Result<(), RuntimeError> {
        if self.is_ended() {
            return Err(RuntimeError::SceneEnded);
        }

        // Extract what we need from the choice before mutating
        let (condition_met, effects, nav_target) = {
            let passage = self
                .current_passage()
                .ok_or(RuntimeError::NoCurrentPassage)?;

            let choices: Vec<&Choice> = passage
                .content
                .iter()
                .filter_map(|c| match c {
                    PassageContent::Choice(choice) => Some(choice),
                    _ => None,
                })
                .collect();

            let choice = choices
                .get(choice_index)
                .ok_or(RuntimeError::InvalidChoiceIndex {
                    index: choice_index,
                    available: choices.len(),
                })?;

            let condition_met = self.evaluate_condition(&choice.condition);
            let effects = choice.effects.clone();
            let nav_target = choice.target.clone();

            (condition_met, effects, nav_target)
        };

        // Check condition
        if !condition_met {
            return Err(RuntimeError::ConditionNotMet);
        }

        // Execute effects
        for effect in &effects {
            self.execute_effect(effect)?;
        }

        // Navigate
        if let Some(nav) = &nav_target {
            if nav.is_end {
                self.state = RuntimeState::Ended;
            } else {
                let idx = self
                    .passage_index
                    .get(&nav.target)
                    .copied()
                    .ok_or_else(|| RuntimeError::UnknownPassage(nav.target.to_string()))?;
                self.state = RuntimeState::Running(idx);
            }
        } else {
            // No navigation target - scene ends
            self.state = RuntimeState::Ended;
        }

        Ok(())
    }

    /// Jump directly to a named passage.
    pub fn jump_to(&mut self, passage_name: &str) -> Result<(), RuntimeError> {
        if passage_name == "END" {
            self.state = RuntimeState::Ended;
            return Ok(());
        }

        let idx = self
            .passage_index
            .get(passage_name)
            .copied()
            .ok_or_else(|| RuntimeError::UnknownPassage(passage_name.to_string()))?;

        self.state = RuntimeState::Running(idx);
        Ok(())
    }

    /// Check if the scene's requirements are met.
    pub fn check_scene_requirements(&self) -> bool {
        self.evaluate_condition(&self.scene.meta.requires)
    }

    /// Evaluate a condition against the current game state.
    fn evaluate_condition(&self, condition: &Option<Condition>) -> bool {
        let Some(cond) = condition else {
            return true;
        };

        // All clauses must be true (AND)
        cond.clauses.iter().all(|clause| self.evaluate_clause(clause))
    }

    /// Evaluate a single condition clause.
    fn evaluate_clause(&self, clause: &ConditionClause) -> bool {
        match clause {
            ConditionClause::Has(has) => self.handler.has(&has.category, &has.key),

            ConditionClause::Compare(cmp) => {
                let var_value = self.handler.get_var(&cmp.var);
                self.compare_values(&var_value, cmp.op, &cmp.value)
            }

            ConditionClause::Not(inner) => !self.evaluate_clause(inner),
        }
    }

    /// Compare two values with the given operator.
    fn compare_values(&self, left: &Value, op: CompareOp, right: &Value) -> bool {
        match op {
            CompareOp::Eq => self.values_equal(left, right),
            CompareOp::Ne => !self.values_equal(left, right),
            CompareOp::Gt => self.value_cmp(left, right).map(|o| o.is_gt()).unwrap_or(false),
            CompareOp::Lt => self.value_cmp(left, right).map(|o| o.is_lt()).unwrap_or(false),
            CompareOp::Ge => self.value_cmp(left, right).map(|o| o.is_ge()).unwrap_or(false),
            CompareOp::Le => self.value_cmp(left, right).map(|o| o.is_le()).unwrap_or(false),
        }
    }

    /// Check if two values are equal.
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Int(a), Value::Float(b)) | (Value::Float(b), Value::Int(a)) => {
                (*a as f64 - b).abs() < f64::EPSILON
            }
            (Value::String(a), Value::String(b)) => a == b,
            // Bool true can equal int 1, false equals 0
            (Value::Bool(true), Value::Int(1)) | (Value::Int(1), Value::Bool(true)) => true,
            (Value::Bool(false), Value::Int(0)) | (Value::Int(0), Value::Bool(false)) => true,
            _ => false,
        }
    }

    /// Compare two values for ordering.
    fn value_cmp(&self, left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Some(a.cmp(b)),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
            (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }

    /// Execute a single effect.
    fn execute_effect(&mut self, effect: &Effect) -> Result<(), RuntimeError> {
        match effect {
            Effect::Set(set) => {
                self.handler.set_var(&set.var, set.value.clone());
            }

            Effect::Modify(modify) => {
                let current = self.handler.get_var(&modify.var);
                let new_value = match current {
                    Value::Int(n) => Value::Int(n + modify.delta),
                    Value::Float(f) => Value::Float(f + modify.delta as f64),
                    Value::Null => Value::Int(modify.delta),
                    other => other, // Can't modify non-numeric
                };
                self.handler.set_var(&modify.var, new_value);
            }

            Effect::Call(call) => {
                self.handler
                    .call(&call.name, &call.args)
                    .map_err(RuntimeError::EffectError)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestHandler {
        vars: HashMap<String, Value>,
        has_items: HashMap<String, Vec<String>>,
        calls: Vec<(String, Vec<Value>)>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                vars: HashMap::new(),
                has_items: HashMap::new(),
                calls: Vec::new(),
            }
        }

        fn with_var(mut self, name: &str, value: impl Into<Value>) -> Self {
            self.vars.insert(name.to_string(), value.into());
            self
        }

        fn with_has(mut self, category: &str, key: &str) -> Self {
            self.has_items
                .entry(category.to_string())
                .or_default()
                .push(key.to_string());
            self
        }
    }

    impl EffectHandler for TestHandler {
        fn get_var(&self, name: &str) -> Value {
            self.vars.get(name).cloned().unwrap_or(Value::Null)
        }

        fn set_var(&mut self, name: &str, value: Value) {
            self.vars.insert(name.to_string(), value);
        }

        fn has(&self, category: &str, key: &str) -> bool {
            self.has_items
                .get(category)
                .map(|keys| keys.contains(&key.to_string()))
                .unwrap_or(false)
        }

        fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
            self.calls.push((name.to_string(), args.to_vec()));
            Ok(())
        }
    }

    fn parse_test_scene(source: &str) -> Scene {
        crate::parse(source, "test.scene").expect("Failed to parse test scene")
    }

    #[test]
    fn test_runtime_basic() {
        let scene = parse_test_scene(
            r#"---
id: test
title: Test
weight: 10
cooldown: 5
---

=== intro

Hello, world!

* [Continue]
  -> END
"#,
        );

        let handler = TestHandler::new();
        let mut runtime = Runtime::new(&scene, handler);

        assert!(!runtime.is_ended());
        assert_eq!(runtime.current_prose(), Some("Hello, world!".to_string()));

        let choices = runtime.current_choices();
        assert_eq!(choices.len(), 1);
        assert!(choices[0].available);

        runtime.select_choice(0).unwrap();
        assert!(runtime.is_ended());
    }

    #[test]
    fn test_runtime_conditions() {
        let scene = parse_test_scene(
            r#"---
id: test
title: Test
weight: 10
cooldown: 5
---

=== intro

Choose wisely.

* [Rich option] { gold >= 100 }
  -> END

* [Poor option]
  -> END
"#,
        );

        // Without enough gold
        let handler = TestHandler::new().with_var("gold", 50i64);
        let runtime = Runtime::new(&scene, handler);
        let choices = runtime.current_choices();
        assert!(!choices[0].available);
        assert!(choices[1].available);

        // With enough gold
        let handler = TestHandler::new().with_var("gold", 200i64);
        let runtime = Runtime::new(&scene, handler);
        let choices = runtime.current_choices();
        assert!(choices[0].available);
        assert!(choices[1].available);
    }

    #[test]
    fn test_runtime_has_condition() {
        let scene = parse_test_scene(
            r#"---
id: test
title: Test
weight: 10
cooldown: 5
---

=== intro

Do you have the key?

* [Use key] { inventory.key }
  -> END

* [Leave]
  -> END
"#,
        );

        // Without key
        let handler = TestHandler::new();
        let runtime = Runtime::new(&scene, handler);
        let choices = runtime.current_choices();
        assert!(!choices[0].available);

        // With key
        let handler = TestHandler::new().with_has("inventory", "key");
        let runtime = Runtime::new(&scene, handler);
        let choices = runtime.current_choices();
        assert!(choices[0].available);
    }

    #[test]
    fn test_runtime_effects() {
        let scene = parse_test_scene(
            r#"---
id: test
title: Test
weight: 10
cooldown: 5
---

=== intro

Get rewards!

* [Collect]
  ~ gold += 100
  ~ visited = true
  ~ notify "reward_collected"
  -> END
"#,
        );

        let handler = TestHandler::new().with_var("gold", 50i64);
        let mut runtime = Runtime::new(&scene, handler);

        runtime.select_choice(0).unwrap();

        let handler = runtime.handler();
        assert_eq!(handler.get_var("gold"), Value::Int(150));
        assert_eq!(handler.get_var("visited"), Value::Bool(true));
        assert_eq!(handler.calls.len(), 1);
        assert_eq!(handler.calls[0].0, "notify");
    }

    #[test]
    fn test_runtime_navigation() {
        let scene = parse_test_scene(
            r#"---
id: test
title: Test
weight: 10
cooldown: 5
---

=== intro

Start here.

* [Go to middle]
  -> middle

=== middle

You're in the middle.

* [Go to end]
  -> END
"#,
        );

        let handler = TestHandler::new();
        let mut runtime = Runtime::new(&scene, handler);

        assert_eq!(
            runtime.current_passage().map(|p| p.name.as_str()),
            Some("intro")
        );

        runtime.select_choice(0).unwrap();
        assert!(!runtime.is_ended());
        assert_eq!(
            runtime.current_passage().map(|p| p.name.as_str()),
            Some("middle")
        );

        runtime.select_choice(0).unwrap();
        assert!(runtime.is_ended());
    }
}
