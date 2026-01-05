//! Lexer for the spween DSL.
//!
//! Uses the `logos` crate for efficient tokenization.

use logos::{Logos, SpannedIter};
use smol_str::SmolStr;
use std::ops::Range;

use crate::Span;

/// Raw tokens produced by the logos lexer.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]
pub enum RawToken {
    #[token("---")]
    FrontmatterDelim,

    #[token("===")]
    PassageHeader,

    #[token("*")]
    ChoiceMarker,

    #[token("~")]
    EffectMarker,

    #[token("->")]
    Arrow,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token(".")]
    Dot,

    #[token("!")]
    Bang,

    #[token("?")]
    Question,

    #[token("'")]
    Apostrophe,

    #[token(";")]
    Semicolon,

    #[token("-", priority = 1)]
    Minus,

    #[token(">=")]
    Ge,

    #[token("<=")]
    Le,

    #[token("==")]
    EqEq,

    #[token("!=")]
    Ne,

    #[token(">")]
    Gt,

    #[token("<")]
    Lt,

    #[token("+=")]
    PlusEq,

    #[token("-=")]
    MinusEq,

    #[token("=")]
    Eq,

    #[regex(r"-?[0-9]+(\.[0-9]+)?", |lex| lex.slice().to_string())]
    Number(String),

    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    String(String),

    #[token("when")]
    When,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority = 1)]
    Identifier(String),

    #[token("\n")]
    Newline,

    #[regex(r"//[^\n]*", logos::skip)]
    Comment,
}

