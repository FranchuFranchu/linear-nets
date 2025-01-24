#![feature(let_chains)]

use std::io::Read;

use crate::util::join_with;
use std::collections::BTreeMap;

pub mod icombs;
pub mod net;
pub mod syntax;
pub mod types;
pub mod util;

#[cfg(test)]
pub mod test;

//
// - Input string
// |
// | syntax::parser
// |
// V Syntax trees
// |
// | syntax::desugarer
// |
// V Desugared syntax (list of simple net operations)
// |
// | syntax::compiler (using net module)
// |
// v Net
// |
// | net::rules
// |
// v Normalized net
// |
// | types::infer
// |
// v Type of free ports
// |
// | types::show
// |
// - Output string

pub fn main() {
    use syntax::Parser;
    let mut s = String::new();
    std::io::stdin().lock().read_to_string(&mut s).unwrap();
    let mut parser = Parser::new(&s);
    let book = parser.parse_book();
    let mut compiler = crate::syntax::compiler::Compiler::default();
    let book = match book {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Syntax error: {}", e);
            return;
        }
    };
    compiler.compile_book(book);

    let mut net = compiler.main_net();

    let mut scope = std::collections::BTreeMap::new();
    let show_agent = |x| format!("{:?}", x);
    print!("{}", net.show_net(&show_agent, &mut scope, 0));

    net.normal(crate::net::rules::apply_rule);
    println!("----- reduce");

    let mut scope = std::collections::BTreeMap::new();
    let show_agent = |x| format!("{:?}", x);
    print!("{}", net.show_net(&show_agent, &mut scope, 0));
    println!("----- infer");

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

    let net_icombs = icombs::Translator::translate_net(net);
    println!("{:?}", net_icombs);
}
