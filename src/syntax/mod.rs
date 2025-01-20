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

pub mod compiler;
pub mod desugarer;
pub mod parser;

pub use parser::Parser;
