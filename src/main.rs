


#[derive(Debug)]
enum Op {
    Add(usize),
    Sub(usize),
    Left(usize),
    Right(usize),
    Input(usize),
    Output(usize),
    JmpZero(usize),
    JmpNonZero(usize),
}

impl Op {
    pub fn as_char(&self) -> char {
        match self {
            Self::Add(_)        => '+',
            Self::Sub(_)        => '-',
            Self::Left(_)       => '<',
            Self::Right(_)      => '>',
            Self::Input(_)      => ',',
            Self::Output(_)     => '.',
            Self::JmpZero(_)    => '[',
            Self::JmpNonZero(_) => ']',
        }
    }
    
    pub fn from(ch: char, n: usize) -> Self {
        match ch {
            '+' => {Self::Add(n)},
            '-' => {Self::Sub(n)},
            '<' => {Self::Left(n)},
            '>' => {Self::Right(n)},
            ',' => {Self::Input(n)},
            '.' => {Self::Output(n)},
            '[' => {Self::JmpZero(n)},
            ']' => {Self::JmpNonZero(n)},
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

struct Addrs (Vec<usize>);

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
        while self.has_more_chars() && !Op::is_op(self.source[0]) {
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
            match self.source[0] {
                '+' | '-' | '<' | '>' | ',' | '.'  => {
                    let i = self.source[0];
                    let n = self.chop_while(|x| *x == i);
                    Some(Op::from(i, n))
                },
                '[' => {
                    self.advance();
                    Some(Op::JmpZero(0))
                },
                ']' => {
                    self.advance();
                    Some(Op::JmpNonZero(0))
                },
                _ => {unreachable!()}
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct Ops (Vec<Op>);

impl Ops {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn gen(&mut self, file_path: &str) -> Result<()> {
        
        let Result::Ok(mut f) = File::open(file_path) else {
            return Err(anyhow::format_err!(
                "file `{}` not found", file_path
            ))
        };

        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let contents = &contents.chars().collect::<Vec<_>>();
        let mut stack = Addrs(vec![]);

        let mut lexer = Lexer::new(&contents);

        while let Some(op) = lexer.next() {
            // println!("{op:?}");
            match op {
                Op::JmpZero(_) => {
                    let addr = self.0.len();
                    self.0.push(op);
                    stack.0.push(addr);
                },
                Op::JmpNonZero(_) => {
                    let Some(&addr) = stack.0.last() else {
                        return Err(anyhow::format_err!(
                            "unbalanced loop at {}", lexer.pos
                        ))
                    };
                    stack.0.pop();
                    self.0.push(Op::JmpNonZero(addr + 1));

                    let new_addr = self.0.len();
                    if let Op::JmpZero(ref mut wrapped_value) = self.0[addr] {
                        *wrapped_value = new_addr;
                    }
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
use std::{env, fs::File, io::Read, str::Chars};

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
    ops.gen(&file)?;

    println!("{:?}", ops);

    // println!("Hello, world!");

    Ok(())
}
