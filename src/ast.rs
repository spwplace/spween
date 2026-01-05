//! AST types for the spween DSL.
//!
//! This module defines the abstract syntax tree produced by parsing a scene file.

use smol_str::SmolStr;

use crate::Value;

/// A span in the source file (byte offsets).
pub type Span = std::ops::Range<usize>;

/// Trait for AST nodes that have source location information.
pub trait Spanned {
    fn span(&self) -> Span;
}

/// A complete scene file.
#[derive(Debug, Clone)]
pub struct Scene {
    /// Scene metadata from frontmatter
    pub meta: SceneMeta,
    /// Passages in the scene
    pub passages: Vec<Passage>,
    /// Source span
    pub span: Span,
}

/// Scene metadata from the YAML frontmatter.
#[derive(Debug, Clone)]
pub struct SceneMeta {
    /// Unique scene identifier
    pub id: SmolStr,
    /// Human-readable title
    pub title: SmolStr,
    /// Tags for categorization/filtering
    pub tags: Vec<SmolStr>,
    /// Selection weight (higher = more likely)
    pub weight: u32,
    /// Cooldown before scene can repeat
    pub cooldown: u32,
    /// Optional preconditions for this scene
    pub requires: Option<Condition>,
    /// Additional custom metadata (key-value pairs from YAML)
    pub custom: Vec<(SmolStr, Value)>,
    /// Source span
    pub span: Span,
}

/// A passage (named section) in a scene.
#[derive(Debug, Clone)]
pub struct Passage {
    /// Passage name (used for navigation)
    pub name: SmolStr,
    /// Content of the passage
    pub content: Vec<PassageContent>,
    /// Source span
    pub span: Span,
}

/// Content that can appear in a passage.
#[derive(Debug, Clone)]
pub enum PassageContent {
    /// Narrative text
    Prose(Prose),
    /// A choice the player can make
    Choice(Choice),
    /// An effect to execute when the passage is entered
    Effect(Effect),
}

/// Narrative text in a passage.
#[derive(Debug, Clone)]
pub struct Prose {
    /// The text content
    pub text: SmolStr,
    /// Source span
    pub span: Span,
}

/// A choice the player can make.
#[derive(Debug, Clone)]
pub struct Choice {
    /// Display text for the choice
    pub text: SmolStr,
    /// Optional condition for when this choice is available
    pub condition: Option<Condition>,
    /// Effects triggered when this choice is selected
    pub effects: Vec<Effect>,
    /// Where to navigate after selecting this choice
    pub target: Option<NavigationTarget>,
    /// Source span
    pub span: Span,
}

/// Navigation target for a choice.
#[derive(Debug, Clone)]
pub struct NavigationTarget {
    /// Target passage name, or "END" to end the scene
    pub target: SmolStr,
    /// True if this is `-> END`
    pub is_end: bool,
    /// Source span
    pub span: Span,
}

// ============================================================================
// Conditions
// ============================================================================

/// A condition (multiple clauses ANDed together).
#[derive(Debug, Clone)]
pub struct Condition {
    /// Clauses that must all be true
    pub clauses: Vec<ConditionClause>,
    /// Source span
    pub span: Span,
}

/// A single clause in a condition.
#[derive(Debug, Clone)]
pub enum ConditionClause {
    /// Check if a category has a tag: `category.tag`
    Has(HasClause),
    /// Compare a variable to a value: `var >= 10`
    Compare(CompareClause),
    /// Negated clause: `!condition`
    Not(Box<ConditionClause>),
}

/// Check if a category has a tag.
/// DSL: `category.tag` (e.g., `inventory.sword`, `skills.lockpick`)
#[derive(Debug, Clone)]
pub struct HasClause {
    /// The category to check
    pub category: SmolStr,
    /// The tag/key to look for
    pub key: SmolStr,
    /// Source span
    pub span: Span,
}

/// Compare a variable to a value.
/// DSL: `var op value` (e.g., `gold >= 100`, `health < 50`)
#[derive(Debug, Clone)]
pub struct CompareClause {
    /// Variable name
    pub var: SmolStr,
    /// Comparison operator
    pub op: CompareOp,
    /// Value to compare against
    pub value: Value,
    /// Source span
    pub span: Span,
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    /// Greater than or equal: `>=`
    Ge,
    /// Less than or equal: `<=`
    Le,
    /// Greater than: `>`
    Gt,
    /// Less than: `<`
    Lt,
    /// Equal: `==`
    Eq,
    /// Not equal: `!=`
    Ne,
}

// ============================================================================
// Effects
// ============================================================================

/// An effect that modifies game state.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Set a variable to a value: `var = value`
    Set(SetEffect),
    /// Modify a variable by a delta: `var += delta` or `var -= delta`
    Modify(ModifyEffect),
    /// Call a custom function: `call("name", arg1, arg2)`
    Call(CallEffect),
}

/// Set a variable to a specific value.
/// DSL: `~ var = value`
#[derive(Debug, Clone)]
pub struct SetEffect {
    /// Variable name
    pub var: SmolStr,
    /// Value to set
    pub value: Value,
    /// Source span
    pub span: Span,
}

/// Modify a variable by adding/subtracting a delta.
/// DSL: `~ var += delta` or `~ var -= delta`
#[derive(Debug, Clone)]
pub struct ModifyEffect {
    /// Variable name
    pub var: SmolStr,
    /// Delta to add (negative for subtraction)
    pub delta: i64,
    /// Source span
    pub span: Span,
}

/// Call a custom effect handler function.
/// DSL: `~ call("name", arg1, arg2)` or legacy effects like `~ damage hull 10`
#[derive(Debug, Clone)]
pub struct CallEffect {
    /// Function name
    pub name: SmolStr,
    /// Arguments
    pub args: Vec<Value>,
    /// Source span
    pub span: Span,
}

// ============================================================================
// Spanned implementations
// ============================================================================

impl Spanned for Scene {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Spanned for Passage {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Spanned for Choice {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Spanned for Condition {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
