//! WASM bindings for the spween playground.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;

use spween::{parse, EffectHandler, Runtime, Scene, Value};

/// Simple effect handler that stores state in HashMaps.
#[derive(Debug, Default, Clone)]
struct PlaygroundHandler {
    vars: HashMap<String, Value>,
    has_items: HashMap<String, HashSet<String>>,
    calls: Vec<(String, Vec<Value>)>,
}

impl EffectHandler for PlaygroundHandler {
    fn get_var(&self, name: &str) -> Value {
        self.vars.get(name).cloned().unwrap_or(Value::Null)
    }

    fn set_var(&mut self, name: &str, value: Value) {
        self.vars.insert(name.to_string(), value);
    }

    fn has(&self, category: &str, key: &str) -> bool {
        self.has_items
            .get(category)
            .is_some_and(|keys| keys.contains(key))
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
        self.calls.push((name.to_string(), args.to_vec()));
        Ok(())
    }
}

/// Choice info for JS.
#[derive(Serialize, Deserialize)]
struct JsChoice {
    index: usize,
    text: String,
    available: bool,
}

/// Variable info for JS.
#[derive(Serialize, Deserialize)]
struct JsVariable {
    name: String,
    value: String,
    kind: String,
}

/// Call info for JS.
#[derive(Serialize, Deserialize)]
struct JsCall {
    name: String,
    args: Vec<String>,
}

/// The playground state.
#[wasm_bindgen]
pub struct Playground {
    scene: Option<Scene>,
    handler: PlaygroundHandler,
    passage_name: Option<String>,
    ended: bool,
}

#[wasm_bindgen]
impl Playground {
    /// Create a new playground instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            scene: None,
            handler: PlaygroundHandler::default(),
            passage_name: None,
            ended: true,
        }
    }

    /// Parse a scene from source code.
    pub fn parse(&mut self, source: &str) -> Result<(), JsValue> {
        match parse(source, "playground.scene") {
            Ok(scene) => {
                self.scene = Some(scene);
                Ok(())
            }
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }

    /// Start or restart the scene from the beginning.
    pub fn start(&mut self) -> Result<(), JsValue> {
        let scene = self
            .scene
            .as_ref()
            .ok_or_else(|| JsValue::from_str("No scene loaded"))?;

        self.handler = PlaygroundHandler::default();
        self.ended = scene.passages.is_empty();
        self.passage_name = scene.passages.first().map(|p| p.name.to_string());

        Ok(())
    }

    /// Set a variable value before starting.
    pub fn set_var(&mut self, name: &str, value: &str) {
        let v = if value == "true" {
            Value::Bool(true)
        } else if value == "false" {
            Value::Bool(false)
        } else if value == "null" {
            Value::Null
        } else if let Ok(n) = value.parse::<i64>() {
            Value::Int(n)
        } else if let Ok(f) = value.parse::<f64>() {
            Value::Float(f)
        } else {
            Value::String(value.into())
        };
        self.handler.vars.insert(name.to_string(), v);
    }

    /// Add a "has" item (e.g., inventory item).
    pub fn add_has(&mut self, category: &str, key: &str) {
        self.handler
            .has_items
            .entry(category.to_string())
            .or_default()
            .insert(key.to_string());
    }

    /// Get the current prose text.
    pub fn get_prose(&self) -> Option<String> {
        let scene = self.scene.as_ref()?;
        let passage_name = self.passage_name.as_ref()?;
        let passage = scene
            .passages
            .iter()
            .find(|p| p.name.as_str() == passage_name)?;

        let prose: Vec<&str> = passage
            .content
            .iter()
            .filter_map(|c| match c {
                spween::PassageContent::Prose(p) => Some(p.text.as_str()),
                _ => None,
            })
            .collect();

        if prose.is_empty() {
            None
        } else {
            Some(prose.join("\n\n"))
        }
    }

    /// Get the current passage name.
    pub fn get_passage_name(&self) -> Option<String> {
        self.passage_name.clone()
    }

    /// Get available choices as JSON.
    pub fn get_choices(&self) -> JsValue {
        let empty = || serde_wasm_bindgen::to_value(&Vec::<JsChoice>::new()).unwrap();

        let Some(scene) = &self.scene else {
            return empty();
        };
        let Some(passage_name) = &self.passage_name else {
            return empty();
        };

        let mut runtime = Runtime::new(scene, self.handler.clone());
        if runtime.jump_to(passage_name).is_err() {
            return empty();
        }

        let choices: Vec<JsChoice> = runtime
            .current_choices()
            .into_iter()
            .map(|c| JsChoice {
                index: c.index,
                text: c.text.to_string(),
                available: c.available,
            })
            .collect();

        serde_wasm_bindgen::to_value(&choices).unwrap()
    }

    /// Select a choice by index.
    pub fn select_choice(&mut self, index: usize) -> Result<(), JsValue> {
        let scene = self
            .scene
            .as_ref()
            .ok_or_else(|| JsValue::from_str("No scene loaded"))?;

        if self.ended {
            return Err(JsValue::from_str("Scene has ended"));
        }

        let mut runtime = Runtime::new(scene, self.handler.clone());

        if let Some(name) = &self.passage_name {
            runtime
                .jump_to(name)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }

        runtime
            .select_choice(index)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Extract state from runtime before consuming it
        self.ended = runtime.is_ended();
        self.passage_name = runtime.current_passage().map(|p| p.name.to_string());
        self.handler = runtime.into_handler();

        Ok(())
    }

    /// Check if the scene has ended.
    pub fn is_ended(&self) -> bool {
        self.ended
    }

    /// Get all variables as JSON.
    pub fn get_variables(&self) -> JsValue {
        let vars: Vec<JsVariable> = self
            .handler
            .vars
            .iter()
            .map(|(name, value)| JsVariable {
                name: name.clone(),
                value: value.to_string(),
                kind: match value {
                    Value::Null => "null",
                    Value::Bool(_) => "bool",
                    Value::Int(_) => "int",
                    Value::Float(_) => "float",
                    Value::String(_) => "string",
                }
                .to_string(),
            })
            .collect();

        serde_wasm_bindgen::to_value(&vars).unwrap()
    }

    /// Get effect calls made during this run.
    pub fn get_calls(&self) -> JsValue {
        let calls: Vec<JsCall> = self
            .handler
            .calls
            .iter()
            .map(|(name, args)| JsCall {
                name: name.clone(),
                args: args.iter().map(|a| a.to_string()).collect(),
            })
            .collect();

        serde_wasm_bindgen::to_value(&calls).unwrap()
    }

    /// Get the scene title.
    pub fn get_title(&self) -> Option<String> {
        self.scene.as_ref().map(|s| s.meta.title.to_string())
    }
}

impl Default for Playground {
    fn default() -> Self {
        Self::new()
    }
}
