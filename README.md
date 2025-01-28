# `linear-nets`

A plan to build simple and type-checked nets by construction. Work in progress.

Reads input file from stdin.

## To-Do list

- [X] Simple net building implementation
- [X] Implement proof net reduction rules
- [X] Implement typesystem reduction rules
- [X] Implement parser
- [X] Implement AST -> Net compiler
  - [X] Implement composing nets together
- [X] Add better displaying of nets
- [X] Add tests
- [X] Interaction combinator compiler for proof net
- [ ] Interaction combinator compiler for typesystem
- [X] Compiler from interaction combinators to external runtimes.

## Interaction Combinator translation

```
T[Times(a)(b)] = Con(T[a] T[b])
T[Par(a b)] = Con(T[a] T[b])
T[Left(a)] = Con(x Con(Con(x T[a]) Era))
T[Right(b)] = Con(x Con(Era Con(x T[b])))
T[With(ctx)[ca va][cb vb]) = Con(T[ctx] Con(Con(T[ca] T[va]) Con(T[cb] T[vb])))
T[Exp0[a]] = LafontCode[T[a]] // Con(Con(dup_in tree) Con(dup_0 dup_1))
T[Exp1(context_tree)[context_hole value]] = {
  T[context_tree] = Con(Con(c T[context_hole]) Con(a b))
  LafontCode[T[value]] = Con(Con(f g) Con(d e))
  Con(Con(Con(c f) g) Con(Con(a d) Con(b e))
}
T[Weak(c)[a]] = {
  c = a
  Era
}
T[Dere(out)] = Con(Con(Dup(a b) T[out]) (a b))
T[Cntr(a)(b)] = Dup(a b)
```
