# `linear-nets`

A plan to build simple and type-checked nets by construction. Work in progress.

Reads input file from stdin.

- [X] Simple net building implementation
- [ ] Implement proof net reduction rules
  - [X] Times, Par, 1, False
  - [X] Left, Right, With, True
  - [X] Exp, Weak, Dere, Cntr
  - [ ] All, Any
- [ ] Implement typesystem reduction rules
  - [X] Times, Par, 1, False
  - [X] Left, Right, With, True
  - [X] Exp, Weak, Dere, Cntr
  - [ ] All, Any
- [X] Implement parser
- [X] Implement AST -> Net compiler
  - [X] Implement composing nets together
- [X] Add better displaying of nets
- [X] Add tests
- [X] Interaction combinator compiler for proof net
- [ ] Interaction combinator compiler for typesystem
- [X] Compiler from interaction combinators to external runtimes.
