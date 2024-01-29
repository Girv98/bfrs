#[derive(Debug)]
enum OpKind {
    Add,
    Sub,
    Left,
    Right,
    Input,
    Output,
    JmpZero,
    JmpNonZero,
}

impl OpKind {
    pub fn as_char(&self) -> char {
        match self {
            Self::Add        => '+',
            Self::Sub        => '-',
            Self::Left       => '<',
            Self::Right      => '>',
            Self::Input      => ',',
            Self::Output     => '.',
            Self::JmpZero    => '[',
            Self::JmpNonZero => ']',
        }
    }
    
    pub fn from(ch: char) -> Self {
        match ch {
            '+' => Self::Add,
            '-' => Self::Sub,
            '<' => Self::Left,
            '>' => Self::Right,
            ',' => Self::Input,
            '.' => Self::Output,
            '[' => Self::JmpZero,
            ']' => Self::JmpNonZero,
            _ => unimplemented!()
        }
    } 

    pub fn is_op(ch: char) -> bool {
        match ch {
            '+' | '-' | '<' | '>' | ',' | '.' | '[' | ']' => true,
            _ => false
        }
    }
}

struct Op {
    kind: OpKind,
    value: usize
}

impl Op {
    pub fn from_char(ch: char, value: usize) -> Self{
        let kind  = OpKind::from(ch);
        Self { kind, value }
    }
}


struct Addrs (Vec<usize>);

impl Addrs {
    pub fn new() -> Self {
        Self(vec![])
    }
}

struct Lexer<'a> {
    source: &'a [char],
    pos: usize
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Self { source: content, pos: 0 }
    }

    fn has_more_chars(&self) -> bool { !self.source.is_empty() }

    fn advance(&mut self) {
        self.source = &self.source[1..];
        self.pos += 1;
    }

    fn trim_whitespace(&mut self) {
        while self.has_more_chars() && !OpKind::is_op(self.source[0]) {
            self.advance();
        }
    }

    fn chop(&mut self, n: usize) {
        self.source = &self.source[n..];
        self.pos += n;
    }

    fn chop_while<P>(&mut self, mut pred: P) -> usize where P: FnMut(&char) -> bool {
        let mut n = 0;
        while n < self.source.len() && pred(&self.source[n]) {
            n += 1;
        }
        self.chop(n);
        n
    }

    pub fn next(&mut self) -> Option<Op> {
        self.trim_whitespace();
        if self.has_more_chars() {
            let (i , n) = match self.source[0] {
                '+' | '-' | '<' | '>' | ',' | '.'  => {
                    let i = self.source[0];
                    (i, self.chop_while(|x| *x == i))
                },
                '[' | ']' => {
                    let i = self.source[0];
                    self.advance();
                    (i, 0)
                },
                _ => unreachable!(),
            };
            Some(Op::from_char(i, n))
        } else {
            None
        }
    }
}

struct Ops (Vec<Op>);

impl Debug for Ops {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.debug_struct("Op").field("kind", &self.kind).field("value", &self.value).finish()

        writeln!(f, "Ops ")?;
        for  Op {kind, value} in &self.0 {
            let k = kind.as_char();
            writeln!(f, "{k} -> {value}")?;
        }
        Result::Ok(())
    }
}

impl Ops {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn gen_from_file(&mut self, file_path: &str) -> Result<()> {
        
        let Result::Ok(mut f) = File::open(file_path) else {
            return Err(anyhow::format_err!(
                "file `{}` not found", file_path
            ))
        };

        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let contents = &contents.chars().collect::<Vec<_>>();
        let mut stack = Addrs::new();

        let mut lexer = Lexer::new(&contents);

        while let Some(op) = lexer.next() {
            // println!("{op:?}");
            match op.kind {
                OpKind::JmpZero => {
                    let addr = self.0.len();
                    self.0.push(op);
                    stack.0.push(addr);
                },
                OpKind::JmpNonZero => {
                    let Some(&addr) = stack.0.last() else {
                        return Err(anyhow::format_err!(
                            "unbalanced loop at {}", lexer.pos
                        ))
                    };
                    stack.0.pop();
                    let op = Op { kind: OpKind::JmpNonZero, value: addr + 1 };
                    self.0.push(op);
                    self.0[addr].value  = self.0.len();
                },
                _ => self.0.push(op),
            }
        }
        
        Ok(())
    }
}


fn usage() {
    todo!()
}

use anyhow::{Ok, Result};
use std::{env, fmt::Debug, fs::File, io::Read};

fn main() -> Result<()> {
    let mut file = String::new();
    for arg in env::args().skip(1) {
        match &arg[..] {
            "-h" | "--help" => { 
                usage(); 
                return Ok(()) 
            },
            _ if arg.starts_with('-') => {
                usage();
                // return Err(anyhow::format_err!(
                //     "no such command: `{}` \n\n\t\
                //     adasdsaff",
                //     arg
                // ))
                return Err(anyhow::format_err!(
                    "unknown command: `{}`", arg
                ))
            },
            _ => {
                if !file.is_empty() {
                    usage();
                    return Err(anyhow::format_err!(
                        "cannot provide more than one file."
                    ))
                }
                file = arg;
            }
        }
    }

    if file.is_empty() {
        usage();
        return Err(anyhow::format_err!(
            "no input provided."
        ))
    }

    let mut ops = Ops::new();
    ops.gen_from_file(&file)?;

    println!("{:?}", ops);

    // println!("Hello, world!");

    Ok(())
}