/// Processed tokens with additional context.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    FrontmatterDelim,
    PassageHeader(SmolStr),
    ChoiceMarker,
    EffectMarker,
    Arrow,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Comma,
    Colon,
    Dot,
    Bang,
    Question,
    Apostrophe,
    Semicolon,
    Minus,
    Ge,
    Le,
    EqEq,
    Ne,
    Gt,
    Lt,
    PlusEq,
    MinusEq,
    Eq,
    Number(i64),
    Float(f64),
    String(SmolStr),
    Identifier(SmolStr),
    When,
    Newline,
    Indent,
    Dedent,
    Eof,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::FrontmatterDelim => write!(f, "---"),
            Token::PassageHeader(name) => write!(f, "=== {}", name),
            Token::ChoiceMarker => write!(f, "*"),
            Token::EffectMarker => write!(f, "~"),
            Token::Arrow => write!(f, "->"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Dot => write!(f, "."),
            Token::Bang => write!(f, "!"),
            Token::Question => write!(f, "?"),
            Token::Apostrophe => write!(f, "'"),
            Token::Semicolon => write!(f, ";"),
            Token::Minus => write!(f, "-"),
            Token::Ge => write!(f, ">="),
            Token::Le => write!(f, "<="),
            Token::EqEq => write!(f, "=="),
            Token::Ne => write!(f, "!="),
            Token::Gt => write!(f, ">"),
            Token::Lt => write!(f, "<"),
            Token::PlusEq => write!(f, "+="),
            Token::MinusEq => write!(f, "-="),
            Token::Eq => write!(f, "="),
            Token::Number(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::When => write!(f, "when"),
            Token::Newline => write!(f, "\\n"),
            Token::Indent => write!(f, "INDENT"),
            Token::Dedent => write!(f, "DEDENT"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// A token with its source span.
#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

/// The lexer state machine.
pub struct Lexer<'src> {
    source: &'src str,
    inner: SpannedIter<'src, RawToken>,
    indent_stack: Vec<usize>,
    pending_dedents: usize,
    pos: usize,
}

impl<'src> Lexer<'src> {
    /// Create a new lexer for the given source.
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            inner: RawToken::lexer(source).spanned(),
            indent_stack: vec![0],
            pending_dedents: 0,
            pos: 0,
        }
    }

    fn next_raw(&mut self) -> Option<(Result<RawToken, ()>, Range<usize>)> {
        self.inner.next()
    }

    fn measure_indent_at(&self, pos: usize) -> usize {
        let mut indent = 0;
        for ch in self.source[pos..].chars() {
            match ch {
                ' ' => indent += 1,
                '\t' => indent += 2,
                _ => break,
            }
        }
        indent
    }

    fn read_passage_name(&mut self) -> SmolStr {
        if let Some((Ok(RawToken::Identifier(name)), _)) = self.next_raw() {
            SmolStr::new(&name)
        } else {
            SmolStr::default()
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = SpannedToken;

    fn next(&mut self) -> Option<Self::Item> {
        // Handle pending dedents
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            return Some(SpannedToken {
                token: Token::Dedent,
                span: self.pos..self.pos,
            });
        }

        let (tok_res, span) = self.next_raw()?;
        self.pos = span.end;

        let tok = match tok_res {
            Ok(t) => t,
            Err(_) => {
                // Skip unrecognized characters
                return self.next();
            }
        };

        match tok {
            RawToken::FrontmatterDelim => Some(SpannedToken {
                token: Token::FrontmatterDelim,
                span,
            }),

            RawToken::PassageHeader => {
                let name = self.read_passage_name();
                Some(SpannedToken {
                    token: Token::PassageHeader(name),
                    span: span.start..self.pos,
                })
            }

            RawToken::ChoiceMarker => Some(SpannedToken {
                token: Token::ChoiceMarker,
                span,
            }),

            RawToken::EffectMarker => Some(SpannedToken {
                token: Token::EffectMarker,
                span,
            }),

            RawToken::Arrow => Some(SpannedToken {
                token: Token::Arrow,
                span,
            }),

            RawToken::Newline => {
                // Handle indentation
                let indent = self.measure_indent_at(span.end);
                let current_indent = *self.indent_stack.last().unwrap_or(&0);

                if indent > current_indent {
                    self.indent_stack.push(indent);
                    return Some(SpannedToken {
                        token: Token::Indent,
                        span: span.end..span.end + indent,
                    });
                } else if indent < current_indent {
                    while self.indent_stack.len() > 1
                        && *self.indent_stack.last().unwrap_or(&0) > indent
                    {
                        self.indent_stack.pop();
                        self.pending_dedents += 1;
                    }
                    if self.pending_dedents > 0 {
                        self.pending_dedents -= 1;
                        return Some(SpannedToken {
                            token: Token::Dedent,
                            span: span.clone(),
                        });
                    }
                }

                Some(SpannedToken {
                    token: Token::Newline,
                    span,
                })
            }

            RawToken::Number(s) => {
                if s.contains('.') {
                    let n: f64 = s.parse().unwrap_or(0.0);
                    Some(SpannedToken {
                        token: Token::Float(n),
                        span,
                    })
                } else {
                    let n: i64 = s.parse().unwrap_or(0);
                    Some(SpannedToken {
                        token: Token::Number(n),
                        span,
                    })
                }
            }

            RawToken::String(s) => Some(SpannedToken {
                token: Token::String(SmolStr::new(&s)),
                span,
            }),

            RawToken::Identifier(s) => Some(SpannedToken {
                token: Token::Identifier(SmolStr::new(&s)),
                span,
            }),

            RawToken::LBracket => Some(SpannedToken { token: Token::LBracket, span }),
            RawToken::RBracket => Some(SpannedToken { token: Token::RBracket, span }),
            RawToken::LBrace => Some(SpannedToken { token: Token::LBrace, span }),
            RawToken::RBrace => Some(SpannedToken { token: Token::RBrace, span }),
            RawToken::LParen => Some(SpannedToken { token: Token::LParen, span }),
            RawToken::RParen => Some(SpannedToken { token: Token::RParen, span }),
            RawToken::Comma => Some(SpannedToken { token: Token::Comma, span }),
            RawToken::Colon => Some(SpannedToken { token: Token::Colon, span }),
            RawToken::Dot => Some(SpannedToken { token: Token::Dot, span }),
            RawToken::Bang => Some(SpannedToken { token: Token::Bang, span }),
            RawToken::Question => Some(SpannedToken { token: Token::Question, span }),
            RawToken::Apostrophe => Some(SpannedToken { token: Token::Apostrophe, span }),
            RawToken::Semicolon => Some(SpannedToken { token: Token::Semicolon, span }),
            RawToken::Minus => Some(SpannedToken { token: Token::Minus, span }),
            RawToken::Ge => Some(SpannedToken { token: Token::Ge, span }),
            RawToken::Le => Some(SpannedToken { token: Token::Le, span }),
            RawToken::EqEq => Some(SpannedToken { token: Token::EqEq, span }),
            RawToken::Ne => Some(SpannedToken { token: Token::Ne, span }),
            RawToken::Gt => Some(SpannedToken { token: Token::Gt, span }),
            RawToken::Lt => Some(SpannedToken { token: Token::Lt, span }),
            RawToken::PlusEq => Some(SpannedToken { token: Token::PlusEq, span }),
            RawToken::MinusEq => Some(SpannedToken { token: Token::MinusEq, span }),
            RawToken::Eq => Some(SpannedToken { token: Token::Eq, span }),
            RawToken::When => Some(SpannedToken { token: Token::When, span }),
            RawToken::Comment => self.next(),
        }
    }
}

/// Tokenize source code into a vector of spanned tokens.
pub fn lex(source: &str) -> Vec<SpannedToken> {
    let lexer = Lexer::new(source);
    let mut tokens: Vec<SpannedToken> = lexer.collect();
    tokens.push(SpannedToken {
        token: Token::Eof,
        span: source.len()..source.len(),
    });
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontmatter() {
        let source = "---\nid: test\n---";
        let tokens = lex(source);
        assert!(matches!(tokens[0].token, Token::FrontmatterDelim));
    }

    #[test]
    fn test_passage_header() {
        let source = "=== intro";
        let tokens = lex(source);
        assert!(matches!(&tokens[0].token, Token::PassageHeader(name) if name == "intro"));
    }

    #[test]
    fn test_choice() {
        let source = "* [Test choice]";
        let tokens = lex(source);
        assert!(matches!(tokens[0].token, Token::ChoiceMarker));
        assert!(matches!(tokens[1].token, Token::LBracket));
    }

    #[test]
    fn test_effect() {
        let source = "~ gold += 10";
        let tokens = lex(source);
        assert!(matches!(tokens[0].token, Token::EffectMarker));
        assert!(matches!(&tokens[1].token, Token::Identifier(s) if s == "gold"));
        assert!(matches!(tokens[2].token, Token::PlusEq));
        assert!(matches!(tokens[3].token, Token::Number(10)));
    }

    #[test]
    fn test_navigation() {
        let source = "-> END";
        let tokens = lex(source);
        assert!(matches!(tokens[0].token, Token::Arrow));
        assert!(matches!(&tokens[1].token, Token::Identifier(s) if s == "END"));
    }
}
