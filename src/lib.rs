//! # spween
//!
//! A game-agnostic DSL for narrative/choice-based content.
//!
//! ## Overview
//!
//! spween provides a simple markup language for creating branching narratives
//! with passages, choices, conditions, and effects. It's designed to be embedded
//! in games or interactive fiction engines.
//!
//! ## DSL Syntax
//!
//! ```text
//! ---
//! id: example_scene
//! title: Example Scene
//! weight: 10
//! ---
//!
//! === intro
//!
//! You stand at a crossroads.
//!
//! * [Go left] { has_item("map") }
//!   ~ visited_left = true
//!   -> left_path
//!
//! * [Go right] when courage >= 5
//!   ~ courage -= 1
//!   -> right_path
//!
//! * [Turn back]
//!   -> END
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use spween::{parse, Runtime, EffectHandler, Value};
//!
//! // Parse a scene file
//! let scene = parse(source, "example.scene")?;
//!
//! // Implement your game's effect handler
//! struct MyGame { /* ... */ }
//! impl EffectHandler for MyGame { /* ... */ }
//!
//! // Run the scene
//! let mut runtime = Runtime::new(&scene, &mut my_game);
//! let choices = runtime.current_choices();
//! runtime.select_choice(0)?;
//! ```

mod ast;
mod error;
mod lexer;
mod parser;
mod runtime;
mod value;

pub use ast::*;
pub use error::*;
pub use parser::parse;
pub use runtime::{EffectHandler, Runtime, RuntimeState};
pub use value::Value;
