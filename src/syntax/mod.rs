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

#[derive(Debug)]
#[allow(unused)] // we'll use them later ;)
pub struct AstNet {
    name: String,
    outputs: Vec<Argument>,
    instructions: Vec<(Tree, Tree)>,
}

pub mod compiler;
pub mod desugarer;
pub mod parser;

pub use parser::Parser;
