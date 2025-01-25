#[derive(Debug)]
pub enum Argument {
    Partition(Vec<Tree>),
    Box(Vec<Tree>),
}

#[derive(Debug)]
pub enum Tree {
    Agent(String, Vec<Argument>),
    Var(usize),
}

impl Tree {
    pub fn is_var(&self) -> bool {
        matches!(self, Tree::Var(..))
    }
}

pub type Book = Vec<AstNet>;

#[derive(Debug)]
pub enum Instruction {
    Multicut(String, Vec<Tree>),
    Monocut(Tree, Tree),
}

#[derive(Debug)]
pub struct AstNet {
    name: String,
    outputs: Vec<Argument>,
    instructions: Vec<Instruction>,
}

impl std::fmt::Display for AstNet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.name)?;
        for i in &self.outputs {
            write!(f, "{}", i)?;
        }
        write!(f, " {{\n")?;
        for i in &self.instructions {
            write!(f, "    {}\n", i)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Instruction::Multicut(name, trees) => {
                write!(f, "{}(", name)?;
                let mut sp = false;
                for i in trees {
                    if sp {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", i)?;
                    sp = true
                }
                write!(f, ")")?;
            }
            Instruction::Monocut(l, r) => {
                write!(f, "{} = {}", l, r)?;
            }
        };
        Ok(())
    }
}
impl std::fmt::Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Argument::Partition(trees) => {
                write!(f, "(")?;
                let mut sp = false;
                for i in trees {
                    if sp {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", i)?;
                    sp = true
                }
                write!(f, ")")?;
            }
            Argument::Box(trees) => {
                write!(f, "[")?;
                let mut sp = false;
                for i in trees {
                    if sp {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", i)?;
                    sp = true
                }
                write!(f, "]")?;
            }
        };
        Ok(())
    }
}
impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Tree::Agent(name, args) => {
                write!(f, "{}", name)?;
                for i in args {
                    write!(f, "{}", i)?;
                }
            }
            Tree::Var(id) => {
                write!(f, "x{}", id)?;
            }
        };
        Ok(())
    }
}

pub mod compiler;
pub mod desugarer;
pub mod parser;

pub use parser::Parser;
