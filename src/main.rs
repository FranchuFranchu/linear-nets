#![feature(let_chains)]

use std::io::Read;

use crate::util::join_with;
use std::collections::BTreeMap;

pub mod net;
pub mod syntax;
pub mod types;
pub mod util;

pub fn main() {
    use syntax::Parser;
    let mut s = String::new();
    std::io::stdin().lock().read_to_string(&mut s).unwrap();
    let mut parser = Parser::new(&s);
    let net = parser.parse_net();
    let mut compiler = crate::syntax::compiler::Compiler::default();
    let net = compiler.compile_net(net.unwrap());

    let mut scope = std::collections::BTreeMap::new();
    let show_agent = |x| format!("{:?}", x);
    println!("{}", net.show_net(&show_agent, &mut scope, 0));
    let trees = net.substitute_iter(net.ports.iter());
    let types = types::infer(trees);
    let mut ctx = BTreeMap::new();
    println!(
        "|- {}",
        join_with(
            types.into_iter().map(|x| x.show(&mut ctx)),
            ", ".to_string()
        )
    );
}
