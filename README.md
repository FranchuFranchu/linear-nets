# `linear-nets`

A language to write vicious-circle-free and well-typed programs using Linear Logic.

## Usage

```sh
# The program reads its input program from standard input.
cargo run < test.line
```

`linear-nets` also includes optional `hvm` and `ivm` features which will output the resulting interaction combinator net in `HVM2` or `ivm` format.

## Introduction

Linear Logic [^1] is a substructural logic which rejects the rules of _weakening_ and _contraction_. This means that, "by default", proofs must be used _exactly once_.

For example, this is not provable in Linear Logic.

$$ \forall A . A \to A \land A $$

To prove this, you'd need to somehow have a way to duplicate A. In traditional logical system, it's possible to do this with any proposition, but in linear logic, this is not possible. This means that in linear logic, propositions are similar to _resources_; if you want to use one twice, you need to make two of them.

An additional property of linear logic is that it disallows _mixing_. This guarantees independent proofs remain separate, while proofs that are joined together are guaranteed to depend on each other in a way.

This is very interesting for interaction nets, because in interaction nets [^2], wires have exactly two ports. Values, by default, must be used exactly once, and erasing and duplication is not defined to begin with. This is not the case in Lambda Calculus, where duplicating and erasure is implicit in beta-reduction. In fact, historically, interaction nets were first formulated as a _generalization_ of a concept in linear logic called _proof nets_ [^1].

In the last few years a lot of interaction net languages sprung up, usually based on Interaction Combinators. However, many of them suffer from _vicious circles_, which is a type of structure in an interaction net in which the value of a variable depends on itself. Vicious circles cause a program to halt without producing a final reduced value, and they're hard to debug.

Additionally, most type systems devised for these languages are based on λ-calculus and are not as well-suited for interaction nets.

This is an attempt to fix both of these problems. By writing Linear Logic proof nets as interaction nets, we can type these nets using logical formuals. The proof nets, by their construction properties, are guaranteed to be vicious-circle-free and connected, and remain so during the whole normalization process.

Lafont has proven that any interaction system can be translated to Interaction Combinators [^3]. This is the case for proof nets too, since they too are an interaction system. We can translate proof nets to interaction combinators, and they'll behave as programs that will remain vicious-circle-free and well-typed.

## How it works:

Lafont proved that interaction nets that are built according to a specific set of rules will be _simple_. Simple nets are vicious-circle-free, and simplicity is preserved by reduction. I heavily recommend reading and understand his paper first before reading this section.

The simplicity operations operate on a set of nets, and return a net a result. Each net has a set of free ports.

There are three simplicity operations, which are called `Wire`, `Graft`, and `Cut`. These are explained in the paper. They take in a set of nets and return a single unified net as a result.

A `linear-nets` program consists of a number of _definitions_, for example:
```
Foo(a b out) {
  # ...
}
Bar(z) {
  # ...
}
...
```

Each definition consists of a list of instructions, which say how to construct the net which has the wires named in the definition as free ports. Each instruction is either a simplicity opereation, or many operations bundled into one. `linear-nets` will desugar compound instructions into many instructions The order of the instructions of a definition is relevant.

Instructions are either _monocuts_ or _multicuts_. A monocut can either be a cut, a wire, or a graft. We'll look at multicuts later (TODO) since they're an extension.

A monocut creates a link between two entities. They can either be two cells, a cell and a variable, or two variable.

```
# Two variables. This is a wire, but can also be a cut depending on the other instructions.
a = b
# A cell and a var. This is a graft, but can also be a cut depending on the other instructions.
Times(a)(b) = c
# Two cells. This is always a cut.
Times(a)(b) = Par(c d)
```

Each variable is implicitly part of a _net_, which says which net is this variable a free port of. After a simplicity operation, all free ports end up being part of the same net.

Parentheses and brackets separate variables that must be in the same net. Additionally, brackets create boxes. When brackets are used, _all_ remaining free ports in the net must be used in the simplicity operation.
For example:
```
# This requires that `a` and `b` are part of different nets.
Times(a)(b) = c
# This requires that `a` and `b` are part of the same net.
Par(a b) = c
# This requires that `a` is in a different net from `b` and `c`, and that `b` and `c` belong to the same net.
# Additionally, there must be no other variables which are also free ports of the net that `b` and `c` are a part of.
Exp1(a)[b c]
```

Variable usages with the same name are linked together in pairs depending on the order they occur.

The set of wires in the definition must be linked to something before the end of the definition. Additionally, they must be free ports of the same net. (A definition must not be a set of disconnected nets)

In the `tests` folder, there are many examples of `linear-nets` programs.

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
# TODO: Formulate All and Any rules.
```

## References / Further reading

[^1]: Jean-Yves Girard (1987), _Linear Logic_.
[^2]: Yves Lafont (1990), _Interaction Nets_.
[^3]: Yves Lafont (1997), _Interaction Combinators_.
