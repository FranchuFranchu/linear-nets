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

    Any(usize, Box<Type>),
    All(usize, Box<Type>),

    /// A propositional variable introduced in an axiom link
    Var(usize, bool),
    /// A formula that is quantified over by a forall, and can't be unified with other variables.
    Eigenvar(usize, bool),
    /// A formula imṕlicitly introduced in an additive.
    Hole,

    Error,
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
            Type::Plus(a, b) => Type::With(Box::new(!*a), Box::new(!*b)),
            Type::With(a, b) => Type::Plus(Box::new(!*a), Box::new(!*b)),
            Type::Ofc(a) => Type::Why(Box::new(!*a)),
            Type::Why(a) => Type::Ofc(Box::new(!*a)),
            Type::Var(a, b) => Type::Var(a, !b),
            Type::Eigenvar(a, b) => Type::Eigenvar(a, !b),
            Type::All(a, b) => Type::Any(a, Box::new(!*b)),
            Type::Any(a, b) => Type::All(a, Box::new(!*b)),
            Type::Hole => Type::Hole,
            Type::Error => Type::Error,
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
            Type::Ofc(a) | Type::Why(a) | Type::All(_, a) | Type::Any(_, a) => {
                a.replace(k, v);
            }
            Type::One
            | Type::False
            | Type::Zero
            | Type::True
            | Type::Hole
            | Type::Error
            | Type::Eigenvar(_, _) => (),
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
            Type::Ofc(a) | Type::Why(a) | Type::Any(_, a) | Type::All(_, a) => {
                a.replace_vars(f);
            }
            Type::One | Type::False | Type::Zero | Type::True | Type::Hole | Type::Error => (),
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
            Type::Ofc(a) | Type::Why(a) | Type::All(_, a) | Type::Any(_, a) => a.as_ref().var_set(),
            Type::One
            | Type::False
            | Type::Zero
            | Type::True
            | Type::Hole
            | Type::Error
            | Type::Eigenvar(..) => BTreeSet::new(),
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

        pub fn var_set(&self, tree: &Type) -> BTreeSet<usize> {
            let mut set = BTreeSet::new();
            for i in tree.var_set() {
                set.insert(i);
                if let Some(i) = self.vars_concrete.get(&(i, false)) {
                    set.append(&mut self.var_set(i))
                }
            }
            set
        }
        pub fn make_var_concrete(&mut self, id: usize, flip: bool, b: Type) -> Type {
            match self.vars_concrete.get(&(id, flip)) {
                Some(a) => self.unify(a.clone(), b.clone()),
                None => {
                    self.vars_concrete.insert((id, flip), b.clone());
                    self.vars_concrete.insert((id, !flip), !b.clone());
                    b
                }
            }
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
                (Type::Why(a), Type::Why(b)) => Type::Why(Box::new(self.unify(*a, *b))),
                (Type::Ofc(a), Type::Ofc(b)) => Type::Ofc(Box::new(self.unify(*a, *b))),
                // TODO: Is this correct?
                (Type::Var(a0, a1), Type::Var(b0, b1)) => {
                    if a0 == b0 && a1 != b1 {
                        return Type::Error;
                    }
                    if a0 == b0 {
                        return Type::Var(a0, a1);
                    }
                    match (
                        self.vars_concrete.get(&(a0, a1)),
                        self.vars_concrete.get(&(b0, b1)),
                    ) {
                        (Some(a), Some(b)) => self.unify(a.clone(), b.clone()),
                        (None, Some(b)) => self.make_var_concrete(a0, a1, b.clone()),
                        (Some(a), None) => self.make_var_concrete(b0, b1, a.clone()),
                        (None, None) => {
                            self.vars_concrete.insert((a0, a1), Type::Var(b0, b1));
                            self.vars_concrete.insert((a0, !a1), Type::Var(b0, !b1));
                            Type::Var(b0, b1)
                        }
                    }
                }
                (Type::Var(a0, a1), b) | (b, Type::Var(a0, a1)) => {
                    self.make_var_concrete(a0, a1, b)
                }
                a => {
                    eprintln!("Unification error: {:?}", a);
                    Type::Error
                }
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
                            self.unify(other_inp_t, Type::Ofc(Box::new(!inp_t)));

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
                        Cell::Cntr((a, b)) => {
                            let a_t = self.infer(a);
                            let b_t = self.infer(b);
                            if matches!((&a_t, &b_t), (Type::Why(..), Type::Why(..))) {
                                self.unify(a_t, b_t)
                            } else if let (Type::Var(a_id, a_pol), Type::Var(b_id, b_pol)) =
                                (&a_t, &b_t)
                            {
                                let v1 = self.make_new_var();
                                self.vars_concrete.insert(
                                    (*a_id, *a_pol),
                                    Type::Why(Box::new(Type::Var(v1, false))),
                                );
                                self.vars_concrete.insert(
                                    (*a_id, !*a_pol),
                                    !Type::Why(Box::new(Type::Var(v1, false))),
                                );
                                self.vars_concrete.insert(
                                    (*b_id, *b_pol),
                                    Type::Why(Box::new(Type::Var(v1, false))),
                                );
                                self.vars_concrete.insert(
                                    (*b_id, !*b_pol),
                                    !Type::Why(Box::new(Type::Var(v1, false))),
                                );
                                Type::Why(Box::new(Type::Var(v1, false)))
                            } else {
                                Type::Error
                            }
                        }
                        Cell::All((ctx,), mut net) => {
                            net.canonical();
                            let ports = core::mem::take(&mut net.ports);
                            let Ok([mut ctx_in, mut vars, mut body_in]): Result<[Type; 3], _> =
                                infer(ports.into()).try_into()
                            else {
                                return Type::Error;
                            };
                            self.freshen_vars(&mut [&mut ctx_in, &mut vars, &mut body_in]);
                            let ctx_out = self.infer(ctx);

                            let var_id = self.make_new_var();
                            let var_t = Type::Eigenvar(var_id, false);
                            self.unify(
                                !vars,
                                Type::Ofc(Box::new(Type::With(
                                    Box::new(Type::Par(
                                        Box::new(var_t.clone()),
                                        Box::new(!var_t.clone()),
                                    )),
                                    Box::new(Type::Par(
                                        Box::new(!var_t.clone()),
                                        Box::new(var_t.clone()),
                                    )),
                                ))),
                            );
                            if self.var_set(&ctx_in).contains(&var_id) {
                                eprintln!("Bad forall");
                                return Type::Error;
                            }

                            if self.unify(!ctx_out, ctx_in) == Type::Error {
                                return Type::Error;
                            }
                            Type::All(var_id, Box::new(body_in))
                        }
                        Cell::Any((ctx,), mut net) => {
                            net.canonical();
                            let ports = core::mem::take(&mut net.ports);
                            let Ok([mut ctx_in, mut vars, mut body_in]): Result<[Type; 3], _> =
                                infer(ports.into()).try_into()
                            else {
                                return Type::Error;
                            };
                            self.freshen_vars(&mut [&mut ctx_in, &mut vars, &mut body_in]);
                            let ctx_out = self.infer(ctx);

                            let var_id = self.make_new_var();
                            let var_t = Type::Var(var_id, false);
                            self.unify(
                                !vars,
                                Type::Ofc(Box::new(Type::With(
                                    Box::new(Type::Par(
                                        Box::new(var_t.clone()),
                                        Box::new(!var_t.clone()),
                                    )),
                                    Box::new(Type::Par(
                                        Box::new(!var_t.clone()),
                                        Box::new(var_t.clone()),
                                    )),
                                ))),
                            );

                            if self.unify(!ctx_out, ctx_in) == Type::Error {
                                return Type::Error;
                            }
                            Type::Any(var_id, Box::new(body_in))
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
