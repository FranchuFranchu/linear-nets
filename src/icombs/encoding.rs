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
            Tree::Con(a, b) => Tree::Con(
                Box::new(self.encode_subtree(*a)),
                Box::new(self.encode_subtree(*b)),
            ),
            Tree::Dup(a, b) => {
                let a = self.encode_subtree(*a);
                let b = self.encode_subtree(*b);
                let (c, d) = self.net.create_wire();
                self.dups.push((c, a, b));
                d
            }
            Tree::Era => Tree::Era,
            Tree::Var(a) => Tree::Var(a),
        }
    }
    fn merge_ctrs(&mut self, mut ports: Vec<(Tree, Tree, Tree)>) -> (Tree, Tree, Tree) {
        if ports.len() == 1 {
            ports.pop().unwrap()
        } else {
            let rest = ports.split_off(ports.len() / 2);
            let (l0, l1, l2) = self.merge_ctrs(ports);
            let (r0, r1, r2) = self.merge_ctrs(rest);
            (
                Tree::Con(Box::new(l0), Box::new(r0)),
                Tree::Con(Box::new(l1), Box::new(r1)),
                Tree::Con(Box::new(l2), Box::new(r2)),
            )
        }
    }
    fn encode_tree(&mut self, tree: Tree) -> Tree {
        let tree = self.encode_subtree(tree);
        let dups = core::mem::take(&mut self.dups);
        let (inputs, l, r) = self.merge_ctrs(dups);
        Tree::Con(
            Box::new(Tree::Con(Box::new(inputs), Box::new(tree))),
            Box::new(Tree::Con(Box::new(l), Box::new(r))),
        )
    }
}
