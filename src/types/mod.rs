use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use crate::net::{Tree, Cell};

pub mod show;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
  Times(Box<Type>, Box<Type>),
  One,
  Par(Box<Type>, Box<Type>),
  False,
  
  Plus(Box<Type>, Box<Type>),
  Zero,
  With(Box<Type>, Box<Type>),
  True,

  Why(Box<Type>),
  Ofc(Box<Type>),

  Any(Box<Type>),
  All(Box<Type>),
  
  Var(usize, bool),

  Error,
}

impl std::ops::Not for Type {
  type Output = Type;

  fn not(self) -> Type {
    match self {
      Type::One => Type::False,
      Type::False => Type::One,
      Type::Var(a, b) => Type::Var(a, !b),
      Type::Times(a, b) => Type::Par(Box::new(!*a), Box::new(!*b)),
      Type::Par(a, b) => Type::Times(Box::new(!*a), Box::new(!*b)),
      _ => todo!(),
    }
  }
}

impl Type {
  fn replace(&mut self, k: (usize, bool), v: Type) {
    match self {
      Type::Var(ka, kb) if (*ka, *kb) == k => {
        *self = v;
      },
      Type::Var(..) => {},
      Type::Times(a, b) => {
        a.replace(k, v.clone());
        b.replace(k, v);
      },
      Type::One | Type::False | Type::Zero | Type::True => (),
      _ => todo!(),
    }
  }
}

pub fn infer(trees: Vec<Tree>) -> Vec<Type> {
  #[derive(Default)]
  struct State {
    tree_vars: BTreeMap<usize, Type>,
    vars_concrete: BTreeMap<(usize, bool), Type>,
    new_var: usize,
  }
  impl State {
    fn unify(&mut self, a: Type, b: Type) -> Type {
      match (a, b) {
        (Type::One, Type::One) => Type::One,
        (Type::False, Type::False) => Type::False,
        (Type::Times(a0, a1), Type::Times(b0, b1)) => Type::Times(Box::new(self.unify(*a0, *b0)), Box::new(self.unify(*a1, *b1))),
        (Type::Par(a0, a1), Type::Par(b0, b1)) => Type::Par(Box::new(self.unify(*a0, *b0)), Box::new(self.unify(*a1, *b1))),
        (Type::Var(_a0, _a1), Type::Var(_b0, _b1)) => todo!(),
        (Type::Var(a0, a1), b) | (b, Type::Var(a0, a1)) => {
          self.vars_concrete.insert((a0, a1), b.clone());
          self.vars_concrete.insert((a0, !a1), !b.clone());
          b
        },
        _ => Type::Error
      }
    }
    fn infer(&mut self, tree: Tree) -> Type {
      match tree {
        Tree::Var(id) => {
          match self.tree_vars.entry(id) {
            Entry::Occupied(e) => {
              e.remove()
            }
            Entry::Vacant(e) => {
              self.new_var += 1;
              e.insert(Type::Var(self.new_var, true));
              Type::Var(self.new_var, false)
            }
          }
        },
        Tree::Agent(a, b) => {
          let Some(cell) = Cell::from_tree(Tree::Agent(a, b)) else { return Type::Error };
          match cell {
            Cell::Times((a,), (b,)) => {
              let ta = self.infer(a);
              let tb = self.infer(b);
              Type::Times(Box::new(ta), Box::new(tb))
            },
            Cell::Par((a,b)) => {
              let ta = self.infer(a);
              let tb = self.infer(b);
              Type::Par(Box::new(ta), Box::new(tb))
            },
            Cell::One() => Type::One,
            Cell::False((a,), mut b) => {
              b.normal(crate::net::rules::apply_rule);
              let ports = core::mem::take(&mut b.ports);
              let Ok([t0]): Result<[Type; 1], _> = infer(ports.into()).try_into() else { return Type::Error };
              let t1 = self.infer(a);
              let tt = self.unify(t0, !t1);
              if tt != Type::Error {
                Type::False
              } else {
                Type::Error
              }
            }
            _ => todo!(),
          }
        }
      }
    }
  }
  let mut state = State::default();
  let mut types: Vec<_> = trees.into_iter().map(|x| state.infer(x)).collect();
  for (k, v) in state.vars_concrete.into_iter() {
    for i in types.iter_mut() {
      i.replace(k, v.clone());
    }
  }
  types
}

