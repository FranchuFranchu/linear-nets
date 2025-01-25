use crate::net::{Cell, Tree};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

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
    Hole,
}

impl std::ops::Not for Type {
    type Output = Type;

    fn not(self) -> Type {
        match self {
            Type::One => Type::False,
            Type::False => Type::One,
            Type::Times(a, b) => Type::Par(Box::new(!*a), Box::new(!*b)),
            Type::Par(a, b) => Type::Times(Box::new(!*a), Box::new(!*b)),
            Type::True => Type::Zero,
            Type::Zero => Type::True,
            Type::Plus(a, b) => Type::Plus(Box::new(!*a), Box::new(!*b)),
            Type::With(a, b) => Type::With(Box::new(!*a), Box::new(!*b)),
            Type::Var(a, b) => Type::Var(a, !b),
            Type::Hole => Type::Hole,
            _ => todo!(),
        }
    }
}

impl Type {
    fn replace(&mut self, k: (usize, bool), v: Type) {
        match self {
            Type::Var(ka, kb) if (*ka, *kb) == k => {
                *self = v;
            }
            Type::Var(..) => {}
            Type::Times(a, b) | Type::Par(a, b) | Type::Plus(a, b) | Type::With(a, b) => {
                a.replace(k, v.clone());
                b.replace(k, v);
            }
            Type::One | Type::False | Type::Zero | Type::True | Type::Hole => (),
            _ => todo!(),
        }
    }
    fn replace_vars(&mut self, f: &impl Fn(usize) -> usize) {
        match self {
            Type::Var(ka, _) => {
                *ka = f(*ka);
            }
            Type::Times(a, b) | Type::Par(a, b) | Type::Plus(a, b) | Type::With(a, b) => {
                a.replace_vars(f);
                b.replace_vars(f);
            }
            Type::One | Type::False | Type::Zero | Type::True | Type::Hole => (),
            _ => todo!(),
        }
    }
    fn var_set(&self) -> BTreeSet<usize> {
        match self {
            Type::Var(ka, _) => BTreeSet::from([*ka]),
            Type::Times(a, b) | Type::Par(a, b) | Type::Plus(a, b) | Type::With(a, b) => {
                let mut vs = a.as_ref().var_set();
                vs.append(&mut b.as_ref().var_set());
                vs
            }
            Type::One | Type::False | Type::Zero | Type::True | Type::Hole => BTreeSet::new(),
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
        fn make_new_var(&mut self) -> usize {
            self.new_var += 1;
            self.new_var - 1
        }
        fn unify(&mut self, a: Type, b: Type) -> Type {
            match (a, b) {
                (Type::Hole, a) => a,
                (a, Type::Hole) => a,
                (Type::One, Type::One) => Type::One,
                (Type::False, Type::False) => Type::False,
                (Type::Zero, Type::Zero) => Type::Zero,
                (Type::True, Type::True) => Type::True,
                (Type::Times(a0, a1), Type::Times(b0, b1)) => Type::Times(
                    Box::new(self.unify(*a0, *b0)),
                    Box::new(self.unify(*a1, *b1)),
                ),
                (Type::Par(a0, a1), Type::Par(b0, b1)) => Type::Par(
                    Box::new(self.unify(*a0, *b0)),
                    Box::new(self.unify(*a1, *b1)),
                ),
                (Type::Plus(a0, a1), Type::Plus(b0, b1)) => Type::Plus(
                    Box::new(self.unify(*a0, *b0)),
                    Box::new(self.unify(*a1, *b1)),
                ),
                (Type::With(a0, a1), Type::With(b0, b1)) => Type::With(
                    Box::new(self.unify(*a0, *b0)),
                    Box::new(self.unify(*a1, *b1)),
                ),
                // TODO: Is this correct?
                (Type::Var(a0, a1), Type::Var(b0, b1)) => {
                    match (
                        self.vars_concrete.get(&(a0, a1)),
                        self.vars_concrete.get(&(b0, b1)),
                    ) {
                        (Some(a), Some(b)) => self.unify(a.clone(), b.clone()),
                        (None, Some(b)) => self.unify(Type::Var(a0, a1), b.clone()),
                        (Some(a), None) => self.unify(a.clone(), Type::Var(b0, b1)),
                        (None, None) => {
                            self.vars_concrete.insert((a0, a1), Type::Var(b0, b1));
                            self.vars_concrete.insert((a0, !a1), Type::Var(b0, !b1));
                            Type::Var(b0, b1)
                        }
                    }
                }
                (Type::Var(a0, a1), b) | (b, Type::Var(a0, a1)) => {
                    self.vars_concrete.insert((a0, a1), b.clone());
                    self.vars_concrete.insert((a0, !a1), !b.clone());
                    b
                }
                _ => Type::Error,
            }
        }
        fn freshen_vars(&mut self, types: &mut [&mut Type]) {
            let mut map = BTreeMap::new();
            for t in &mut *types {
                for old in t.var_set() {
                    if !map.contains_key(&old) {
                        let new = self.make_new_var();
                        map.insert(old, new);
                    }
                }
            }
            for t in &mut *types {
                t.replace_vars(&|old| *map.get(&old).unwrap());
            }
        }
        fn infer(&mut self, tree: Tree) -> Type {
            match tree {
                Tree::Var(id) => match self.tree_vars.entry(id) {
                    Entry::Occupied(e) => e.remove(),
                    Entry::Vacant(e) => {
                        self.new_var += 1;
                        let new_var = self.new_var - 1;
                        e.insert(Type::Var(new_var, true));
                        Type::Var(new_var, false)
                    }
                },
                Tree::Agent(a, b) => {
                    let Some(cell) = Cell::from_tree(Tree::Agent(a, b)) else {
                        return Type::Error;
                    };
                    match cell {
                        Cell::Times((a,), (b,)) => {
                            let ta = self.infer(a);
                            let tb = self.infer(b);
                            Type::Times(Box::new(ta), Box::new(tb))
                        }
                        Cell::Par((a, b)) => {
                            let ta = self.infer(a);
                            let tb = self.infer(b);
                            Type::Par(Box::new(ta), Box::new(tb))
                        }
                        Cell::One() => Type::One,
                        Cell::False((a,), mut b) => {
                            b.normal(crate::net::rules::apply_rule);
                            let mut ports = core::mem::take(&mut b.ports);
                            ports.iter_mut().for_each(|x| *x = b.substitute_ref(x));
                            let Ok([mut t0]): Result<[Type; 1], _> = infer(ports.into()).try_into()
                            else {
                                return Type::Error;
                            };
                            let t1 = self.infer(a);
                            self.freshen_vars(&mut [&mut t0]);
                            let tt = self.unify(t0, !t1);
                            if tt != Type::Error {
                                Type::False
                            } else {
                                Type::Error
                            }
                        }
                        Cell::Left((out,)) => {
                            Type::Plus(Box::new(self.infer(out)), Box::new(Type::Hole))
                        }
                        Cell::Right((out,)) => {
                            Type::Plus(Box::new(Type::Hole), Box::new(self.infer(out)))
                        }
                        Cell::True((out,)) => {
                            self.infer(out);
                            Type::True
                        }
                        Cell::With((ctx,), mut left, mut right) => {
                            left.normal(crate::net::rules::apply_rule);
                            right.normal(crate::net::rules::apply_rule);

                            let mut ports = core::mem::take(&mut left.ports);
                            ports.iter_mut().for_each(|x| *x = left.substitute_ref(x));
                            let Ok([mut tvl, mut tcl]): Result<[Type; 2], _> =
                                infer(ports.into()).try_into()
                            else {
                                return Type::Error;
                            };
                            self.freshen_vars(&mut [&mut tvl, &mut tcl]);

                            let mut ports = core::mem::take(&mut right.ports);
                            ports.iter_mut().for_each(|x| *x = right.substitute_ref(x));
                            let Ok([mut tvr, mut tcr]): Result<[Type; 2], _> =
                                infer(ports.into()).try_into()
                            else {
                                return Type::Error;
                            };
                            self.freshen_vars(&mut [&mut tvr, &mut tcr]);

                            let tctx = self.infer(ctx);
                            let tc = self.unify(tcl, tcr);
                            if self.unify(!tctx, tc) != Type::Error {
                                Type::With(Box::new(tvl), Box::new(tvr))
                            } else {
                                Type::Error
                            }
                        }
                        Cell::Exp0(mut net) => {
                            net.normal(crate::net::rules::apply_rule);

                            let mut ports = core::mem::take(&mut net.ports);
                            ports.iter_mut().for_each(|x| *x = net.substitute_ref(x));
                            let Ok([mut t]): Result<[Type; 1], _> = infer(ports.into()).try_into()
                            else {
                                return Type::Error;
                            };
                            self.freshen_vars(&mut [&mut t]);

                            Type::Ofc(Box::new(t))
                        }
                        Cell::Exp1((inp,), mut net) => {
                            net.normal(crate::net::rules::apply_rule);

                            let mut ports = core::mem::take(&mut net.ports);
                            ports.iter_mut().for_each(|x| *x = net.substitute_ref(x));
                            let Ok([mut t, inp_t]): Result<[Type; 2], _> =
                                infer(ports.into()).try_into()
                            else {
                                return Type::Error;
                            };
                            self.freshen_vars(&mut [&mut t]);

                            let other_inp_t = self.infer(inp);
                            self.unify(other_inp_t, !inp_t);

                            Type::Ofc(Box::new(t))
                        }
                        Cell::Weak((ctx,), mut net) => {
                            let mut ports = core::mem::take(&mut net.ports);
                            ports.iter_mut().for_each(|x| *x = net.substitute_ref(x));
                            let Ok([mut t]): Result<[Type; 1], _> = infer(ports.into()).try_into()
                            else {
                                return Type::Error;
                            };
                            self.freshen_vars(&mut [&mut t]);

                            let c_t = self.infer(ctx);

                            self.unify(c_t, !t);

                            Type::Why(Box::new(Type::Hole))
                        }
                        Cell::Dere((a,)) => Type::Why(Box::new(self.infer(a))),
                        Cell::Cntr((a,), (b,)) => {
                            let a_t = self.infer(a);
                            let b_t = self.infer(b);
                            if matches!((&a_t, &b_t), (Type::Why(..), Type::Why(..))) {
                                self.unify(a_t, b_t)
                            } else {
                                Type::Error
                            }
                        }
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
