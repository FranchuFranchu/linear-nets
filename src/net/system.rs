use crate::net::{Net, PartitionOrBox, SymbolId, Tree};

pub enum Cell {
    Times((Tree,), (Tree,)),
    Par((Tree, Tree)),
    One(),
    False((Tree,), Net),
    Left((Tree,)),
    Right((Tree,)),
    With((Tree,), Net, Net),
    True((Tree,)),

    Exp0(Net),
    Exp1((Tree,), Net),
    Weak((Tree,), Net),
    Dere((Tree,)),
    Cntr((Tree, Tree)),

    // All:
    // Var out port: typed with eigenvariable, non-unifiable
    // Var in port: Must be variable
    // Any:
    // Witness in port: Typed with B
    // Witness out port: Typed with B
    // They have 4 ports:
    // - Context out
    // - Context in
    // - Var/Witness inout
    // - Body in
    // The last three are boxed together
    All((Tree,), Net),
    Any((Tree,), Net),
}

impl Cell {
    fn from_symbol_args(symbol: SymbolId, args: Vec<PartitionOrBox>) -> Option<Cell> {
        match symbol {
            SymbolId::Times => {
                let [PartitionOrBox::Partition(a), PartitionOrBox::Partition(b)]: [PartitionOrBox;
                    2] = args.try_into().ok()?
                else {
                    return None;
                };
                let [a] = a.try_into().ok()?;
                let [b] = b.try_into().ok()?;
                Some(Cell::Times((a,), (b,)))
            }
            SymbolId::Par => {
                let [PartitionOrBox::Partition(a)]: [PartitionOrBox; 1] = args.try_into().ok()?
                else {
                    return None;
                };
                let [a, b] = a.try_into().ok()?;
                Some(Cell::Par((a, b)))
            }
            SymbolId::One => {
                let []: [PartitionOrBox; 0] = args.try_into().ok()?;
                Some(Cell::One())
            }
            SymbolId::False => {
                let [PartitionOrBox::Partition(a), PartitionOrBox::Box(b)]: [PartitionOrBox; 2] =
                    args.try_into().ok()?
                else {
                    return None;
                };
                let [a] = a.try_into().unwrap();
                Some(Cell::False((a,), b))
            }
            SymbolId::Left => {
                let [PartitionOrBox::Partition(a)]: [PartitionOrBox; 1] = args.try_into().ok()?
                else {
                    return None;
                };
                let [a] = a.try_into().ok()?;
                Some(Cell::Left((a,)))
            }
            SymbolId::Right => {
                let [PartitionOrBox::Partition(a)]: [PartitionOrBox; 1] = args.try_into().ok()?
                else {
                    return None;
                };
                let [a] = a.try_into().ok()?;
                Some(Cell::Right((a,)))
            }
            SymbolId::With => {
                let [PartitionOrBox::Partition(a), PartitionOrBox::Box(left), PartitionOrBox::Box(right)]: [PartitionOrBox; 3] = args.try_into().ok()?
                else {
                    return None;
                };
                let [a] = a.try_into().ok()?;
                Some(Cell::With((a,), left, right))
            }
            SymbolId::True => {
                let [PartitionOrBox::Partition(a)]: [PartitionOrBox; 1] = args.try_into().ok()?
                else {
                    return None;
                };
                let [a] = a.try_into().ok()?;
                Some(Cell::True((a,)))
            }
            SymbolId::Exp0 => {
                let [PartitionOrBox::Box(b)]: [PartitionOrBox; 1] = args.try_into().ok()? else {
                    return None;
                };
                Some(Cell::Exp0(b))
            }
            SymbolId::Exp1 => {
                let [PartitionOrBox::Partition(a), PartitionOrBox::Box(b)]: [PartitionOrBox; 2] =
                    args.try_into().ok()?
                else {
                    return None;
                };
                let [a] = a.try_into().unwrap();
                Some(Cell::Exp1((a,), b))
            }
            SymbolId::Weak => {
                let [PartitionOrBox::Partition(a), PartitionOrBox::Box(b)]: [PartitionOrBox; 2] =
                    args.try_into().ok()?
                else {
                    return None;
                };
                let [a] = a.try_into().unwrap();
                Some(Cell::Weak((a,), b))
            }
            SymbolId::Dere => {
                let [PartitionOrBox::Partition(a)]: [PartitionOrBox; 1] = args.try_into().ok()?
                else {
                    return None;
                };
                let [a] = a.try_into().ok()?;
                Some(Cell::Dere((a,)))
            }
            SymbolId::Cntr => {
                let [PartitionOrBox::Partition(a)]: [PartitionOrBox; 1] = args.try_into().ok()?
                else {
                    return None;
                };
                let [a, b] = a.try_into().ok()?;
                Some(Cell::Cntr((a, b)))
            }

            SymbolId::All => {
                let [PartitionOrBox::Partition(a), PartitionOrBox::Box(b)]: [PartitionOrBox; 2] =
                    args.try_into().ok()?
                else {
                    return None;
                };
                let [a] = a.try_into().unwrap();
                Some(Cell::All((a,), b))
            }
            SymbolId::Any => {
                let [PartitionOrBox::Partition(a), PartitionOrBox::Box(b)]: [PartitionOrBox; 2] =
                    args.try_into().ok()?
                else {
                    return None;
                };
                let [a] = a.try_into().unwrap();
                Some(Cell::Any((a,), b))
            }
        }
    }
    pub fn from_tree(tree: Tree) -> Option<Cell> {
        match tree {
            Tree::Var(_id) => None,
            Tree::Agent(symbol, args) => Self::from_symbol_args(symbol, args),
        }
    }
    pub fn to_tree(self) -> Tree {
        match self {
            Cell::Times(_, _) => todo!(),
            Cell::Par(_) => todo!(),
            Cell::One() => todo!(),
            Cell::False(_, _) => todo!(),
            Cell::Left(_) => todo!(),
            Cell::Right(_) => todo!(),
            Cell::With(_, _, _) => todo!(),
            Cell::True(_) => todo!(),
            Cell::Exp0(a) => Tree::Agent(SymbolId::Exp0, vec![PartitionOrBox::Box(a)]),
            Cell::Exp1((a,), b) => Tree::Agent(
                SymbolId::Exp1,
                vec![PartitionOrBox::Partition(vec![a]), PartitionOrBox::Box(b)],
            ),
            Cell::Weak(_, _) => todo!(),
            Cell::Dere(_) => todo!(),
            Cell::Cntr(..) => todo!(),
            Cell::All(..) => todo!(),
            Cell::Any(..) => todo!(),
        }
    }
}
