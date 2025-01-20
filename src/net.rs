// Net implementation.
// Understands simplicity and understands boxing.

use std::collections::BTreeMap;
use std::rc::Rc;

pub type VarId = usize;
pub type AgentId = usize;


pub fn reorder<T>(a: &mut Vec<T>, mut indices: Vec<usize>, reorder_rest: bool) -> bool {
  let mut result = vec![];
  indices.reverse();
  while let Some(idx) = indices.pop() {
    result.push(a.remove(idx));
    indices = indices.into_iter().map(|x| if x > idx { x - 1 } else {x}).collect()
  }
  if a.len() > 0 {
    if reorder_rest {
      result.append(a);
    } else {
      return false;
    }
  }
  core::mem::replace(a, result);
  return true;
}

#[derive(Debug)]
pub enum Arg {
  Partition(usize),
  Box(usize),
}

#[derive(Debug)]
pub struct Symbol {
  id: usize,
  name: String,
  aux: Vec<Arg>,
}

impl Symbol {
  pub fn new(id: usize, name: String, aux: Vec<Arg>) -> Rc<Symbol> {
    Rc::new(Symbol {id, name, aux})
  }
}

#[derive(Debug)]
pub enum GraftArg {
  // net in the partition, and list of free ports
  Partition(Net, Vec<usize>),
  // net to box, and list of free ports. All free ports must be used here.
  Box(Net, Vec<usize>),
}
#[derive(Debug)]
enum PartitionOrBox {
  Partition(Vec<Port>),
  Box(Net),
}
impl PartitionOrBox {
  fn map_vars(&mut self, m: &impl Fn(VarId) -> VarId) {
    use PartitionOrBox::*;
    match self {
      PartitionOrBox::Partition(ports) => { ports.iter_mut().for_each(|x| x.map_vars(m)) }
      _ => {}
    }
  }
}

#[derive(Debug)]
pub enum Port {
  Var(VarId),
  Agent(Rc<Symbol>, Vec<PartitionOrBox>)
}

impl Port {
  fn map_vars(&mut self, m: &impl Fn(VarId) -> VarId) {
    use Port::*;
    match self {
      Var(x) => { *x = m(*x) }
      Agent(_, ports) => { ports.iter_mut().for_each(|x| x.map_vars(m)) }
    }
  }
}

#[derive(Debug)]
pub struct Net {
  ports: Vec<Port>,
  redexes: Vec<(Port, Port)>,
  vars: BTreeMap<usize, Option<Port>>,
}

impl Net {
  fn empty() -> Net {
    Net {
      ports: vec![],
      redexes: vec![],
      vars: BTreeMap::new(),
    }
  }
  fn interact(&mut self, a: Port, b: Port) {
    todo!();
  }
  pub fn map_vars(&mut self, m: &impl Fn(VarId) -> VarId) {
    self.ports.iter_mut().for_each(|x| x.map_vars(m));
    self.redexes.iter_mut().for_each(|(a, b)| { a.map_vars(m); b.map_vars(m) });
    self.vars.values_mut().for_each(|a| { a.as_mut().map(|x| x.map_vars(m)); });
  }
  fn allocate_var_id(&mut self) -> VarId {
    for i in 0.. {
      if self.vars.get(&i).is_none() {
        return i;
      }
    }
    unreachable!();
  }
  fn create_wire(&mut self) -> (Port, Port) {
    let id = self.allocate_var_id();
    self.vars.insert(id, None);
    (Port::Var(id), Port::Var(id))
  }
  pub fn wire() -> Net {
    let mut net = Net::empty();
    let (a, b) = net.create_wire();
    net.ports.append(&mut vec![a, b]);
    net
  }
  pub fn graft(symbol: Rc<Symbol>, args: Vec<GraftArg>) -> Net {
    assert!(symbol.aux.len() == args.len());
    let mut aux = vec![];
    let mut built_net = Net::empty();
    for (q, i) in symbol.aux.iter().zip(args) {
      match (q, i) {
        (Arg::Box(size), GraftArg::Box(mut net, ports)) => {
          assert!(*size == ports.len());
          reorder(&mut net.ports, ports, false);
          aux.push(PartitionOrBox::Box(net));
        }
        (Arg::Partition(size), GraftArg::Partition(mut net, ports)) => {
          assert!(*size == ports.len());
          reorder(&mut net.ports, ports, true);
          let mut ports = vec![];
          for i in 0..*size {
            ports.push(net.ports.remove(0));
          }
          let mut var_map = built_net.shift_map();
          built_net = built_net.mix(net);
          ports.iter_mut().for_each(|x| x.map_vars(&var_map));
          aux.push(PartitionOrBox::Partition(ports));
        }
        _ => { todo!("ERror") },
      }
    }
    built_net.ports.push(Port::Agent(symbol, aux));
    built_net
  }
  fn shift_map(&self) -> impl Fn(VarId) -> VarId {
    let vars = self.vars.len();
    Box::new(move |x| x + vars)
  }
  fn mix(mut self, mut other: Net) -> Net {
    let map = |x| x + self.vars.len();
    other.map_vars(&map);
    self.ports.append(&mut other.ports);
    self.redexes.append(&mut other.redexes);
    self.vars.append(&mut other.vars);
    self
  }
  pub fn cut(this: Net, this_port: usize, mut other: Net, other_port: usize) {
    let this_len = this.ports.len();
    let mut composite = this.mix(other);
    let port_a = composite.ports.remove(this_port);
    let port_b = composite.ports.remove(other_port+this_len-1);
    composite.interact(port_a, port_b);
  }
}
