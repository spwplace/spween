//! Parser for the spween DSL.
//!
//! Parses tokenized source into an AST.

use smol_str::SmolStr;

use crate::{
    ast::*,
    error::ParseError,
    lexer::{lex, SpannedToken, Token},
    Value,
};

/// Parser state.
pub struct Parser {
    source: String,
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    /// Create a new parser for the given source.
    pub fn new(source: &str, _filename: &str) -> Self {
        let tokens = lex(source);
        Self {
            source: source.to_string(),
            tokens,
            pos: 0,
        }
    }

    fn current(&self) -> &SpannedToken {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().expect("Token stream should not be empty"))
    }

    fn current_token(&self) -> &Token {
        &self.current().token
    }

    fn current_span(&self) -> Span {
        self.current().span.clone()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_token(), Token::Eof)
    }

    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(self.current_token()) == std::mem::discriminant(token)
    }

    fn advance(&mut self) -> &SpannedToken {
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn consume(&mut self, expected: &Token, message: &str) -> Result<SpannedToken, ParseError> {
        if self.check(expected) {
            Ok(self.advance().clone())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: message.to_string(),
                found: format!("{}", self.current_token()),
                span: self.current_span(),
            })
        }
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_newlines(&mut self) {
        while self.match_token(&Token::Newline) {}
    }

    /// Parse the complete scene file.
    pub fn parse(&mut self) -> Result<Scene, ParseError> {
        self.skip_newlines();

        let meta = self.parse_frontmatter()?;
        let mut passages = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            if matches!(self.current_token(), Token::PassageHeader(_)) {
                passages.push(self.parse_passage()?);
            } else {
                self.advance();
            }
        }

        let span = 0..self.tokens.last().map(|t| t.span.end).unwrap_or(0);

        Ok(Scene {
            meta,
            passages,
            span,
        })
    }

    fn parse_frontmatter(&mut self) -> Result<SceneMeta, ParseError> {
        let start_span = self.current_span();
        let start_token =
            self.consume(&Token::FrontmatterDelim, "Expected --- to start frontmatter")?;

        // Find the end of frontmatter by looking for the next ---
        let yaml_start = start_token.span.end;
        let mut yaml_end = yaml_start;

        while !self.check(&Token::FrontmatterDelim) && !self.is_at_end() {
            yaml_end = self.current_span().end;
            self.advance();
        }

        let end_span = self.current_span();
        self.consume(&Token::FrontmatterDelim, "Expected --- to end frontmatter")?;
        self.skip_newlines();

        // Extract YAML from raw source
        let yaml_str = &self.source[yaml_start..yaml_end];
        let yaml_data: serde_yaml::Value =
            serde_yaml::from_str(yaml_str).map_err(|e| ParseError::InvalidFrontmatter {
                message: e.to_string(),
                span: start_span.clone(),
            })?;

        let id = yaml_data
            .get("id")
            .and_then(|v| v.as_str())
            .map(SmolStr::new)
            .unwrap_or_default();

        let title = yaml_data
            .get("title")
            .and_then(|v| v.as_str())
            .map(SmolStr::new)
            .unwrap_or_default();

        let tags = yaml_data
            .get("tags")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(SmolStr::new))
                    .collect()
            })
            .unwrap_or_default();

        let weight = yaml_data
            .get("weight")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as u32;

        let cooldown = yaml_data
            .get("cooldown")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as u32;

        // Parse requires block if present
        let requires = yaml_data
            .get("requires")
            .and_then(|v| v.as_mapping())
            .map(|m| self.parse_requires_from_yaml(m, &start_span));

        // Collect custom fields (anything not in the standard set)
        let standard_keys = ["id", "title", "tags", "weight", "cooldown", "requires", "context"];
        let custom: Vec<(SmolStr, Value)> = yaml_data
            .as_mapping()
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| {
                        let key = k.as_str()?;
                        if standard_keys.contains(&key) {
                            return None;
                        }
                        let value = yaml_value_to_value(v);
                        Some((SmolStr::new(key), value))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let span = start_span.start..end_span.end;

        Ok(SceneMeta {
            id,
            title,
            tags,
            weight,
            cooldown,
            requires,
            custom,
            span,
        })
    }

    fn parse_requires_from_yaml(
        &self,
        map: &serde_yaml::Mapping,
        span: &Span,
    ) -> Condition {
        let mut clauses = Vec::new();

        // Parse has requirements (category.key patterns)
        // e.g., has: [inventory.sword, skills.lockpick]
        if let Some(has_list) = map.get("has").and_then(|v| v.as_sequence()) {
            for item in has_list {
                if let Some(s) = item.as_str() {
                    if let Some((cat, key)) = s.split_once('.') {
                        clauses.push(ConditionClause::Has(HasClause {
                            category: SmolStr::new(cat),
                            key: SmolStr::new(key),
                            span: span.clone(),
                        }));
                    }
                }
            }
        }

        // Parse min requirements (variable >= value)
        // e.g., min: { gold: 100, health: 50 }
        if let Some(min_map) = map.get("min").and_then(|v| v.as_mapping()) {
            for (k, v) in min_map {
                if let (Some(var), Some(val)) = (k.as_str(), v.as_i64()) {
                    clauses.push(ConditionClause::Compare(CompareClause {
                        var: SmolStr::new(var),
                        op: CompareOp::Ge,
                        value: Value::Int(val),
                        span: span.clone(),
                    }));
                }
            }
        }

        // Parse max requirements (variable <= value)
        // e.g., max: { danger: 5 }
        if let Some(max_map) = map.get("max").and_then(|v| v.as_mapping()) {
            for (k, v) in max_map {
                if let (Some(var), Some(val)) = (k.as_str(), v.as_i64()) {
                    clauses.push(ConditionClause::Compare(CompareClause {
                        var: SmolStr::new(var),
                        op: CompareOp::Le,
                        value: Value::Int(val),
                        span: span.clone(),
                    }));
                }
            }
        }

        // Parse required flags (must be truthy)
        // e.g., flags: [visited_town, has_key]
        if let Some(flags) = map.get("flags").and_then(|v| v.as_sequence()) {
            for flag in flags {
                if let Some(name) = flag.as_str() {
                    clauses.push(ConditionClause::Compare(CompareClause {
                        var: SmolStr::new(name),
                        op: CompareOp::Eq,
                        value: Value::Bool(true),
                        span: span.clone(),
                    }));
                }
            }
        }

        // Parse excluded flags (must be falsy)
        // e.g., not: [game_over, quest_failed]
        if let Some(not_flags) = map.get("not").and_then(|v| v.as_sequence()) {
            for flag in not_flags {
                if let Some(name) = flag.as_str() {
                    clauses.push(ConditionClause::Not(Box::new(ConditionClause::Compare(
                        CompareClause {
                            var: SmolStr::new(name),
                            op: CompareOp::Eq,
                            value: Value::Bool(true),
                            span: span.clone(),
                        },
                    ))));
                }
            }
        }

        Condition {
            clauses,
            span: span.clone(),
        }
    }

    fn parse_passage(&mut self) -> Result<Passage, ParseError> {
        let header_token = self.advance().clone();
        let name = match &header_token.token {
            Token::PassageHeader(name) => name.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "passage header".to_string(),
                    found: format!("{}", header_token.token),
                    span: header_token.span,
                })
            }
        };

        self.skip_newlines();

        let mut content = Vec::new();
        let mut prose_lines = Vec::new();
        let mut prose_start: Option<Span> = None;

        let flush_prose = |content: &mut Vec<PassageContent>,
                          prose_lines: &mut Vec<String>,
                          prose_start: &mut Option<Span>| {
            if !prose_lines.is_empty() {
                let text = prose_lines.join("\n").trim().to_string();
                if !text.is_empty() {
                    content.push(PassageContent::Prose(Prose {
                        text: SmolStr::new(&text),
                        span: prose_start.clone().unwrap_or(0..0),
                    }));
                }
                prose_lines.clear();
                *prose_start = None;
            }
        };

        while !self.is_at_end() && !matches!(self.current_token(), Token::PassageHeader(_)) {
            self.skip_newlines();

            if self.is_at_end() || matches!(self.current_token(), Token::PassageHeader(_)) {
                break;
            }

            match self.current_token() {
                Token::ChoiceMarker => {
                    flush_prose(&mut content, &mut prose_lines, &mut prose_start);
                    content.push(PassageContent::Choice(self.parse_choice()?));
                }
                Token::Identifier(_) => {
                    if prose_start.is_none() {
                        prose_start = Some(self.current_span());
                    }
                    let line = self.collect_prose_line();
                    if !line.is_empty() {
                        prose_lines.push(line);
                    }
                }
                Token::Newline => {
                    self.advance();
                }
                Token::EffectMarker | Token::Arrow => {
                    // These shouldn't appear at passage level outside choices
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }

        flush_prose(&mut content, &mut prose_lines, &mut prose_start);

        let span = header_token.span.start..self.current_span().start;

        Ok(Passage {
            name,
            content,
            span,
        })
    }

    fn collect_prose_line(&mut self) -> String {
        let mut result = String::new();

        while !self.check(&Token::Newline)
            && !self.check(&Token::ChoiceMarker)
            && !self.check(&Token::EffectMarker)
            && !matches!(self.current_token(), Token::PassageHeader(_))
            && !self.is_at_end()
        {
            let (part, is_punctuation) = match self.current_token() {
                Token::Identifier(s) => (s.to_string(), false),
                Token::Number(n) => (n.to_string(), false),
                Token::String(s) => (format!("\"{}\"", s), false),
                Token::Dot => (".".to_string(), true),
                Token::Comma => (",".to_string(), true),
                Token::Colon => (":".to_string(), true),
                Token::Bang => ("!".to_string(), true),
                _ => (String::new(), false),
            };
            if !part.is_empty() {
                // Don't add space before punctuation
                if !result.is_empty() && !is_punctuation {
                    result.push(' ');
                }
                result.push_str(&part);
            }
            self.advance();
        }

        result
    }

    fn parse_choice(&mut self) -> Result<Choice, ParseError> {
        let start_span = self.current_span();
        self.consume(&Token::ChoiceMarker, "Expected * for choice")?;

        // Parse choice text: [text here]
        self.consume(&Token::LBracket, "Expected [ after *")?;

        let mut text_parts = Vec::new();
        while !self.check(&Token::RBracket) && !self.is_at_end() {
            if self.check(&Token::Newline) {
                break;
            }
            let part = match self.current_token() {
                Token::Identifier(s) => s.to_string(),
                Token::Number(n) => n.to_string(),
                Token::String(s) => s.to_string(),
                Token::Dot => ".".to_string(),
                Token::Comma => ",".to_string(),
                Token::Colon => ":".to_string(),
                Token::Bang => "!".to_string(),
                _ => String::new(),
            };
            if !part.is_empty() {
                text_parts.push(part);
            }
            self.advance();
        }
        let text = SmolStr::new(&text_parts.join(" "));
        self.consume(&Token::RBracket, "Expected ] to close choice text")?;

        // Optional condition: { ... } or `when <expr>`
        let condition = if self.check(&Token::LBrace) {
            Some(self.parse_condition()?)
        } else if self.check(&Token::When) {
            Some(self.parse_when_condition()?)
        } else {
            None
        };

        self.skip_newlines();

        // Parse effects and navigation (may be indented)
        let mut effects = Vec::new();
        let mut target = None;

        // Check for indent
        let in_indent = self.match_token(&Token::Indent);

        loop {
            self.skip_newlines();

            if in_indent && self.check(&Token::Dedent) {
                self.advance();
                break;
            }

            if self.check(&Token::EffectMarker) {
                effects.push(self.parse_effect()?);
            } else if self.check(&Token::Arrow) {
                target = Some(self.parse_navigation()?);
            } else if self.check(&Token::Newline) {
                self.advance();
            } else {
                break;
            }
        }

        // Handle non-indented navigation
        if target.is_none() && self.check(&Token::Arrow) {
            target = Some(self.parse_navigation()?);
        }

        let end_span = self.current_span();

        Ok(Choice {
            text,
            condition,
            effects,
            target,
            span: start_span.start..end_span.start,
        })
    }

    fn parse_condition(&mut self) -> Result<Condition, ParseError> {
        let start_span = self.current_span();
        self.consume(&Token::LBrace, "Expected {")?;

        let mut clauses = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            clauses.push(self.parse_condition_clause()?);

            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        let end_span = self.current_span();
        self.consume(&Token::RBrace, "Expected }")?;

        Ok(Condition {
            clauses,
            span: start_span.start..end_span.end,
        })
    }

    fn parse_condition_clause(&mut self) -> Result<ConditionClause, ParseError> {
        let start_span = self.current_span();

        // Check for negation
        let negated = self.match_token(&Token::Bang);

        // Expect identifier
        let first = match self.current_token() {
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                return Err(ParseError::InvalidCondition {
                    message: "Expected identifier".to_string(),
                    span: self.current_span(),
                })
            }
        };

        // Check for has condition: category.key
        if self.check(&Token::Dot) {
            self.advance();
            let key = match self.current_token() {
                Token::Identifier(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                _ => {
                    return Err(ParseError::InvalidCondition {
                        message: "Expected key name after .".to_string(),
                        span: self.current_span(),
                    })
                }
            };

            let clause = ConditionClause::Has(HasClause {
                category: first,
                key,
                span: start_span.start..self.current_span().start,
            });

            return Ok(if negated {
                ConditionClause::Not(Box::new(clause))
            } else {
                clause
            });
        }

        // Check for comparison: var op value
        if let Some(op) = self.try_parse_compare_op() {
            let value = self.parse_value()?;
            let clause = ConditionClause::Compare(CompareClause {
                var: first,
                op,
                value,
                span: start_span.start..self.current_span().start,
            });

            return Ok(if negated {
                ConditionClause::Not(Box::new(clause))
            } else {
                clause
            });
        }

        // Default: simple truthy check (var == true)
        let clause = ConditionClause::Compare(CompareClause {
            var: first,
            op: CompareOp::Eq,
            value: Value::Bool(true),
            span: start_span.start..self.current_span().start,
        });

        Ok(if negated {
            ConditionClause::Not(Box::new(clause))
        } else {
            clause
        })
    }

    fn try_parse_compare_op(&mut self) -> Option<CompareOp> {
        let op = match self.current_token() {
            Token::Ge => Some(CompareOp::Ge),
            Token::Le => Some(CompareOp::Le),
            Token::Gt => Some(CompareOp::Gt),
            Token::Lt => Some(CompareOp::Lt),
            Token::EqEq => Some(CompareOp::Eq),
            Token::Ne => Some(CompareOp::Ne),
            _ => None,
        };

        if op.is_some() {
            self.advance();
        }

        op
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        match self.current_token() {
            Token::Number(n) => {
                let n = *n;
                self.advance();
                Ok(Value::Int(n))
            }
            Token::Float(f) => {
                let f = *f;
                self.advance();
                Ok(Value::Float(f))
            }
            Token::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Value::String(s))
            }
            Token::Identifier(s) if s == "true" => {
                self.advance();
                Ok(Value::Bool(true))
            }
            Token::Identifier(s) if s == "false" => {
                self.advance();
                Ok(Value::Bool(false))
            }
            Token::Identifier(s) if s == "null" => {
                self.advance();
                Ok(Value::Null)
            }
            _ => Err(ParseError::InvalidCondition {
                message: "Expected value".to_string(),
                span: self.current_span(),
            }),
        }
    }

    fn parse_effect(&mut self) -> Result<Effect, ParseError> {
        let start_span = self.current_span();
        self.consume(&Token::EffectMarker, "Expected ~ for effect")?;

        let keyword = match self.current_token() {
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                return Err(ParseError::InvalidEffect {
                    message: "Expected effect keyword or variable".to_string(),
                    span: self.current_span(),
                })
            }
        };

        // Check for call effect: call("name", args...)
        if keyword.as_str() == "call" {
            return self.parse_call_effect(start_span);
        }

        // Check for assignment operators (variable effects)
        if self.check(&Token::PlusEq) {
            self.advance();
            let value = self.parse_number()?;
            self.skip_newlines();
            return Ok(Effect::Modify(ModifyEffect {
                var: keyword,
                delta: value,
                span: start_span.start..self.current_span().start,
            }));
        }

        if self.check(&Token::MinusEq) {
            self.advance();
            let value = self.parse_number()?;
            self.skip_newlines();
            return Ok(Effect::Modify(ModifyEffect {
                var: keyword,
                delta: -value,
                span: start_span.start..self.current_span().start,
            }));
        }

        if self.check(&Token::Eq) {
            self.advance();
            let value = self.parse_value()?;
            self.skip_newlines();
            return Ok(Effect::Set(SetEffect {
                var: keyword,
                value,
                span: start_span.start..self.current_span().start,
            }));
        }

        // Legacy call syntax: ~ keyword arg1 arg2 ...
        // e.g., ~ damage hull 10 -> call("damage", ["hull", 10])
        let mut args = Vec::new();

        // Collect remaining tokens on the line as arguments
        while !self.check(&Token::Newline) && !self.is_at_end() {
            match self.current_token() {
                Token::Number(n) => {
                    args.push(Value::Int(*n));
                    self.advance();
                }
                Token::Float(f) => {
                    args.push(Value::Float(*f));
                    self.advance();
                }
                Token::String(s) => {
                    args.push(Value::String(s.clone()));
                    self.advance();
                }
                Token::Identifier(s) => {
                    // Could be a string arg or boolean
                    match s.as_str() {
                        "true" => args.push(Value::Bool(true)),
                        "false" => args.push(Value::Bool(false)),
                        _ => args.push(Value::String(s.clone())),
                    }
                    self.advance();
                }
                _ => break,
            }
        }

        self.skip_newlines();

        Ok(Effect::Call(CallEffect {
            name: keyword,
            args,
            span: start_span.start..self.current_span().start,
        }))
    }

    fn parse_call_effect(&mut self, start_span: Span) -> Result<Effect, ParseError> {
        self.consume(&Token::LParen, "Expected ( after call")?;

        // Parse function name (string)
        let name = match self.current_token() {
            Token::String(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                return Err(ParseError::InvalidEffect {
                    message: "Expected function name in quotes".to_string(),
                    span: self.current_span(),
                })
            }
        };

        let mut args = Vec::new();

        // Parse arguments
        while self.match_token(&Token::Comma) {
            args.push(self.parse_value()?);
        }

        self.consume(&Token::RParen, "Expected )")?;
        self.skip_newlines();

        Ok(Effect::Call(CallEffect {
            name,
            args,
            span: start_span.start..self.current_span().start,
        }))
    }

    fn parse_number(&mut self) -> Result<i64, ParseError> {
        match self.current_token() {
            Token::Number(n) => {
                let n = *n;
                self.advance();
                Ok(n)
            }
            Token::Float(f) => {
                let n = *f as i64;
                self.advance();
                Ok(n)
            }
            _ => Err(ParseError::InvalidEffect {
                message: "Expected number".to_string(),
                span: self.current_span(),
            }),
        }
    }

    fn parse_navigation(&mut self) -> Result<NavigationTarget, ParseError> {
        let start_span = self.current_span();
        self.consume(&Token::Arrow, "Expected ->")?;

        let target = match self.current_token() {
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "passage name or END".to_string(),
                    found: format!("{}", self.current_token()),
                    span: self.current_span(),
                })
            }
        };

        let is_end = target.as_str() == "END";
        self.skip_newlines();

        Ok(NavigationTarget {
            target,
            is_end,
            span: start_span.start..self.current_span().start,
        })
    }

    fn parse_when_condition(&mut self) -> Result<Condition, ParseError> {
        let start_span = self.current_span();
        self.consume(&Token::When, "Expected 'when'")?;

        let mut clauses = Vec::new();

        while !self.check(&Token::Newline) && !self.is_at_end() {
            clauses.push(self.parse_condition_clause()?);

            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(Condition {
            clauses,
            span: start_span.start..self.current_span().start,
        })
    }
}

