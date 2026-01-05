# Welcome to Parsers and Interpreters

**A hands-on guide using `spween`, a narrative DSL for games**

Have you ever wondered how programming languages actually work? How does a computer take text like `if x > 5` and know what to do with it? If that question excites you, you're in the right place.

This guide will demystify the magic behind programming languages by building something real: a domain-specific language (DSL) for creating interactive stories in games. By the end, you'll understand the fundamental pipeline that powers everything from Python to JavaScript to your favorite game's scripting system.

Don't worry if you're new to this—we'll take it step by step, and everything will click into place.

---

## Quick Links

**Want to use spween in your project?**
- [Getting Started](docs/01-getting-started.md) — Write your first interactive scene
- [DSL Syntax](docs/02-dsl-syntax.md) — Complete language reference
- [Conditions](docs/03-conditions.md) — Making choices conditional
- [Effects](docs/04-effects.md) — Modifying game state
- [Runtime API](docs/05-runtime.md) — Integrating with your game
- [Examples](docs/06-examples.md) — Complete working examples

**Want to learn how interpreters work?** Keep reading!

---

## Table of Contents

1. [The Big Picture](#the-big-picture)
2. [Our Running Example](#our-running-example)
3. [Stage 1: Lexical Analysis](#stage-1-lexical-analysis-tokenization)
4. [Stage 2: Parsing](#stage-2-parsing-building-the-ast)
5. [Stage 3: The Abstract Syntax Tree](#stage-3-the-abstract-syntax-tree)
6. [Stage 4: Runtime Execution](#stage-4-runtime-execution)
7. [Putting It All Together](#putting-it-all-together)
8. [Key Concepts Recap](#key-concepts-recap)

---

## The Big Picture

Here's a secret that might surprise you: every programming language—from the most complex enterprise systems to this tiny DSL—follows the exact same fundamental pipeline:

```
Source Code → Lexer → Tokens → Parser → AST → Runtime → Behavior
```

That's it! Three main stages transform text into action. Let's give each one a friendly name:

| Stage | What It Does | Real-World Analogy |
|-------|--------------|-------------------|
| **Lexer** | Breaks text into meaningful chunks | Like your eyes scanning words in a sentence |
| **Parser** | Builds a tree structure from those chunks | Like your brain understanding grammar |
| **Runtime** | Executes the structure | Like comprehending the meaning and acting on it |

Think about reading the sentence "The cat sat on the mat." Your eyes don't see individual letters—they group them into words (lexing). Your brain doesn't see a flat list of words—it understands that "the cat" is the subject doing the sitting (parsing). And finally, you picture a cat on a mat (execution).

Programming languages work exactly the same way. Once you internalize this, a huge amount of computer science suddenly makes sense.

---

## Our Running Example

Let's look at a scene written in spween's DSL. Don't worry about understanding every detail yet—we'll break it all down:

```
---
id: tavern_encounter
title: The Mysterious Stranger
weight: 10
---

=== intro

A hooded figure sits alone in the corner of the tavern.

* [Approach them] { courage >= 5 }
  ~ courage -= 1
  -> conversation

* [Order a drink instead]
  ~ gold -= 2
  -> END

=== conversation

"I've been expecting you," they whisper.

* [Ask about the quest]
  ~ quest_started = true
  -> END
```

Even if you've never seen this syntax before, you can probably guess what's happening: it's describing an interactive scene where a player makes choices that affect the story.

This tiny program contains:
- **Metadata** (the YAML frontmatter between `---`)
- **Passages** (named sections starting with `===`)
- **Prose** (the narrative text)
- **Choices** (lines starting with `*`)
- **Conditions** (the `{ courage >= 5 }` part—a requirement to show the choice)
- **Effects** (lines starting with `~`—things that happen when you pick a choice)
- **Navigation** (the `->` arrows telling the game where to go next)

Now let's see how each stage of our pipeline processes this. This is where it gets fun!

---

## Stage 1: Lexical Analysis (Tokenization)

**File: `src/lexer.rs`**

The lexer's job is beautifully simple: take a stream of characters and group them into *tokens*—the atomic building blocks of the language. It's like having someone read a sentence aloud and pause between each word.

### What Are Tokens?

A token is just a meaningful chunk. When the lexer sees `>=`, it doesn't see two separate characters—it recognizes them as a single "greater than or equal" operator. When it sees `courage`, it knows that's an identifier (a name for something).

Here's what spween's tokens look like:

```rust
pub enum Token {
    FrontmatterDelim,           // ---
    PassageHeader(SmolStr),     // === intro
    ChoiceMarker,               // *
    EffectMarker,               // ~
    Arrow,                      // ->
    LBracket,                   // [
    RBracket,                   // ]
    LBrace,                     // {
    RBrace,                     // }
    Ge,                         // >=
    PlusEq,                     // +=
    Number(i64),                // 42
    String(SmolStr),            // "hello"
    Identifier(SmolStr),        // courage, gold, intro
    // ... and more
}
```

Each variant represents a different kind of "word" in our language.

### Seeing It in Action

Given this input:
```
* [Approach them] { courage >= 5 }
```

The lexer produces this stream of tokens:
```
ChoiceMarker        at 0..1
LBracket            at 2..3
Identifier("Approach")  at 3..11
Identifier("them")      at 12..16
RBracket            at 16..17
LBrace              at 18..19
Identifier("courage")   at 20..27
Ge                  at 28..30
Number(5)           at 31..32
RBrace              at 33..34
```

Notice something important: each token remembers *where* it came from in the original text (that's what those byte ranges mean). This is crucial for error messages—when something goes wrong, we can point the user to exactly where in their file the problem is.

### How It Works Under the Hood

spween uses a library called `logos` that makes lexer writing almost effortless. You just describe what patterns to match:

```rust
#[derive(Logos)]
#[logos(skip r"[ \t]+")]  // Skip whitespace
pub enum RawToken {
    #[token("---")]
    FrontmatterDelim,

    #[token("===")]
    PassageHeader,

    #[token("*")]
    ChoiceMarker,

    #[token(">=")]
    Ge,

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse())]
    Number(i64),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier(String),
}
```

The `#[token(...)]` attribute says "when you see exactly this text, produce this token." The `#[regex(...)]` attribute says "when you see text matching this pattern, produce this token." Logos compiles these rules into an efficient state machine that zips through your source code.

### A Neat Trick: Indentation

spween uses indentation to group effects under choices (like Python does with code blocks). The lexer tracks indentation levels and emits special `Indent` and `Dedent` tokens when the indentation changes:

```rust
pub struct Lexer<'src> {
    source: &'src str,
    inner: SpannedIter<'src, RawToken>,
    indent_stack: Vec<usize>,     // Track nesting levels
    pending_dedents: usize,       // Emit DEDENT tokens when we unindent
}
```

This means the parser never has to think about whitespace—it just sees clean structural tokens.

---

## Stage 2: Parsing (Building the AST)

**File: `src/parser.rs`**

Now things get interesting. The parser takes our flat stream of tokens and builds a *tree structure* that captures the meaning of the program. This is where we go from "a sequence of words" to "an understanding of grammar."

### What the Parser Needs to Figure Out

When the parser sees `* [Approach them] { courage >= 5 }`, it needs to understand:
- This is a choice
- The choice text is "Approach them"
- There's a condition attached: courage must be at least 5
- The condition is a comparison, with a variable on one side and a number on the other

That's a lot of structure hidden in a flat sequence of tokens!

### Recursive Descent: The Elegant Solution

spween uses a technique called *recursive descent parsing*. The idea is wonderfully simple: write one function for each "kind of thing" in your language, and have them call each other as needed.

```rust
impl Parser {
    pub fn parse(&mut self) -> Result<Scene, ParseError> {
        let meta = self.parse_frontmatter()?;
        let mut passages = Vec::new();

        while !self.is_at_end() {
            if matches!(self.current_token(), Token::PassageHeader(_)) {
                passages.push(self.parse_passage()?);
            }
        }

        Ok(Scene { meta, passages, span })
    }

    fn parse_passage(&mut self) -> Result<Passage, ParseError> {
        // Get the passage name from === header
        let name = /* ... */;

        let mut content = Vec::new();
        while /* not at next passage or EOF */ {
            match self.current_token() {
                Token::ChoiceMarker => {
                    content.push(PassageContent::Choice(self.parse_choice()?));
                }
                Token::EffectMarker => {
                    content.push(PassageContent::Effect(self.parse_effect()?));
                }
                // ... handle other content types
            }
        }

        Ok(Passage { name, content, span })
    }
}
```

See how `parse()` calls `parse_passage()`, which might call `parse_choice()`, which might call `parse_condition()`? The call stack naturally mirrors the structure of the language. That's the "recursive" part of recursive descent.

### The Parser's Toolbox

Every parser needs a few basic operations:

```rust
// Look at the current token without consuming it
fn current_token(&self) -> &Token { ... }

// Move to the next token and return the previous one
fn advance(&mut self) -> &SpannedToken { ... }

// Demand a specific token (error if it's not there)
fn consume(&mut self, expected: &Token, message: &str)
    -> Result<SpannedToken, ParseError> { ... }

// Check if the current token matches (without consuming)
fn check(&self, token: &Token) -> bool { ... }

// Consume if it matches, return whether it matched
fn match_token(&mut self, token: &Token) -> bool { ... }
```

These simple building blocks let you write parsing code that reads almost like a description of the language:

```rust
fn parse_choice(&mut self) -> Result<Choice, ParseError> {
    self.consume(&Token::ChoiceMarker, "Expected *")?;
    self.consume(&Token::LBracket, "Expected [")?;

    let text = /* collect text until ] */;

    self.consume(&Token::RBracket, "Expected ]")?;

    // Optional condition: { ... }
    let condition = if self.check(&Token::LBrace) {
        Some(self.parse_condition()?)
    } else {
        None
    };

    // Parse effects and navigation...
    Ok(Choice { text, condition, effects, target, span })
}
```

### Good Errors Matter

One mark of a well-designed parser is helpful error messages. Because we track spans (where each token came from), we can tell users exactly where things went wrong:

```rust
pub enum ParseError {
    #[error("unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,  // Points to the exact location
    },

    #[error("invalid condition: {message}")]
    InvalidCondition { message: String, span: Span },
}
```

Instead of "syntax error somewhere," users get "expected ']' at line 5, column 23." That's the difference between frustration and understanding.

---

## Stage 3: The Abstract Syntax Tree

**File: `src/ast.rs`**

The parser produces an *Abstract Syntax Tree* (AST)—a tree-shaped data structure that represents the program. It's "abstract" because it captures the *meaning* without the syntactic noise. No parentheses, no commas, no worrying about whitespace—just pure structure.

### The Shape of Our Tree

Here's what spween's AST looks like:

```rust
/// A complete scene file
pub struct Scene {
    pub meta: SceneMeta,        // Frontmatter data
    pub passages: Vec<Passage>, // Named sections
    pub span: Span,
}

/// A passage (named section)
pub struct Passage {
    pub name: SmolStr,              // "intro", "conversation"
    pub content: Vec<PassageContent>,
    pub span: Span,
}

/// What can appear in a passage
pub enum PassageContent {
    Prose(Prose),      // Narrative text
    Choice(Choice),    // Player decision point
    Effect(Effect),    // State modification
}

/// A player choice
pub struct Choice {
    pub text: SmolStr,                    // "[Approach them]"
    pub condition: Option<Condition>,     // { courage >= 5 }
    pub effects: Vec<Effect>,             // ~ courage -= 1
    pub target: Option<NavigationTarget>, // -> conversation
    pub span: Span,
}
```

Each struct and enum clearly represents one concept from our language. A `Scene` contains `Passage`s, a `Passage` contains `PassageContent` items, and so on.

### Why Trees?

You might wonder: why go to all this trouble? Why not just interpret the tokens directly?

Trees are powerful for several reasons:

1. **Separation of concerns**: The parser deals with syntax (is this valid?), the runtime deals with semantics (what does it mean?). Each can be understood and modified independently.

2. **Multiple backends**: Once you have an AST, you can interpret it, compile it to machine code, transpile it to another language, or anything else. The AST is a universal intermediate representation.

3. **Tooling**: Linters, formatters, syntax highlighters, and code analyzers can all work with the AST without reimplementing parsing.

4. **Optimization**: You can transform the AST (simplify expressions, inline constants, etc.) before execution.

The AST is the clean handoff point between "understanding the text" and "doing something with it."

---

## Stage 4: Runtime Execution

**File: `src/runtime.rs`**

Finally, the moment of truth: we have a beautifully structured AST, and now we need to make it *do* something. The runtime walks through the tree and performs actions—this is where the program actually "runs."

### The EffectHandler: Your Game, Your Rules

Here's something clever about spween's design: it doesn't know anything about your game. It doesn't know what "gold" or "courage" or "inventory" mean. Instead, it asks *you* to define that through a trait:

```rust
pub trait EffectHandler {
    /// Get the value of a variable
    fn get_var(&self, name: &str) -> Value;

    /// Set the value of a variable
    fn set_var(&mut self, name: &str, value: Value);

    /// Check if a category has a specific key (like inventory.sword)
    fn has(&self, category: &str, key: &str) -> bool;

    /// Call a custom game function
    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String>;
}
```

This is *dependency injection*: the runtime provides the general machinery for executing narratives, and you plug in your game-specific logic. Want `gold` to be stored in a database? Want `play_sound` to trigger your audio system? Just implement the trait:

```rust
struct MyGame {
    variables: HashMap<String, Value>,
    inventory: HashSet<String>,
    audio: AudioSystem,
}

impl EffectHandler for MyGame {
    fn get_var(&self, name: &str) -> Value {
        self.variables.get(name).cloned().unwrap_or(Value::Null)
    }

    fn set_var(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    fn has(&self, category: &str, key: &str) -> bool {
        if category == "inventory" {
            self.inventory.contains(key)
        } else {
            false
        }
    }

    fn call(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
        match name {
            "play_sound" => {
                if let Some(Value::String(sound)) = args.get(0) {
                    self.audio.play(sound);
                }
                Ok(())
            }
            _ => Err(format!("Unknown function: {}", name))
        }
    }
}
```

### Evaluating Conditions

When the player sees a choice, we need to check if its condition is satisfied:

```rust
fn evaluate_condition(&self, condition: &Option<Condition>) -> bool {
    let Some(cond) = condition else {
        return true;  // No condition means always available
    };

    // All clauses must be true (they're ANDed together)
    cond.clauses.iter().all(|clause| self.evaluate_clause(clause))
}

fn evaluate_clause(&self, clause: &ConditionClause) -> bool {
    match clause {
        ConditionClause::Has(has) =>
            self.handler.has(&has.category, &has.key),

        ConditionClause::Compare(cmp) => {
            let var_value = self.handler.get_var(&cmp.var);
            self.compare_values(&var_value, cmp.op, &cmp.value)
        }

        ConditionClause::Not(inner) =>
            !self.evaluate_clause(inner),
    }
}
```

Notice how the code mirrors the AST structure? Each AST node type has a corresponding case in our evaluation logic. This pattern—matching on AST nodes and recursively processing children—is the heart of tree-walking interpretation.

### Executing Effects

When a player selects a choice, we run its effects:

```rust
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
                other => other,
            };
            self.handler.set_var(&modify.var, new_value);
        }

        Effect::Call(call) => {
            self.handler.call(&call.name, &call.args)?;
        }
    }
    Ok(())
}
```

Again, pattern matching on the AST. The structure of the code follows the structure of the data.

---

## Putting It All Together

Here's how a game might use spween:

```rust
use spween::{parse, Runtime, EffectHandler, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parse the scene file (Lexer → Parser → AST)
    let source = std::fs::read_to_string("tavern.scene")?;
    let scene = parse(&source, "tavern.scene")?;

    // 2. Create game state (implements EffectHandler)
    let mut game = MyGame::new();
    game.set_var("courage", Value::Int(10));
    game.set_var("gold", Value::Int(50));

    // 3. Create runtime (connects AST to game state)
    let mut runtime = Runtime::new(&scene, game)?;

    // 4. Game loop
    while !runtime.is_ended() {
        // Display prose
        if let Some(prose) = runtime.current_prose() {
            println!("\n{}\n", prose);
        }

        // Display choices
        let choices = runtime.current_choices();
        for (i, choice) in choices.iter().enumerate() {
            let marker = if choice.available { ">" } else { "x" };
            println!("{} {}. {}", marker, i + 1, choice.text);
        }

        // Get player input and execute choice
        let selection = get_player_input();
        runtime.select_choice(selection)?;
    }

    println!("Scene ended!");
    Ok(())
}
```

That's the complete pipeline: text → lexer → tokens → parser → AST → runtime → interactive narrative.

---

## Key Concepts Recap

You've just learned the fundamentals that power virtually every programming language implementation. Let's review:

### Lexer Takeaways
- Transforms raw text into tokens (meaningful atomic units)
- Uses patterns (literal strings or regular expressions) to recognize tokens
- Tracks source positions for error reporting
- Handles whitespace, comments, and structural elements like indentation

### Parser Takeaways
- Recursive descent: one function per grammar rule, calling each other as needed
- Builds hierarchical tree structure from flat token stream
- `consume()` demands specific tokens, `check()` peeks ahead
- Good error messages require careful span tracking

### AST Takeaways
- Abstract representation of program structure—meaning without syntactic noise
- Enums for variants (different kinds of things), structs for data
- Clean separation between syntax (parser's job) and semantics (runtime's job)
- Enables tooling, optimization, and multiple backends

### Runtime Takeaways
- Tree-walking interpreter: pattern match on AST nodes, recursively process children
- Trait-based extensibility lets users plug in their own game logic
- State machine manages control flow (which passage are we in?)
- Effects modify external state, conditions read it

---

## Where to Go From Here

Congratulations! You now understand the fundamental architecture that powers programming languages. Here are some paths forward:

**Learn by using:**
- [Getting Started](docs/01-getting-started.md) — Build your first interactive scene with spween
- [Examples](docs/06-examples.md) — See complete working games

**Learn by reading:**
- **[Crafting Interpreters](https://craftinginterpreters.com/)** by Robert Nystrom — The definitive free guide to building languages
- **Writing An Interpreter In Go** by Thorsten Ball — Practical and approachable
- **The logos crate** — The lexer generator spween uses

**Learn by building:**
- Add a new feature to spween (random selection? Variables in prose text?)
- Build your own tiny language from scratch
- Implement a different backend (compile to Lua? Generate flowcharts?)

---

*spween is designed to be embedded in games. The architecture shown here—lexer → parser → AST → runtime—is the same pattern used by everything from Python to JavaScript to your favorite game's scripting system. Now you know the secret!*
