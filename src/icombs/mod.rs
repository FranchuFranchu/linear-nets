#[cfg(feature = "hvm")]
pub mod hvm2;
#[cfg(feature = "ivy")]
pub mod ivy;
pub mod net;
pub mod encoding;

use std::collections::BTreeMap;

use crate::net::{Cell, Net, Tree};
use net::Net as ICombNet;
use net::Tree as ICombTree;

#[derive(Default)]
pub struct Translator {
    net: ICombNet,
    /// Maps old variables to new variables.
    var_map: BTreeMap<usize, usize>,
}

impl Translator {
    pub fn translate_net(from: Net) -> ICombNet {
        let mut translator = Self::default();

        for (a, b) in from.redexes {
            let a = translator.translate_tree(a);
            let b = translator.translate_tree(b);
            translator.net.redexes.push_back((a, b));
        }
        for a in from.ports {
            let a = translator.translate_tree(a);
            translator.net.ports.push_back(a);
        }
        for (k, v) in from.vars {
            if let Some(a) = v {
                let a = translator.translate_tree(a);
                translator
                    .net
                    .vars
                    .insert(*translator.var_map.get(&k).unwrap(), Some(a));
            }
        }
        translator.net
    }
    fn translate_net_and_merge(&mut self, from: Net) -> Vec<ICombTree> {
        let mut net = Self::translate_net(from);
        let mut map = BTreeMap::new();

        net.map_vars(&mut |x| {
            if let Some(v) = map.get(&x) {
                *v
            } else {
                let v = self.net.allocate_var_id();
                self.net.vars.insert(v, None);
                map.insert(x, v);
                v
            }
        });
        for (k, v) in map {
            if let Some(i) = net.vars.remove(&k) {
                self.net.vars.insert(v, i);
            }
        }
        self.net.redexes.append(&mut net.redexes);
        core::mem::take(&mut net.ports).into()
    }
    fn translate_tree(&mut self, from: Tree) -> ICombTree {
        match from {
            Tree::Var(id) => match self.var_map.remove(&id) {
                Some(a) => ICombTree::Var(a),
                None => {
                    let a = self.net.allocate_var_id();
                    self.net.vars.insert(a, None);
                    self.var_map.insert(id, a);
                    ICombTree::Var(a)
                }
            },
            cell => self.translate_cell(Cell::from_tree(cell).unwrap()),
        }
    }
    fn translate_cell(&mut self, cell: Cell) -> ICombTree {
        match cell {
            Cell::Times((a,), (b,)) => ICombTree::Con(
                Box::new(self.translate_tree(a)),
                Box::new(self.translate_tree(b)),
            ),
            Cell::Par((a, b)) => ICombTree::Con(
                Box::new(self.translate_tree(a)),
                Box::new(self.translate_tree(b)),
            ),
            Cell::One() => ICombTree::Era,
            Cell::False((a,), b) => {
                let Ok([b]): Result<[ICombTree; 1], _> = self.translate_net_and_merge(b).try_into()
                else {
                    unreachable!()
                };
                let a = self.translate_tree(a);
                self.net.link(a, b);
                ICombTree::Era
            }
            Cell::Left((out,)) => {
                let out = self.translate_tree(out);
                let (a, b) = self.net.create_wire();
                ICombTree::Con(
                    Box::new(a),
                    Box::new(ICombTree::Con(
                        Box::new(ICombTree::Con(Box::new(b), Box::new(out))),
                        Box::new(ICombTree::Era),
                    )),
                )
            }
            Cell::Right((out,)) => {
                let out = self.translate_tree(out);
                let (a, b) = self.net.create_wire();
                ICombTree::Con(
                    Box::new(a),
                    Box::new(ICombTree::Con(
                        Box::new(ICombTree::Era),
                        Box::new(ICombTree::Con(Box::new(b), Box::new(out))),
                    )),
                )
            }
            Cell::True((out,)) => {
                let out = self.translate_tree(out);
                self.net.link(ICombTree::Era, out);
                ICombTree::Era
            }
            Cell::With((ctx,), left, right) => {
                let Ok([cl, vl]): Result<[ICombTree; 2], _> =
                    self.translate_net_and_merge(left).try_into()
                else {
                    unreachable!()
                };
                let Ok([cr, vr]): Result<[ICombTree; 2], _> =
                    self.translate_net_and_merge(right).try_into()
                else {
                    unreachable!()
                };
                let ctx = self.translate_tree(ctx);
                ICombTree::Con(
                    Box::new(ctx),
                    Box::new(ICombTree::Con(
                        Box::new(ICombTree::Con(Box::new(cl), Box::new(vl))),
                        Box::new(ICombTree::Con(Box::new(cr), Box::new(vr))),
                    )),
                )
            }
            _ => todo!(),
        }
    }
}
