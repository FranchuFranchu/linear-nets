use crate::net::GraftArg;
use crate::net::Net;
use crate::net::SymbolId;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Default)]
pub struct Compiler {
    pub wire_to_nets: BTreeMap<usize, (usize, usize)>,
    pub nets: BTreeMap<usize, (Net, Vec<usize>)>,
    pub next_net_id: usize,
}

fn agent_name_to_id(s: &str) -> Option<SymbolId> {
    match s {
        "Times" => Some(SymbolId::Times),
        "Par" => Some(SymbolId::Par),
        "False" => Some(SymbolId::False),
        "One" => Some(SymbolId::One),
        _ => None,
    }
}

impl Compiler {
    fn make_new_net_id(&mut self) -> usize {
        self.next_net_id += 1;
        self.next_net_id - 1
    }
    pub fn compile_net(&mut self, net: crate::syntax::AstNet) -> Net {
        for i in net.instructions {
            println!("{:?}", i);
            self.compile_pair(i);
        }
        // If everything was done right, there's exactly one net left.
        assert!(self.nets.len() == 1);
        let net = core::mem::take(&mut self.nets)
            .into_iter()
            .next()
            .unwrap()
            .1
             .0;
        // TODO: This is the place where we'd add support for composing smaller nets
        net
    }
    fn compile_pair(&mut self, (left, right): (super::Tree, super::Tree)) {
        match (left, right) {
            (super::Tree::Var(a), super::Tree::Var(b)) => {
                // Decide whether this is a cut or a wire.
                if let (Some((a_net, a_addr)), Some((b_net, b_addr))) =
                    (self.wire_to_nets.get(&a), self.wire_to_nets.get(&b))
                {
                    let a_net = self.nets.remove(&a_net).unwrap();
                    let b_net = self.nets.remove(&b_net).unwrap();
                    let new_net = Net::cut(a_net.0, *a_addr, b_net.0, *b_addr);
                    let new_net_id = self.make_new_net_id();
                    let mut new_wires = vec![];
                    for i in a_net.1 {
                        if i != a {
                            new_wires.push(i);
                            self.wire_to_nets
                                .insert(i, (new_net_id, new_wires.len() - 1));
                        }
                    }
                    for i in b_net.1 {
                        if i != b {
                            new_wires.push(i);
                            self.wire_to_nets
                                .insert(i, (new_net_id, new_wires.len() - 1));
                        }
                    }
                    self.nets.insert(new_net_id, (new_net, new_wires));
                } else if !self.wire_to_nets.contains_key(&a) && !self.wire_to_nets.contains_key(&b)
                {
                    // Wire
                    let new_net = Net::wire();
                    let new_net_id = self.make_new_net_id();
                    self.nets.insert(new_net_id, (new_net, vec![a, b]));
                    self.wire_to_nets.insert(a, (new_net_id, 0));
                    self.wire_to_nets.insert(b, (new_net_id, 1));
                } else {
                    panic!(":(")
                }
            }
            (super::Tree::Agent(agent_name, args), super::Tree::Var(var_id)) => {
                if let Some(symbol_id) = agent_name_to_id(&agent_name) {
                    let mut included_vars = BTreeSet::new();

                    let mut graft_args = vec![];
                    let mut new_vars = vec![var_id];
                    let mut new_index = 1;
                    let new_net_id = self.make_new_net_id();

                    self.wire_to_nets.insert(var_id, (new_net_id, 0));
                    use super::Argument;
                    for i in args {
                        let is_box = matches!(i, Argument::Box(..));
                        match i {
                            Argument::Partition(x) | Argument::Box(x) => {
                                let mut net_id = None;
                                let mut addresses = vec![];
                                for wire in x {
                                    let crate::syntax::Tree::Var(wire) = wire else {
                                        unreachable!()
                                    };
                                    let (net, addr) = self.wire_to_nets.remove(&wire).unwrap();
                                    if let Some(net_id) = net_id {
                                        assert!(net_id == net);
                                    } else {
                                        net_id = Some(net);
                                    };

                                    included_vars.insert(wire);
                                    addresses.push(addr);
                                }
                                let (net, old_wires) = self.nets.remove(&net_id.unwrap()).unwrap();
                                for wire in old_wires {
                                    if !included_vars.contains(&wire) {
                                        if is_box {
                                            panic!("Missing wire in box!");
                                        } else {
                                            self.wire_to_nets.insert(wire, (new_net_id, new_index));
                                            new_vars.push(wire);
                                            new_index += 1;
                                        }
                                    }
                                }
                                if is_box {
                                    graft_args.push(GraftArg::Box(net, addresses));
                                } else {
                                    graft_args.push(GraftArg::Partition(net, addresses));
                                }
                            }
                        }
                    }

                    self.nets
                        .insert(new_net_id, (Net::graft(symbol_id, graft_args), new_vars));
                } else {
                    todo!();
                }
            }
            _ => unreachable!(),
        }
    }
}
