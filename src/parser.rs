//! KannakaHDL parser — a deliberately small language (ADR-0001):
//!
//! ```text
//! # a memory bank grown from discovered primitives
//! cell MemoryBank(n) {
//!     when n > 1  => split MemoryBank(n / 2), MemoryBank(n / 2) bridge "Harmonic Bridge"
//!     when always => base "Memory Seed" min_persistence 0.4 material "metamaterial"
//! }
//! grow MemoryBank(8)
//! ```
//!
//! Rules fire first-match. `split` recursively subdivides space; `base`
//! is a *registry query*, never a hand-drawn structure — grown
//! architectures are built from discovered primitives (PRD H2/H3).

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[error("line {line}: {message}")]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Param(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Guard {
    Always,
    Compare(Expr, Cmp, Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub cell: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Query {
    pub class: String,
    pub min_persistence: f64,
    pub material: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Split {
        children: Vec<Call>,
        bridge: Option<String>,
    },
    Base(Query),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub guard: Guard,
    pub action: Action,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellDef {
    pub name: String,
    pub params: Vec<String>,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub cells: Vec<CellDef>,
    pub grows: Vec<Call>,
}

/* ------------------------------- lexer ------------------------------- */

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    Punct(&'static str),
}

fn lex(source: &str) -> Result<Vec<(Tok, usize)>, ParseError> {
    let mut out = Vec::new();
    for (lineno, raw) in source.lines().enumerate() {
        let line = lineno + 1;
        let text = raw.split('#').next().unwrap_or("");
        let mut chars = text.chars().peekable();
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else if c == '"' {
                chars.next();
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some(ch) => s.push(ch),
                        None => {
                            return Err(ParseError {
                                line,
                                message: "unterminated string".into(),
                            })
                        }
                    }
                }
                out.push((Tok::Str(s), line));
            } else if c.is_ascii_digit() {
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' {
                        s.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if s.contains('.') {
                    let v = s.parse().map_err(|_| ParseError {
                        line,
                        message: format!("bad number {s}"),
                    })?;
                    out.push((Tok::Float(v), line));
                } else {
                    let v = s.parse().map_err(|_| ParseError {
                        line,
                        message: format!("bad number {s}"),
                    })?;
                    out.push((Tok::Int(v), line));
                }
            } else if c.is_alphabetic() || c == '_' {
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        s.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                out.push((Tok::Ident(s), line));
            } else {
                chars.next();
                let follows_eq = chars.peek() == Some(&'=');
                let follows_gt = chars.peek() == Some(&'>');
                let tok = match c {
                    '=' if follows_gt => {
                        chars.next();
                        "=>"
                    }
                    '=' if follows_eq => {
                        chars.next();
                        "=="
                    }
                    '!' if follows_eq => {
                        chars.next();
                        "!="
                    }
                    '<' if follows_eq => {
                        chars.next();
                        "<="
                    }
                    '>' if follows_eq => {
                        chars.next();
                        ">="
                    }
                    '<' => "<",
                    '>' => ">",
                    '(' => "(",
                    ')' => ")",
                    '{' => "{",
                    '}' => "}",
                    ',' => ",",
                    '+' => "+",
                    '-' => "-",
                    '*' => "*",
                    '/' => "/",
                    other => {
                        return Err(ParseError {
                            line,
                            message: format!("unexpected character: {other}"),
                        })
                    }
                };
                out.push((Tok::Punct(tok), line));
            }
        }
    }
    Ok(out)
}

/* ------------------------------ parser ------------------------------- */

struct Parser {
    toks: Vec<(Tok, usize)>,
    pos: usize,
}

impl Parser {
    fn line(&self) -> usize {
        self.toks
            .get(self.pos.min(self.toks.len().saturating_sub(1)))
            .map(|t| t.1)
            .unwrap_or(0)
    }

    fn err(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            line: self.line(),
            message: message.into(),
        }
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.0)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).map(|t| t.0.clone());
        self.pos += 1;
        t
    }

    fn expect_punct(&mut self, p: &str) -> Result<(), ParseError> {
        match self.next() {
            Some(Tok::Punct(got)) if got == p => Ok(()),
            other => Err(self.err(format!("expected `{p}`, got {other:?}"))),
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.next() {
            Some(Tok::Ident(s)) => Ok(s),
            other => Err(self.err(format!("expected identifier, got {other:?}"))),
        }
    }

    fn expect_str(&mut self) -> Result<String, ParseError> {
        match self.next() {
            Some(Tok::Str(s)) => Ok(s),
            other => Err(self.err(format!("expected string literal, got {other:?}"))),
        }
    }

    fn eat_punct(&mut self, p: &str) -> bool {
        if self.peek()
            == Some(&Tok::Punct(match p {
                "(" => "(",
                ")" => ")",
                "{" => "{",
                "}" => "}",
                "," => ",",
                "+" => "+",
                "-" => "-",
                "*" => "*",
                "/" => "/",
                _ => return false,
            }))
        {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_term()?;
        loop {
            if self.eat_punct("+") {
                lhs = Expr::Add(Box::new(lhs), Box::new(self.parse_term()?));
            } else if self.eat_punct("-") {
                lhs = Expr::Sub(Box::new(lhs), Box::new(self.parse_term()?));
            } else {
                return Ok(lhs);
            }
        }
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_factor()?;
        loop {
            if self.eat_punct("*") {
                lhs = Expr::Mul(Box::new(lhs), Box::new(self.parse_factor()?));
            } else if self.eat_punct("/") {
                lhs = Expr::Div(Box::new(lhs), Box::new(self.parse_factor()?));
            } else {
                return Ok(lhs);
            }
        }
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        match self.next() {
            Some(Tok::Int(v)) => Ok(Expr::Int(v)),
            Some(Tok::Ident(name)) => Ok(Expr::Param(name)),
            Some(Tok::Punct("(")) => {
                let e = self.parse_expr()?;
                self.expect_punct(")")?;
                Ok(e)
            }
            other => Err(self.err(format!("expected expression, got {other:?}"))),
        }
    }

    fn parse_guard(&mut self) -> Result<Guard, ParseError> {
        if self.peek() == Some(&Tok::Ident("always".into())) {
            self.pos += 1;
            return Ok(Guard::Always);
        }
        let lhs = self.parse_expr()?;
        let cmp = match self.next() {
            Some(Tok::Punct("<")) => Cmp::Lt,
            Some(Tok::Punct("<=")) => Cmp::Le,
            Some(Tok::Punct(">")) => Cmp::Gt,
            Some(Tok::Punct(">=")) => Cmp::Ge,
            Some(Tok::Punct("==")) => Cmp::Eq,
            Some(Tok::Punct("!=")) => Cmp::Ne,
            other => return Err(self.err(format!("expected comparison, got {other:?}"))),
        };
        let rhs = self.parse_expr()?;
        Ok(Guard::Compare(lhs, cmp, rhs))
    }

    fn parse_call(&mut self) -> Result<Call, ParseError> {
        let cell = self.expect_ident()?;
        self.expect_punct("(")?;
        let mut args = Vec::new();
        if self.peek() != Some(&Tok::Punct(")")) {
            loop {
                args.push(self.parse_expr()?);
                if !self.eat_punct(",") {
                    break;
                }
            }
        }
        self.expect_punct(")")?;
        Ok(Call { cell, args })
    }

    fn parse_action(&mut self) -> Result<Action, ParseError> {
        match self.next() {
            Some(Tok::Ident(kw)) if kw == "split" => {
                let mut children = vec![self.parse_call()?];
                while self.eat_punct(",") {
                    children.push(self.parse_call()?);
                }
                let mut bridge = None;
                if self.peek() == Some(&Tok::Ident("bridge".into())) {
                    self.pos += 1;
                    bridge = Some(self.expect_str()?);
                }
                Ok(Action::Split { children, bridge })
            }
            Some(Tok::Ident(kw)) if kw == "base" => {
                let class = self.expect_str()?;
                let mut query = Query {
                    class,
                    min_persistence: 0.0,
                    material: None,
                };
                loop {
                    match self.peek() {
                        Some(Tok::Ident(k)) if k == "min_persistence" => {
                            self.pos += 1;
                            query.min_persistence = match self.next() {
                                Some(Tok::Float(v)) => v,
                                Some(Tok::Int(v)) => v as f64,
                                other => {
                                    return Err(self.err(format!("expected number, got {other:?}")))
                                }
                            };
                        }
                        Some(Tok::Ident(k)) if k == "material" => {
                            self.pos += 1;
                            query.material = Some(self.expect_str()?);
                        }
                        _ => break,
                    }
                }
                Ok(Action::Base(query))
            }
            other => Err(self.err(format!("expected `split` or `base`, got {other:?}"))),
        }
    }
}

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let toks = lex(source)?;
    let mut p = Parser { toks, pos: 0 };
    let mut cells = Vec::new();
    let mut grows = Vec::new();

    while let Some(tok) = p.peek().cloned() {
        match tok {
            Tok::Ident(kw) if kw == "cell" => {
                p.pos += 1;
                let name = p.expect_ident()?;
                p.expect_punct("(")?;
                let mut params = Vec::new();
                if p.peek() != Some(&Tok::Punct(")")) {
                    loop {
                        params.push(p.expect_ident()?);
                        if !p.eat_punct(",") {
                            break;
                        }
                    }
                }
                p.expect_punct(")")?;
                p.expect_punct("{")?;
                let mut rules = Vec::new();
                while p.peek() != Some(&Tok::Punct("}")) {
                    let line = p.line();
                    match p.next() {
                        Some(Tok::Ident(kw)) if kw == "when" => {}
                        other => return Err(p.err(format!("expected `when`, got {other:?}"))),
                    }
                    let guard = p.parse_guard()?;
                    p.expect_punct("=>")?;
                    let action = p.parse_action()?;
                    rules.push(Rule {
                        guard,
                        action,
                        line,
                    });
                }
                p.expect_punct("}")?;
                if rules.is_empty() {
                    return Err(p.err(format!("cell {name} has no rules")));
                }
                cells.push(CellDef {
                    name,
                    params,
                    rules,
                });
            }
            Tok::Ident(kw) if kw == "grow" => {
                p.pos += 1;
                grows.push(p.parse_call()?);
            }
            other => return Err(p.err(format!("expected `cell` or `grow`, got {other:?}"))),
        }
    }
    if grows.is_empty() {
        return Err(ParseError {
            line: 0,
            message: "no `grow` statement".into(),
        });
    }
    Ok(Program { cells, grows })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROGRAM: &str = r#"
        # memory bank
        cell MemoryBank(n) {
            when n > 1  => split MemoryBank(n / 2), MemoryBank(n / 2) bridge "Harmonic Bridge"
            when always => base "Memory Seed" min_persistence 0.4 material "metamaterial"
        }
        grow MemoryBank(8)
    "#;

    #[test]
    fn parses_the_canonical_program() {
        let prog = parse(PROGRAM).unwrap();
        assert_eq!(prog.cells.len(), 1);
        assert_eq!(prog.cells[0].name, "MemoryBank");
        assert_eq!(prog.cells[0].params, vec!["n"]);
        assert_eq!(prog.cells[0].rules.len(), 2);
        match &prog.cells[0].rules[0].action {
            Action::Split { children, bridge } => {
                assert_eq!(children.len(), 2);
                assert_eq!(bridge.as_deref(), Some("Harmonic Bridge"));
            }
            other => panic!("expected split, got {other:?}"),
        }
        match &prog.cells[0].rules[1].action {
            Action::Base(q) => {
                assert_eq!(q.class, "Memory Seed");
                assert!((q.min_persistence - 0.4).abs() < 1e-9);
                assert_eq!(q.material.as_deref(), Some("metamaterial"));
            }
            other => panic!("expected base, got {other:?}"),
        }
        assert_eq!(prog.grows.len(), 1);
    }

    #[test]
    fn reports_errors_with_line_numbers() {
        let err = parse("cell X() {\n  when n > => base \"Y\"\n}\ngrow X()").unwrap_err();
        assert_eq!(err.line, 2);

        let err = parse("frobnicate").unwrap_err();
        assert!(err.message.contains("expected `cell` or `grow`"));

        let err = parse("cell X() { when always => base \"Y\" }").unwrap_err();
        assert!(err.message.contains("no `grow`"));
    }

    #[test]
    fn expressions_nest_with_precedence() {
        let prog =
            parse("cell X(n, m) { when n + m * 2 > (n - 1) / 2 => base \"Seed\" }\ngrow X(4, 2)")
                .unwrap();
        assert_eq!(prog.cells[0].params.len(), 2);
    }
}
