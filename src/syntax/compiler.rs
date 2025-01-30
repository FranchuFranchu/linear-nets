use super::Tree;
use crate::net::GraftArg;
use crate::net::Net;
use crate::net::SymbolId;
use crate::syntax::Instruction;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Default)]
pub struct Compiler {
    pub wire_to_nets: BTreeMap<usize, (usize, usize)>,
    pub nets: BTreeMap<usize, (Net, Vec<usize>)>,
    pub next_net_id: usize,
    pub global_nets: BTreeMap<String, Net>,
}

fn agent_name_to_id(s: &str) -> Option<SymbolId> {
    match s {
        "Times" => Some(SymbolId::Times),
        "Par" => Some(SymbolId::Par),
        "False" => Some(SymbolId::False),
        "One" => Some(SymbolId::One),

        "Left" => Some(SymbolId::Left),
        "Right" => Some(SymbolId::Right),
        "With" => Some(SymbolId::With),
        "True" => Some(SymbolId::True),

        "Exp0" => Some(SymbolId::Exp0),
        "Exp1" => Some(SymbolId::Exp1),
        "Weak" => Some(SymbolId::Weak),
        "Dere" => Some(SymbolId::Dere),
        "Cntr" => Some(SymbolId::Cntr),

        "All" => Some(SymbolId::All),
        "Any" => Some(SymbolId::Any),
        _ => None,
    }
}

impl Compiler {
    fn make_new_net_id(&mut self) -> usize {
        self.next_net_id += 1;
        self.next_net_id - 1
    }
    pub fn compile_book(&mut self, book: crate::syntax::Book) {
        for net in book {
            self.compile_net(net);
        }
    }
    pub fn compile_net(&mut self, net: crate::syntax::AstNet) {
        self.wire_to_nets = BTreeMap::new();
        self.nets = BTreeMap::new();
        self.next_net_id = 0;
        for i in net.instructions {
            // println!("{:?}", i);
            match i {
                Instruction::Monocut(a, b) => self.compile_monocut(a, b),
                Instruction::Multicut(a, b) => self.compile_multicut(a, b),
            }
            // println!("{:?}\n--", self);
        }
        // If everything was done right, there's exactly one net left.
        if self.nets.len() != 1 {
            panic!(
                "Definition consists of more than one disconnected subnets!: \n{}",
                {
                    use core::fmt::Write;
                    let mut s = String::new();
                    for (net, wires) in self.nets.values() {
                        writeln!(&mut s, "Net with wires: {:?}", wires).unwrap();
                        write!(&mut s, "{}", net.show_net_simple()).unwrap();
                    }
                    s
                }
            )
        }
        let (mut new_net, net_wires) = core::mem::take(&mut self.nets)
            .into_iter()
            .next()
            .unwrap()
            .1;
        // Here, we make the wire order in the resulting net match the wire order in the definition.
        let mut m = BTreeMap::new();
        for wire in net_wires.into_iter() {
            m.insert(wire, new_net.ports.pop_front().unwrap());
        }
        use crate::syntax::Argument;
        let Ok([Argument::Partition(wires)]): Result<[Argument; 1], _> = net.outputs.try_into()
        else {
            unreachable!()
        };
        for wire in wires {
            let Tree::Var(wire) = wire else {
                unreachable!()
            };
            new_net.ports.push_back(m.remove(&wire).unwrap());
        }
        assert!(m.is_empty(), "Extra wires were left!");
        self.global_nets.insert(net.name, new_net);
    }
    pub fn main_net(&mut self) -> Net {
        self.global_nets.get("Main").unwrap().clone()
    }
    fn compile_multicut(&mut self, name: String, trees: Vec<Tree>) {
        let mut net = self.global_nets.get(&name).unwrap().clone();
        let new_net_id = self.make_new_net_id();
        let mut new_vars = vec![];
        let mut new_index = 0;
        for wire in trees.into_iter() {
            let Tree::Var(wire) = wire else {
                unreachable!()
            };
            let (part_net_id, addr) = self.wire_to_nets.remove(&wire).unwrap();
            let (part_net, part_wires) = self.nets.remove(&part_net_id).unwrap();
            for part_wire in part_wires {
                if wire != part_wire {
                    self.wire_to_nets.insert(part_wire, (new_net_id, new_index));
                    new_vars.push(part_wire);
                    new_index += 1;
                }
            }
            net = Net::cut(net, 0, part_net, addr);
        }
        self.nets.insert(new_net_id, (net, new_vars));
    }

    fn compile_monocut(&mut self, left: Tree, right: Tree) {
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
                    panic!(
                        "Found var monocut that is neither a cut nor a wire: {} = {}",
                        a, b
                    )
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
                                        assert!(net_id == net, "Wires from the same partition were found to be from different nets!");
                                    } else {
                                        net_id = Some(net);
                                    };

                                    included_vars.insert(wire);
                                    addresses.push(addr);
                                }
                                let (net, old_wires) = self
                                    .nets
                                    .remove(&net_id.expect("No net!"))
                                    .expect("Net for partition or box not found!");
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
                    unimplemented!("Unknown symbol in monocut");
                }
            }
            (left, right) => unreachable!("{:?} {:?}", left, right),
        }
    }
}
