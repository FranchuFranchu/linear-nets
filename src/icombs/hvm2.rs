pub use hvm::ast::{Net as HVMNet, Tree as HVMTree};
use std::collections::BTreeMap;

use super::net::{Net, Tree};

#[derive(Default)]
pub struct EmitHVM2 {
    map: BTreeMap<usize, String>,
    next_free_var: usize,
}

impl EmitHVM2 {
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
    pub fn emit_net(&mut self, mut net: Net) -> HVMNet {
        HVMNet {
            root: self.emit_tree(net.ports.pop_front().unwrap()),
            rbag: net
                .redexes
                .into_iter()
                .map(|(a, b)| (true, self.emit_tree(a), self.emit_tree(b)))
                .collect(),
        }
    }
    fn emit_tree(&mut self, t: Tree) -> HVMTree {
        match t {
            Tree::Var(a) => HVMTree::Var {
                nam: self.get_var(a),
            },
            Tree::Con(a, b) => HVMTree::Con {
                fst: Box::new(self.emit_tree(*a)),
                snd: Box::new(self.emit_tree(*b)),
            },
            Tree::Dup(a, b) => HVMTree::Dup {
                fst: Box::new(self.emit_tree(*a)),
                snd: Box::new(self.emit_tree(*b)),
            },
            Tree::Era => HVMTree::Era,
        }
    }
}
