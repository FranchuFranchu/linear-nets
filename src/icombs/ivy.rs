pub use ivy::ast::{Net as IvyNet, Nets as IvyNets, Tree as IvyTree};
use std::collections::BTreeMap;

use super::net::{Net, Tree};

#[derive(Default)]
pub struct EmitIvy {
    map: BTreeMap<usize, String>,
    next_free_var: usize,
}

impl EmitIvy {
    fn new_var(&mut self) -> String {
        let var = crate::util::number_to_string(self.next_free_var);
        self.next_free_var += 1;
        var
    }
    fn get_var(&mut self, id: usize) -> String {
        match self.map.remove(&id) {
            Some(a) => a,
            None => {
                let v = self.new_var();
                self.map.insert(id, v.clone());
                v
            }
        }
    }
    pub fn emit_net(&mut self, mut net: Net) -> IvyNet {
        IvyNet {
            root: self.emit_tree(net.ports.pop_front().unwrap()),
            pairs: net
                .redexes
                .into_iter()
                .map(|(a, b)| (self.emit_tree(a), self.emit_tree(b)))
                .collect(),
        }
    }
    fn emit_tree(&mut self, t: Tree) -> IvyTree {
        match t {
            Tree::Var(a) => IvyTree::Var(self.get_var(a)),
            Tree::Con(a, b) => IvyTree::n_ary("con", [self.emit_tree(*a), self.emit_tree(*b)]),
            Tree::Dup(a, b) => IvyTree::n_ary("dup", [self.emit_tree(*a), self.emit_tree(*b)]),
            Tree::Era => IvyTree::n_ary("era", []),
        }
    }
}