/// Convert a YAML value to a spween Value.
fn yaml_value_to_value(v: &serde_yaml::Value) -> Value {
    match v {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(b) => Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        serde_yaml::Value::String(s) => Value::String(SmolStr::new(s)),
        _ => Value::Null, // Sequences and mappings not supported as values
    }
}

/// Parse a scene file from source code.
pub fn parse(source: &str, filename: &str) -> Result<Scene, ParseError> {
    let mut parser = Parser::new(source, filename);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_scene() {
        let source = r#"---
id: test_scene
title: Test Scene
tags: [test]
weight: 10
cooldown: 5
---

=== intro

This is a test.

* [First choice]
  ~ gold += 10
  -> END

* [Second choice]
  -> END
"#;

        let result = parse(source, "test.scene");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let scene = result.unwrap();
        assert_eq!(scene.meta.id.as_str(), "test_scene");
        assert_eq!(scene.meta.title.as_str(), "Test Scene");
        assert_eq!(scene.passages.len(), 1);
        assert_eq!(scene.passages[0].name.as_str(), "intro");
    }

    #[test]
    fn test_parse_choice_with_has_condition() {
        let source = r#"---
id: cond_test
title: Condition Test
tags: []
weight: 10
cooldown: 5
---

=== intro

Test passage.

* [Tagged choice] { inventory.sword }
  -> END
"#;

        let result = parse(source, "test.scene");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let scene = result.unwrap();
        let passage = &scene.passages[0];

        if let PassageContent::Choice(choice) = &passage.content[1] {
            assert!(choice.condition.is_some());
            let condition = choice.condition.as_ref().unwrap();
            assert_eq!(condition.clauses.len(), 1);

            if let ConditionClause::Has(has) = &condition.clauses[0] {
                assert_eq!(has.category.as_str(), "inventory");
                assert_eq!(has.key.as_str(), "sword");
            } else {
                panic!("Expected Has condition");
            }
        } else {
            panic!("Expected choice");
        }
    }

    #[test]
    fn test_parse_multiple_effects() {
        let source = r#"---
id: effect_test
title: Effect Test
tags: []
weight: 10
cooldown: 5
---

=== intro

Test passage.

* [Do something]
  ~ damage hull 10
  ~ visited = true
  ~ gold += 50
  -> END
"#;

        let result = parse(source, "test.scene");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let scene = result.unwrap();
        let passage = &scene.passages[0];

        if let PassageContent::Choice(choice) = &passage.content[1] {
            assert_eq!(choice.effects.len(), 3);

            assert!(matches!(&choice.effects[0], Effect::Call(_)));
            assert!(matches!(&choice.effects[1], Effect::Set(_)));
            assert!(matches!(&choice.effects[2], Effect::Modify(_)));
        } else {
            panic!("Expected choice");
        }
    }

    #[test]
    fn test_parse_when_condition() {
        let source = r#"---
id: when_test
title: When Test
tags: []
weight: 10
cooldown: 5
---

=== intro

Test passage.

* [Choice with when] when gold >= 100
  -> END
"#;

        let result = parse(source, "test.scene");
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let scene = result.unwrap();
        let passage = &scene.passages[0];

        if let PassageContent::Choice(choice) = &passage.content[1] {
            assert!(choice.condition.is_some());
            let condition = choice.condition.as_ref().unwrap();
            assert_eq!(condition.clauses.len(), 1);

            if let ConditionClause::Compare(cmp) = &condition.clauses[0] {
                assert_eq!(cmp.var.as_str(), "gold");
                assert_eq!(cmp.op, CompareOp::Ge);
                assert_eq!(cmp.value, Value::Int(100));
            } else {
                panic!("Expected Compare condition");
            }
        } else {
            panic!("Expected choice");
        }
    }
}
