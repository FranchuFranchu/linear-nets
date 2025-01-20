#![feature(let_chains)]

use crate::util::join_with;
use std::collections::BTreeMap;

pub mod net;
pub mod syntax;
pub mod types;
pub mod util;

pub fn main() {
    use net::{GraftArg, Net, SymbolId};
    let a = Net::wire();
    let a = Net::graft(SymbolId::Par, vec![GraftArg::Partition(a, vec![0, 1])]);
    let b = Net::wire();
    let b = Net::graft(SymbolId::Par, vec![GraftArg::Partition(b, vec![0, 1])]);
    let c = Net::graft(
        SymbolId::Times,
        vec![
            GraftArg::Partition(a, vec![0]),
            GraftArg::Partition(b, vec![0]),
        ],
    );

    let w = Net::wire();
    let d = Net::graft(
        SymbolId::False,
        vec![GraftArg::Partition(w, vec![0]), GraftArg::Box(c, vec![0])],
    );
    let mut scope = std::collections::BTreeMap::new();
    let show_agent = |s| format!("{:?}", s);
    println!("{}", d.show_net(&show_agent, &mut scope, 0));
    let trees = d.substitute_iter(d.ports.iter());
    let types = types::infer(trees);
    let mut ctx = BTreeMap::new();
    println!("|- {}", join_with(types.into_iter().map(|x| x.show(&mut ctx)), ", ".to_string()));
}


