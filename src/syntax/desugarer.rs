// Turns net into its desugared form
use std::collections::BTreeSet;

use std::collections::BTreeMap;

use crate::syntax::Argument;

use crate::syntax::Tree;

pub struct Desugarer {
    pub output: Vec<(Tree, Tree)>,
    new_var: usize,
    // maps old vars to RHS of new wired vars
    validly_declared_vars: BTreeSet<usize>,
    new_wired_vars: BTreeMap<usize, usize>,
}

impl Desugarer {
    pub fn new(new_var: usize) -> Self {
        Self {
            new_var,
            validly_declared_vars: BTreeSet::new(),
            new_wired_vars: BTreeMap::new(),
            output: vec![],
        }
    }
    fn make_new_var(&mut self) -> usize {
        self.new_var += 1;
        self.new_var - 1
    }
    pub fn desugar_contents(&mut self, t: Vec<Argument>) -> Vec<Argument> {
        t.into_iter()
            .map(|x| match x {
                Argument::Partition(u) => {
                    Argument::Partition(u.into_iter().map(|v| self.desugar(v)).collect())
                }
                Argument::Box(u) => Argument::Box(u.into_iter().map(|v| self.desugar(v)).collect()),
            })
            .collect()
    }
    fn desugar(&mut self, t: Tree) -> Tree {
        match t {
            Tree::Var(id) => {
                if self.validly_declared_vars.contains(&id) {
                    self.validly_declared_vars.remove(&id);
                    Tree::Var(id)
                } else {
                    // See if the variable has been auto-declared
                    if let Some(id) = self.new_wired_vars.remove(&id) {
                        Tree::Var(id)
                    } else {
                        // Auto-declare the variable with a wire link.
                        let new_id = self.make_new_var();
                        self.output.push((Tree::Var(id), Tree::Var(new_id)));
                        self.validly_declared_vars.insert(new_id);
                        self.validly_declared_vars.insert(id);
                        self.new_wired_vars.insert(id, new_id);
                        Tree::Var(id)
                    }
                }
            }
            Tree::Agent(id, args) => {
                let new_var = self.make_new_var();
                let o = (
                    Tree::Var(new_var),
                    Tree::Agent(id, self.desugar_contents(args)),
                );
                self.output.push(o);
                self.validly_declared_vars.insert(new_var);
                Tree::Var(new_var)
            }
        }
    }
    pub fn desugar_pair(&mut self, (left, right): (Tree, Tree)) {
        match (left, right) {
            (left @ Tree::Var(idl), right @ Tree::Var(idr)) => {
                self.validly_declared_vars.insert(idl);
                self.validly_declared_vars.insert(idr);
                self.output.push((left, right))
            }
            (Tree::Var(_id), Tree::Agent(_aid, _args)) => {
                panic!("Invalid syntax: var = agent")
            }
            (Tree::Agent(aid, args), Tree::Var(vid)) => {
                // Graft, but needs desugaring contents of LHS
                self.validly_declared_vars.insert(vid);
                let o = (
                    Tree::Agent(aid, self.desugar_contents(args)),
                    Tree::Var(vid),
                );
                self.output.push(o);
            }
            (left @ Tree::Agent(..), right @ Tree::Agent(..)) => {
                // Cut, but needs desugaring
                let o = (self.desugar(left), self.desugar(right));
                self.output.push(o);
            }
        }
    }
}
