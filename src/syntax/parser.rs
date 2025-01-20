use std::collections::BTreeMap;
use TSPL::Parser as TSPLParser;

use crate::syntax::desugarer::Desugarer;

use crate::syntax::AstNet;

use crate::syntax::Tree;

use std::collections::btree_map::Entry;

use crate::syntax::Argument;

#[derive(Debug)]
pub struct Parser<'i> {
    input: &'i str,
    index: usize,
    vars: BTreeMap<String, usize>,
    new_var: usize,
}
impl<'i> TSPLParser<'i> for Parser<'i> {
    fn input(&mut self) -> &'i str {
        &self.input
    }
    fn index(&mut self) -> &mut usize {
        &mut self.index
    }
}
impl<'i> Parser<'i> {
    pub fn new(input: &'i str) -> Self {
        Self {
            input,
            index: 0,
            vars: BTreeMap::new(),
            new_var: 0,
        }
    }
    pub fn parse_net(&mut self) -> Result<AstNet, String> {
        let Tree::Agent(name, args) = self.parse_tree()? else {
            return Err("Not a good net name!".to_string());
        };
        self.skip_trivia();
        let mut instr = vec![];
        self.consume("{")?;
        while !matches!(self.peek_one(), Some('}')) {
            let a = self.parse_tree()?;
            self.skip_trivia();
            self.consume("=")?;
            let b = self.parse_tree()?;
            self.skip_trivia();
            instr.push((a, b));
        }
        self.consume("}")?;
        let mut desugar = Desugarer::new(self.new_var);
        for i in instr {
            desugar.desugar_pair(i);
        }
        let args = desugar.desugar_contents(args);
        Ok(AstNet {
            name,
            outputs: args,
            instructions: desugar.output,
        })
    }
    pub fn parse_tree(&mut self) -> Result<Tree, String> {
        self.skip_trivia();
        if self.peek_one().is_some_and(|x| x.is_ascii_lowercase()) {
            Ok(Tree::Var(self.parse_var()?))
        } else {
            let name = self.parse_name()?;
            self.skip_trivia();
            let mut v = vec![];
            while matches!(self.peek_one(), Some('(') | Some('[')) {
                v.push(self.parse_argument()?);
            }
            Ok(Tree::Agent(name, v))
        }
    }
    pub fn parse_var(&mut self) -> Result<usize, String> {
        self.skip_trivia();
        let name = self.parse_name()?;
        match self.vars.entry(name) {
            Entry::Occupied(e) => Ok(e.remove()),
            Entry::Vacant(e) => {
                let id = self.new_var;
                e.insert(id);
                self.new_var += 1;
                Ok(id)
            }
        }
    }
    pub fn parse_argument(&mut self) -> Result<Argument, String> {
        self.skip_trivia();
        match self.peek_one() {
            Some('(') => {
                self.consume("(")?;
                let mut v = vec![];
                while self.peek_one() != Some(')') {
                    v.push(self.parse_tree()?);
                }
                let _ = self.consume(")");
                Ok(Argument::Partition(v))
            }
            Some('[') => {
                self.consume("[")?;
                let mut v = vec![];
                while self.peek_one() != Some(']') {
                    v.push(self.parse_tree()?);
                }
                let _ = self.consume("]");
                Ok(Argument::Box(v))
            }
            _ => Err("Not an argument!".to_string()),
        }
    }
}
