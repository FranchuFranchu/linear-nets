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
            _ => None,
        }
    }
    pub fn from_tree(tree: Tree) -> Option<Cell> {
        match tree {
            Tree::Var(_id) => None,
            Tree::Agent(symbol, args) => Self::from_symbol_args(symbol, args),
        }
    }
}
