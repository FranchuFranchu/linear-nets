pub mod net;
pub mod syntax;


pub fn main() {
  use net::{Net, Symbol, Arg, GraftArg};
  let times = Symbol::new(0, "times".to_string(), vec![Arg::Partition(1),Arg::Partition(1)]);
  let par = Symbol::new(0, "par".to_string(), vec![Arg::Partition(2)]);
  let a = Net::wire();
  let a = Net::graft(par.clone(), vec![GraftArg::Partition(a, vec![0, 1])]);
  let b = Net::wire();
  let b = Net::graft(par, vec![GraftArg::Partition(b, vec![0, 1])]);
  let c = Net::graft(times, vec![GraftArg::Partition(a, vec![0]), GraftArg::Partition(b, vec![0])]);
  println!("{:#?}", c);
}
