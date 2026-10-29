//! The expressions of OptiFine CEM / EMF animations (`"head.rx": "sin(age)"`):
//! numbers, booleans as 1 and 0, names, the C operators and the functions
//! both mods share. A name is resolved once, at compile time, to a slot of the
//! flat register file the animation reads and writes, so evaluating is a walk
//! over a small tree with no lookups.
use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}
impl Op {
    /// Binding strength as EMF reduces it: `&&` and `||` share the lowest
    /// level (`a || b && c` is `(a || b) && c`), every comparison the next,
    /// then `+ -`, then `* / %`; each level left to right.
    fn precedence(self) -> u8 {
        match self {
            Op::Or | Op::And => 1,
            Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge => 2,
            Op::Add | Op::Sub => 3,
            Op::Mul | Op::Div | Op::Rem => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Func {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    ToRad,
    ToDeg,
    Min,
    Max,
    Clamp,
    Abs,
    Floor,
    Ceil,
    Round,
    Exp,
    Log,
    Pow,
    Sqrt,
    Frac,
    Fmod,
    Signum,
    Random,
    If,
    Between,
    Equals,
    In,
    Lerp,
    WrapDeg,
    WrapRad,
    DegDiff,
    RadDiff,
    /// `catch` gives its first argument through.
    First,
}
impl Func {
    fn named(name: &str) -> Option<Self> {
        Some(match name {
            "sin" => Func::Sin,
            "cos" => Func::Cos,
            "tan" => Func::Tan,
            "asin" => Func::Asin,
            "acos" => Func::Acos,
            "atan" => Func::Atan,
            "atan2" => Func::Atan2,
            "torad" => Func::ToRad,
            "todeg" => Func::ToDeg,
            "min" => Func::Min,
            "max" => Func::Max,
            "clamp" => Func::Clamp,
            "abs" => Func::Abs,
            "floor" => Func::Floor,
            "ceil" => Func::Ceil,
            "round" => Func::Round,
            "exp" => Func::Exp,
            "log" => Func::Log,
            "pow" => Func::Pow,
            "sqrt" => Func::Sqrt,
            "frac" => Func::Frac,
            "fmod" => Func::Fmod,
            "signum" => Func::Signum,
            "random" => Func::Random,
            "if" | "ifb" => Func::If,
            "between" => Func::Between,
            "equals" => Func::Equals,
            "in" => Func::In,
            "lerp" => Func::Lerp,
            "wrapdeg" => Func::WrapDeg,
            "wraprad" => Func::WrapRad,
            "degdiff" => Func::DegDiff,
            "raddiff" => Func::RadDiff,
            "catch" => Func::First,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Const(f32),
    /// A register: an entity value, a part channel or a `var.`.
    Load(u16),
    Neg(Box<Node>),
    Not(Box<Node>),
    Bin(Op, Box<Node>, Box<Node>),
    Call(Func, Vec<Node>),
}

pub fn truth(v: f32) -> bool {
    v != 0.0 && !v.is_nan()
}
fn bool_value(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

thread_local! {
    /// `random()` without a seed: a fresh value at every call.
    static NOISE: Cell<u64> = const { Cell::new(0x9E37_79B9_7F4A_7C15) };
}
/// `random(seed)`: EMF's integer hash of the seed's bits, in [0, 1].
pub fn seeded(seed: f32) -> f32 {
    let mut x = seed.to_bits() as i32;
    x = x ^ 61 ^ (x >> 16);
    x = x.wrapping_add(x << 3);
    x ^= x >> 4;
    x = x.wrapping_mul(668_265_261);
    x ^= x >> 15;
    x.unsigned_abs() as f32 / 2.147_483_6e9
}
/// `Mth.wrapDegrees`.
fn wrap_degrees(v: f32) -> f32 {
    let mut r = v % 360.0;
    if r >= 180.0 {
        r -= 360.0;
    }
    if r < -180.0 {
        r += 360.0;
    }
    r
}

impl Node {
    pub fn eval(&self, regs: &[f32]) -> f32 {
        match self {
            Node::Const(v) => *v,
            Node::Load(slot) => regs[*slot as usize],
            Node::Neg(a) => -a.eval(regs),
            Node::Not(a) => bool_value(!truth(a.eval(regs))),
            Node::Bin(Op::And, a, b) => bool_value(truth(a.eval(regs)) && truth(b.eval(regs))),
            Node::Bin(Op::Or, a, b) => bool_value(truth(a.eval(regs)) || truth(b.eval(regs))),
            Node::Bin(op, a, b) => {
                let (a, b) = (a.eval(regs), b.eval(regs));
                match op {
                    Op::Add => a + b,
                    Op::Sub => a - b,
                    Op::Mul => a * b,
                    Op::Div => a / b,
                    Op::Rem => a % b,
                    Op::Lt => bool_value(a < b),
                    Op::Le => bool_value(a <= b),
                    Op::Gt => bool_value(a > b),
                    Op::Ge => bool_value(a >= b),
                    Op::Eq => bool_value(a == b),
                    Op::Ne => bool_value(a != b),
                    Op::And | Op::Or => unreachable!(),
                }
            }
            Node::Call(f, args) => call(*f, args, regs),
        }
    }
}

fn call(f: Func, args: &[Node], regs: &[f32]) -> f32 {
    let arg = |i: usize| args.get(i).map_or(0.0, |a| a.eval(regs));
    match f {
        // if(c1, v1, c2, v2, ..., else): only the chosen branch runs.
        Func::If => {
            let mut pairs = args.chunks_exact(2);
            for pair in pairs.by_ref() {
                if truth(pair[0].eval(regs)) {
                    return pair[1].eval(regs);
                }
            }
            pairs.remainder().first().map_or(0.0, |e| e.eval(regs))
        }
        // Minecraft's lookup table, as the mods read it.
        Func::Sin => anoma_core::geometry::trig::sin(f64::from(arg(0))),
        Func::Cos => anoma_core::geometry::trig::cos(f64::from(arg(0))),
        Func::Tan => arg(0).tan(),
        Func::Asin => arg(0).asin(),
        Func::Acos => arg(0).acos(),
        Func::Atan => arg(0).atan(),
        Func::Atan2 => arg(0).atan2(arg(1)),
        Func::ToRad => arg(0).to_radians(),
        Func::ToDeg => arg(0).to_degrees(),
        Func::Min => args
            .iter()
            .map(|a| a.eval(regs))
            .fold(f32::INFINITY, f32::min),
        Func::Max => args
            .iter()
            .map(|a| a.eval(regs))
            .fold(f32::NEG_INFINITY, f32::max),
        Func::Clamp => arg(0).max(arg(1)).min(arg(2)),
        Func::Abs => arg(0).abs(),
        Func::Floor => arg(0).floor(),
        Func::Ceil => arg(0).ceil(),
        Func::Round => (arg(0) + 0.5).floor(),
        Func::Exp => arg(0).exp(),
        Func::Log => arg(0).ln(),
        Func::Pow => arg(0).powf(arg(1)),
        Func::Sqrt => arg(0).sqrt(),
        Func::Frac => arg(0).fract(),
        // Math.floorMod on the values cut to integers.
        Func::Fmod => {
            let (a, b) = (arg(0) as i32, arg(1) as i32);
            if b == 0 {
                return f32::NAN;
            }
            let m = a.wrapping_rem(b);
            (if m != 0 && (m < 0) != (b < 0) {
                m + b
            } else {
                m
            }) as f32
        }
        Func::Signum => {
            let v = arg(0);
            if v == 0.0 { 0.0 } else { v.signum() }
        }
        Func::Random => match args.first() {
            Some(seed) => seeded(seed.eval(regs)),
            None => NOISE.with(|n| {
                let mut x = n.get();
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                n.set(x);
                (x >> 40) as f32 / (1u64 << 24) as f32
            }),
        },
        // between(x, lo, hi): !(x > hi) && !(x < lo).
        Func::Between => {
            let x = arg(0);
            bool_value(!(x > arg(2)) && !(x < arg(1)))
        }
        Func::Equals => bool_value((arg(0) - arg(1)).abs() <= arg(2)),
        Func::In => {
            let v = arg(0);
            bool_value(args[1..].iter().any(|a| a.eval(regs) == v))
        }
        Func::Lerp => {
            let t = arg(0);
            arg(1) + (arg(2) - arg(1)) * t
        }
        Func::WrapDeg => wrap_degrees(arg(0)),
        Func::WrapRad => wrap_degrees(arg(0).to_degrees()).to_radians(),
        Func::DegDiff => wrap_degrees(arg(1) - arg(0)).abs(),
        Func::RadDiff => wrap_degrees(arg(1).to_degrees() - arg(0).to_degrees())
            .abs()
            .to_radians(),
        Func::First => arg(0),
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Num(f32),
    Name(String),
    Str,
    Op(&'static str),
    Open,
    Close,
    Comma,
}

fn tokens(text: &str) -> Result<Vec<Token>, String> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || c == '.' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit)
        {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            // A letter right after digits continues a name (`varb.21_2_plus`
            // is read whole below); a bare number ends here.
            let n = text[start..i]
                .parse()
                .map_err(|_| format!("bad number {}", &text[start..i]))?;
            out.push(Token::Num(n));
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.')
            {
                i += 1;
            }
            let name = &text[start..i];
            out.push(Token::Name(name.to_owned()));
            // nbt(Inventory, raw:iregex:.*Slot:-106b.*shield.*): the arguments
            // are a path and a pattern, not an expression; keep them as text.
            let rest = text[i..].trim_start();
            if name == "nbt" && rest.starts_with('(') {
                let open = text.len() - rest.len();
                let mut depth = 0;
                let close = text[open..]
                    .char_indices()
                    .find(|&(_, c)| {
                        depth += match c {
                            '(' => 1,
                            ')' => -1,
                            _ => 0,
                        };
                        depth == 0
                    })
                    .ok_or("missing ')'")?
                    .0;
                out.extend([Token::Open, Token::Str, Token::Close]);
                i = open + close + 1;
            }
        } else if c == '"' || c == '\'' {
            let end = text[i + 1..].find(c).ok_or("unterminated string")?;
            i += end + 2;
            out.push(Token::Str);
        } else {
            let two = text.get(i..i + 2).unwrap_or("");
            let op: &'static str = match two {
                "&&" => "&&",
                "||" => "||",
                "==" => "==",
                "!=" => "!=",
                "<=" => "<=",
                ">=" => ">=",
                _ => match c {
                    '+' => "+",
                    '-' => "-",
                    '*' => "*",
                    '/' => "/",
                    '%' => "%",
                    '<' => "<",
                    '>' => ">",
                    '!' => "!",
                    '(' => {
                        out.push(Token::Open);
                        i += 1;
                        continue;
                    }
                    ')' => {
                        out.push(Token::Close);
                        i += 1;
                        continue;
                    }
                    ',' => {
                        out.push(Token::Comma);
                        i += 1;
                        continue;
                    }
                    other => return Err(format!("unexpected '{other}'")),
                },
            };
            i += op.len();
            out.push(Token::Op(op));
        }
    }
    Ok(out)
}

fn binary(op: &str) -> Option<Op> {
    Some(match op {
        "+" => Op::Add,
        "-" => Op::Sub,
        "*" => Op::Mul,
        "/" => Op::Div,
        "%" => Op::Rem,
        "<" => Op::Lt,
        "<=" => Op::Le,
        ">" => Op::Gt,
        ">=" => Op::Ge,
        "==" => Op::Eq,
        "!=" => Op::Ne,
        "&&" => Op::And,
        "||" => Op::Or,
        _ => return None,
    })
}

struct Parser<'a> {
    tokens: Vec<Token>,
    at: usize,
    resolve: &'a mut dyn FnMut(&str) -> Option<u16>,
}
impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }
    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.at).cloned();
        self.at += 1;
        t
    }
    fn expression(&mut self, min: u8) -> Result<Node, String> {
        let mut left = self.unary()?;
        while let Some(Token::Op(op)) = self.peek() {
            let Some(op) = binary(op) else { break };
            if op.precedence() < min {
                break;
            }
            self.at += 1;
            let right = self.expression(op.precedence() + 1)?;
            left = fold(Node::Bin(op, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }
    fn unary(&mut self) -> Result<Node, String> {
        match self.next() {
            Some(Token::Op("-")) => Ok(fold(Node::Neg(Box::new(self.unary()?)))),
            Some(Token::Op("+")) => self.unary(),
            Some(Token::Op("!")) => Ok(fold(Node::Not(Box::new(self.unary()?)))),
            Some(Token::Num(n)) => Ok(Node::Const(n)),
            Some(Token::Open) => {
                let inner = self.expression(0)?;
                match self.next() {
                    Some(Token::Close) => Ok(inner),
                    _ => Err("missing ')'".into()),
                }
            }
            Some(Token::Name(name)) if self.peek() == Some(&Token::Open) => {
                self.at += 1;
                self.call(&name)
            }
            Some(Token::Name(name)) => Ok(match name.as_str() {
                "pi" => Node::Const(std::f32::consts::PI),
                "e" => Node::Const(std::f32::consts::E),
                "true" => Node::Const(1.0),
                "false" => Node::Const(0.0),
                _ => Node::Load((self.resolve)(&name).ok_or_else(|| format!("unknown '{name}'"))?),
            }),
            other => Err(format!("unexpected {other:?}")),
        }
    }
    fn call(&mut self, name: &str) -> Result<Node, String> {
        // nbt(...) and print(...) take text and names that are not values:
        // nbt reads nothing here, print gives its last argument.
        if matches!(name, "nbt" | "print" | "printb") {
            let mut depth = 1;
            let start = self.at;
            while depth > 0 {
                match self.next() {
                    Some(Token::Open) => depth += 1,
                    Some(Token::Close) => depth -= 1,
                    None => return Err("missing ')'".into()),
                    _ => {}
                }
            }
            if name == "nbt" {
                return Ok(Node::Const(0.0));
            }
            // print(id, every, value): parse the value on its own.
            let inner = self.tokens[start..self.at - 1].to_vec();
            let last = inner
                .iter()
                .rposition(|t| *t == Token::Comma)
                .map_or(0, |c| c + 1);
            let mut sub = Parser {
                tokens: inner[last..].to_vec(),
                at: 0,
                resolve: &mut *self.resolve,
            };
            return sub.expression(0);
        }
        let func = Func::named(name).ok_or_else(|| format!("unknown function '{name}'"))?;
        let mut args = Vec::new();
        if self.peek() == Some(&Token::Close) {
            self.at += 1;
        } else {
            loop {
                args.push(self.expression(0)?);
                match self.next() {
                    Some(Token::Comma) => {}
                    Some(Token::Close) => break,
                    _ => return Err(format!("bad arguments to {name}")),
                }
            }
        }
        Ok(fold(Node::Call(func, args)))
    }
}

/// Folds a node whose operands are all constants (`pi/2`, `torad(30)`).
fn fold(node: Node) -> Node {
    let constant = match &node {
        Node::Neg(a) | Node::Not(a) => matches!(**a, Node::Const(_)),
        Node::Bin(_, a, b) => matches!(**a, Node::Const(_)) && matches!(**b, Node::Const(_)),
        Node::Call(Func::Random, args) => {
            !args.is_empty() && args.iter().all(|a| matches!(a, Node::Const(_)))
        }
        Node::Call(_, args) => args.iter().all(|a| matches!(a, Node::Const(_))),
        _ => false,
    };
    if constant {
        Node::Const(node.eval(&[]))
    } else {
        node
    }
}

/// Compiles `text`, asking `resolve` for the register of every name.
pub fn compile(text: &str, resolve: &mut dyn FnMut(&str) -> Option<u16>) -> Result<Node, String> {
    let mut p = Parser {
        tokens: tokens(text)?,
        at: 0,
        resolve,
    };
    let node = p.expression(0)?;
    if p.at != p.tokens.len() {
        return Err(format!("trailing {:?}", p.tokens[p.at]));
    }
    Ok(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(text: &str, vars: &[(&str, f32)]) -> f32 {
        let names: Vec<&str> = vars.iter().map(|v| v.0).collect();
        let regs: Vec<f32> = vars.iter().map(|v| v.1).collect();
        let node = compile(text, &mut |n| {
            names.iter().position(|x| *x == n).map(|i| i as u16)
        })
        .unwrap_or_else(|e| panic!("{text}: {e}"));
        node.eval(&regs)
    }

    #[test]
    fn operators_bind_as_in_c() {
        assert_eq!(eval("1 + 2 * 3", &[]), 7.0);
        assert_eq!(eval("-2 * -3 - 1", &[]), 5.0);
        assert_eq!(eval("1 < 2 && 3 > 4 || 2 == 2", &[]), 1.0);
        // EMF: && and || share a level, left to right.
        assert_eq!(eval("1 == 1 || 0 == 0 && 1 == 2", &[]), 0.0);
        assert_eq!(eval("!(1 != 1) && !false", &[]), 1.0);
        assert_eq!(eval("10 % 4 / 2", &[]), 1.0);
        assert_eq!(eval("1 - 2 - 3", &[]), -4.0);
    }

    #[test]
    fn names_with_dots_and_digits_load_their_register() {
        let v = [
            ("var.ls_offset", 2.0),
            ("varb.21_2_plus", 1.0),
            ("head.rx", 0.5),
        ];
        assert_eq!(eval("var.ls_offset * 3 + head.rx", &v), 6.5);
        assert_eq!(eval("if(varb.21_2_plus && head.rx > 0, 1, 0)", &v), 1.0);
    }

    #[test]
    fn functions_match_the_mods() {
        assert_eq!(eval("if(0, 1, 0, 2, 3)", &[]), 3.0);
        assert_eq!(eval("if(0, 1, 1, 2, 3)", &[]), 2.0);
        assert_eq!(eval("clamp(5, 0, 1)", &[]), 1.0);
        assert_eq!(eval("min(3, 1, 2) + max(1, 5)", &[]), 6.0);
        assert_eq!(eval("between(2, 1, 3) + between(4, 1, 3)", &[]), 1.0);
        assert_eq!(eval("fmod(7.9, 3) + fmod(-1, 3)", &[]), 3.0);
        assert_eq!(eval("equals(1, 1.05, 0.1) + equals(1, 2, 0.1)", &[]), 1.0);
        assert_eq!(eval("in(3, 1, 2, 3)", &[]), 1.0);
        assert!((eval("torad(180)", &[]) - std::f32::consts::PI).abs() < 1e-6);
        assert!((eval("wrapdeg(370)", &[]) - 10.0).abs() < 1e-4);
        assert_eq!(eval("print(\"x\", 20, 4 + 1)", &[]), 5.0);
        assert_eq!(eval("nbt(Inventory, \"raw:x\") + 1", &[]), 1.0);
        let r = eval("random(7)", &[]);
        assert!((0.0..1.0).contains(&r) && r == eval("random(7)", &[]));
        assert_ne!(eval("random(7)", &[]), eval("random(8)", &[]));
    }

    #[test]
    fn constants_fold_and_unknown_names_are_errors() {
        assert_eq!(
            compile("pi / 2", &mut |_| None).unwrap(),
            Node::Const(std::f32::consts::FRAC_PI_2)
        );
        assert!(compile("nope + 1", &mut |_| None).is_err());
        assert!(compile("sin(1", &mut |_| None).is_err());
        assert!(compile("frobnicate(1)", &mut |_| None).is_err());
    }
}

#[cfg(test)]
mod pack_tests {
    /// Every expression of the packs under `ANOMA_CEM_DIR` parses.
    #[test]
    #[ignore = "needs ANOMA_CEM_DIR"]
    fn every_pack_expression_compiles() {
        let Some(dir) = std::env::var_os("ANOMA_CEM_DIR") else {
            return;
        };
        let (mut ok, mut failed) = (0, Vec::new());
        let mut stack = vec![std::path::PathBuf::from(dir)];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                if !matches!(p.extension().and_then(|x| x.to_str()), Some("jem" | "jpm")) {
                    continue;
                }
                let v: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
                let mut todo = vec![&v];
                while let Some(v) = todo.pop() {
                    match v {
                        serde_json::Value::Object(o) => {
                            for (k, x) in o {
                                if k == "animations" {
                                    for a in x.as_array().into_iter().flatten() {
                                        for (t, e) in a.as_object().into_iter().flatten() {
                                            let text = e
                                                .as_str()
                                                .map(str::to_owned)
                                                .unwrap_or_else(|| e.to_string());
                                            match super::compile(&text, &mut |_| Some(0)) {
                                                Ok(_) => ok += 1,
                                                Err(err) => failed.push(format!(
                                                    "{}: {t}: {err}: {text}",
                                                    p.display()
                                                )),
                                            }
                                        }
                                    }
                                } else {
                                    todo.push(x);
                                }
                            }
                        }
                        serde_json::Value::Array(a) => todo.extend(a),
                        _ => {}
                    }
                }
            }
        }
        for f in failed.iter().take(30) {
            eprintln!("{f}");
        }
        eprintln!("{ok} compiled, {} failed", failed.len());
        assert!(failed.is_empty());
    }
}
