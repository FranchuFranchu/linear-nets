use super::net::{Net, Tree};

/// This carries out Lafont coding on net

// Encodes a specific tree.
struct Encoder<'a> {
    net: &'a mut Net,
    dups: Vec<(Tree, Tree, Tree)>,
}

impl<'a> Encoder<'a> {
    fn encode_subtree(&mut self, tree: Tree) -> Tree {
        match tree {
            Tree::Con(a, b) => Tree::c(self.encode_subtree(*a), self.encode_subtree(*b)),
            Tree::Dup(a, b) => {
                let a = self.encode_subtree(*a);
                let b = self.encode_subtree(*b);
                let (c, d) = self.net.create_wire();
                self.dups.push((c, a, b));
                d
            }
            Tree::Era => Tree::Era,
            Tree::Var(a) => {
                if let Some(Some(a)) = self.net.vars.remove(&a) {
                    self.encode_subtree(a)
                } else {
                    self.net.vars.insert(a.clone(), None);
                    Tree::Var(a)
                }
            }
        }
    }
    fn merge_ctrs(&mut self, mut ports: Vec<(Tree, Tree, Tree)>) -> (Tree, Tree, Tree) {
        if ports.len() == 0 {
            return (Tree::e(), Tree::e(), Tree::e());
        } else if ports.len() == 1 {
            ports.pop().unwrap()
        } else {
            let rest = ports.split_off(ports.len() / 2 + 1);
            let (l0, l1, l2) = self.merge_ctrs(ports);
            let (r0, r1, r2) = self.merge_ctrs(rest);
            (Tree::c(l0, r0), Tree::c(l1, r1), Tree::c(l2, r2))
        }
    }
    fn encode_tree(&mut self, tree: Tree) -> Tree {
        let tree = self.encode_subtree(tree);
        let dups = core::mem::take(&mut self.dups);
        let (inputs, l, r) = self.merge_ctrs(dups);
        Tree::c(Tree::c(inputs, tree), Tree::c(l, r))
    }
}

pub fn encode_tree(net: &mut Net, tree: Tree) -> Tree {
    Encoder { net, dups: vec![] }.encode_tree(tree)
}
