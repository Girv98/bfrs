#[derive(Debug, Clone, Copy)]
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


#[derive(Clone, Copy)]
struct Op {
    kind: OpKind,
    value: usize
}

impl Op {
    pub fn from_char(ch: char, value: usize) -> Self {
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
    
    pub fn interpret(&self, cell_size: CellSize) -> Result<()> {
        let mut head = 0;
        let mut ip = 0;

        let mut mem:Vec<usize> = Vec::new();
        mem.push(0);

        while ip < self.0.len() {
            mem[head] = match cell_size {
                CellSize::U8  => mem[head] as u8  as usize,
                CellSize::U16 => mem[head] as u16 as usize,
                CellSize::U32 => mem[head] as u32 as usize,
            };
            let op = self.0[ip];
            match op.kind {
                OpKind::Add => {
                    // NOTE: 
                    mem[head] = mem[head].wrapping_add(op.value);
                    ip += 1;
                },
                OpKind::Sub => {
                    mem[head] = mem[head].wrapping_sub(op.value);
                    ip += 1;
                },
                OpKind::Left => {
                    if head < op.value {
                        return Err(anyhow::format_err!(
                            "Memory Underflow"
                        ))
                    }
                    head -= op.value;
                    ip += 1;
                },
                OpKind::Right => {
                    head += op.value;
                    if head >= mem.len() {
                        mem.resize(head + 1, 0);
                    }
                    ip += 1;
                },
                OpKind::Input => todo!(),
                OpKind::Output => {
                    for _ in 0..op.value {
                        print!("{}", mem[head] as u8 as char);
                    }
                    ip += 1;
                },
                OpKind::JmpZero => {
                    if mem[head] == 0 {
                        ip = op.value; 
                    } else {
                        ip += 1;
                    }
                },
                OpKind::JmpNonZero => {
                    if mem[head] != 0 {
                        ip = op.value;
                    } else {
                        ip += 1;
                    }
                },
            }

        }
        Ok(())
    }
}

enum CellSize {
    U8,
    U16,
    U32
}

fn usage() {
    todo!()
}

use anyhow::{Ok, Result};
use std::{env, fmt::Debug, fs::File, io::Read};

fn main() -> Result<()> {
    let mut file = String::new();
    let mut cell_size = CellSize::U8;
    let mut cs_flag = false;
    for arg in env::args().skip(1) {
        match &arg[..] {
            "-h" | "--help" => { 
                usage(); 
                return Ok(()) 
            },
            "-c" => {
                cs_flag = true;
            },
            "8" | "u8" | "U8" if cs_flag => {
                cell_size = CellSize::U8;
                cs_flag = false;
            },
            "16" | "u16" | "U16" if cs_flag => {
                cell_size = CellSize::U16;
                cs_flag = false;
            },
            "32" | "u32" | "U32" if cs_flag => {
                cell_size = CellSize::U32;
                cs_flag = false;
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
            _ if !file.is_empty() => {
                    usage();
                    return Err(anyhow::format_err!(
                        "cannot provide more than one file."
                    ))
            },
            _ => file = arg
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

    // println!("{:?}", ops);
    ops.interpret(cell_size)
}
