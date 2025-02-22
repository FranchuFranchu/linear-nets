pub mod rules;
pub mod show;
pub mod system;
// Net implementation.
// Understands simplicity and understands boxing.

use std::collections::{BTreeMap, VecDeque};
pub use system::Cell;

pub type VarId = usize;
pub type AgentId = usize;

pub fn reorder<T>(a: &mut VecDeque<T>, mut indices: VecDeque<usize>, reorder_rest: bool) -> bool {
    let mut result = VecDeque::new();
    while let Some(idx) = indices.pop_front() {
        result.push_back(a.remove(idx).unwrap());
        indices = indices
            .into_iter()
            .map(|x| if x > idx { x - 1 } else { x })
            .collect()
    }
    if a.len() > 0 {
        if reorder_rest {
            result.append(a);
        } else {
            return false;
        }
    }
    *a = result;
    return true;
}

#[derive(Debug)]
pub enum Arg {
    Partition(usize),
    Box(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolId {
    Times,
    One,

    Par,
    False,

    With,
    True,

    Left,
    Right,

    Weak,
    Dere,
    Cntr,
    Exp0,
    Exp1,

    All,
    Any,
}

impl SymbolId {
    fn args(&self) -> Vec<Arg> {
        use SymbolId::*;
        match self {
            Times => vec![Arg::Partition(1), Arg::Partition(1)],
            Par => vec![Arg::Partition(2)],
            One => vec![],
            False => vec![Arg::Partition(1), Arg::Box(1)],

            Left => vec![Arg::Partition(1)],
            Right => vec![Arg::Partition(1)],
            With => vec![Arg::Partition(1), Arg::Box(2), Arg::Box(2)],
            True => vec![Arg::Partition(1)],

            Exp0 => vec![Arg::Box(1)],
            Exp1 => vec![Arg::Partition(1), Arg::Box(2)],
            Weak => vec![Arg::Partition(1), Arg::Box(1)],
            Dere => vec![Arg::Partition(1)],
            Cntr => vec![Arg::Partition(2)],

            All => vec![Arg::Partition(1), Arg::Box(3)],
            Any => vec![Arg::Partition(1), Arg::Box(3)],
        }
    }
}

#[derive(Debug, Clone)]
pub enum GraftArg {
    // net in the partition, and list of free ports
    Partition(Net, Vec<usize>),
    // net to box, and list of free ports. All free ports must be used here.
    Box(Net, Vec<usize>),
}
#[derive(Debug, Clone)]
pub enum PartitionOrBox {
    Partition(Vec<Tree>),
    Box(Net),
}
impl PartitionOrBox {
    fn map_vars(&mut self, m: &impl Fn(VarId) -> VarId) {
        match self {
            PartitionOrBox::Partition(ports) => ports.iter_mut().for_each(|x| x.map_vars(m)),
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum Tree {
    Var(VarId),
    Agent(SymbolId, Vec<PartitionOrBox>),
}

impl Tree {
    fn map_vars(&mut self, m: &impl Fn(VarId) -> VarId) {
        use Tree::*;
        match self {
            Var(x) => *x = m(*x),
            Agent(_, ports) => ports.iter_mut().for_each(|x| x.map_vars(m)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Net {
    pub(crate) ports: VecDeque<Tree>,
    pub(crate) redexes: VecDeque<(Tree, Tree)>,
    pub(crate) vars: BTreeMap<usize, Option<Tree>>,
}

impl Net {
    fn empty() -> Net {
        Net {
            ports: vec![].into(),
            redexes: vec![].into(),
            vars: BTreeMap::new(),
        }
    }
    fn reduce(&mut self, f: fn(&mut Net, Cell, Cell)) -> bool {
        if let Some((a, b)) = self.redexes.pop_front() {
            f(
                self,
                Cell::from_tree(a).unwrap(),
                Cell::from_tree(b).unwrap(),
            );
            true
        } else {
            false
        }
    }
    pub fn normal(&mut self, f: fn(&mut Net, Cell, Cell)) {
        while self.reduce(f) {
            self.print_net_simple()
        }
    }
    pub fn link(&mut self, a: Tree, b: Tree) {
        if let Tree::Var(id) = a {
            match self.vars.remove(&id).unwrap() {
                Some(a) => {
                    self.link(a, b);
                }
                None => {
                    self.vars.insert(id, Some(b));
                }
            }
        } else if let Tree::Var(id) = b {
            self.link(Tree::Var(id), a)
        } else {
            self.redexes.push_back((a, b))
        }
    }
    pub fn map_vars(&mut self, m: &impl Fn(VarId) -> VarId) {
        self.ports.iter_mut().for_each(|x| x.map_vars(m));
        self.redexes.iter_mut().for_each(|(a, b)| {
            a.map_vars(m);
            b.map_vars(m)
        });
        let vars = core::mem::take(&mut self.vars);
        self.vars = vars
            .into_iter()
            .map(|(mut k, mut v)| {
                k = m(k);
                v.as_mut().map(|x| x.map_vars(m));
                (k, v)
            })
            .collect();
    }
    fn allocate_var_id(&mut self) -> VarId {
        for i in 0.. {
            if self.vars.get(&i).is_none() {
                return i;
            }
        }
        unreachable!();
    }
    fn create_wire(&mut self) -> (Tree, Tree) {
        let id = self.allocate_var_id();
        self.vars.insert(id, None);
        (Tree::Var(id), Tree::Var(id))
    }
    pub fn wire() -> Net {
        let mut net = Net::empty();
        let (a, b) = net.create_wire();
        net.ports.append(&mut vec![a, b].into());
        net
    }
    pub fn graft(symbol: SymbolId, args: Vec<GraftArg>) -> Net {
        let symbol_fmt = symbol.args();
        assert!(args.len() == args.len());
        let mut aux = vec![];
        let mut built_net = Net::empty();
        for (q, i) in symbol_fmt.iter().zip(args) {
            match (q, i) {
                (Arg::Box(size), GraftArg::Box(mut net, ports)) => {
                    assert!(*size == ports.len(), "Incorrect box free port size");
                    reorder(&mut net.ports, ports.into(), false);
                    aux.push(PartitionOrBox::Box(net));
                }
                (Arg::Partition(size), GraftArg::Partition(mut net, ports)) => {
                    assert!(*size == ports.len());
                    reorder(&mut net.ports, ports.into(), true);
                    let mut ports = vec![];
                    for _i in 0..*size {
                        ports.push(net.ports.pop_front().unwrap());
                    }
                    let var_map = built_net.shift_map();
                    built_net = built_net.mix(net);
                    ports.iter_mut().for_each(|x| x.map_vars(&var_map));
                    aux.push(PartitionOrBox::Partition(ports));
                }
                _ => {
                    panic!("Incorrect partitioning!")
                }
            }
        }
        built_net.ports.push_front(Tree::Agent(symbol, aux));
        built_net
    }
    fn shift_map(&self) -> impl Fn(VarId) -> VarId {
        let vars = self.vars.keys().fold(0, |b, a| (*a).max(b)) + 1;
        Box::new(move |x| x + vars)
    }
    fn mix(mut self, mut other: Net) -> Net {
        let map = self.shift_map();
        other.map_vars(&map);
        self.ports.append(&mut other.ports);
        self.redexes.append(&mut other.redexes);
        self.vars.append(&mut other.vars);
        self
    }
    pub fn cut(this: Net, this_port: usize, other: Net, other_port: usize) -> Net {
        let this_len = this.ports.len();
        let mut composite = this.mix(other);
        let port_a = composite.ports.remove(this_port).unwrap();
        let port_b = composite.ports.remove(other_port + this_len - 1).unwrap();
        composite.link(port_a, port_b);
        return composite;
    }
    fn plug_box(&mut self, mut other: Net, ports: Vec<Tree>) {
        let m = self.shift_map();
        let other_ports = core::mem::take(&mut other.ports);
        let mut s = core::mem::replace(self, Net::empty());
        s = Net::mix(s, other);
        *self = s;
        for (mut op, sp) in other_ports.into_iter().zip(ports.into_iter()) {
            op.map_vars(&m);
            self.link(op, sp)
        }
    }
    pub fn canonical(&mut self) {
        let mut ports = core::mem::take(&mut self.ports);
        for i in ports.iter_mut() {
            self.substitute_mut(i);
        }
        self.ports = ports;
        let mut redexes = core::mem::take(&mut self.redexes);
        for (a, b) in redexes.iter_mut() {
            self.substitute_mut(a);
            self.substitute_mut(b);
        }
        self.redexes = redexes;
    }

    pub fn substitute_ref(&self, tree: &Tree) -> Tree {
        fn substitute_ref_aux(this: &Net, aux: &PartitionOrBox) -> PartitionOrBox {
            match aux {
                PartitionOrBox::Partition(a) => PartitionOrBox::Partition(
                    a.into_iter().map(|x| this.substitute_ref(x)).collect(),
                ),
                PartitionOrBox::Box(b) => PartitionOrBox::Box(b.clone()),
            }
        }
        match tree {
            Tree::Agent(id, aux) => Tree::Agent(
                id.clone(),
                aux.into_iter()
                    .map(|x| substitute_ref_aux(self, x))
                    .collect(),
            ),
            Tree::Var(id) => {
                if let Some(Some(b)) = self.vars.get(id) {
                    self.substitute_ref(b)
                } else {
                    Tree::Var(*id)
                }
            }
        }
    }

    pub fn substitute_mut(&mut self, tree: &mut Tree) {
        fn substitute_mut_aux(this: &mut Net, aux: &mut PartitionOrBox) {
            match aux {
                PartitionOrBox::Partition(a) => a.iter_mut().for_each(|x| this.substitute_mut(x)),
                PartitionOrBox::Box(b) => b.canonical(),
            }
        }
        match tree {
            Tree::Agent(_, aux) => aux.iter_mut().for_each(|x| substitute_mut_aux(self, x)),
            s => {
                let Tree::Var(id) = &s else { unreachable!() };
                if let Some(Some(_)) = self.vars.get(id) {
                    let Some(Some(mut w)) = self.vars.remove(id) else {
                        unreachable!()
                    };
                    self.substitute_mut(&mut w);
                    *s = w;
                } else {
                }
            }
        }
    }
    pub fn substitute_iter<'a>(&self, trees: impl Iterator<Item = &'a Tree>) -> Vec<Tree> {
        trees.map(|tree| self.substitute_ref(tree)).collect()
    }
}
