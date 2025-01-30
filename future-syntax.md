# Hypothetical syntax for a linear logic based language
Context would get passed implicitly.

these are my personal notes

## Term operators:
Terms and types are distinguished by context.

```
a * b: Times
(*): One
a | b: Par
(|): False
+a: Right
a+: Left
a & b: With (context is implicit, fetched from `a`'s and `b`'s nets)
(&): True
!a: Exponential (context is implicit)
(?): Weakening
?b: Dereliction
a?b: Contraction
@x A: Forall
$x A: Exists
x: Variable
_: Hole
```
In practice, the hard-to-parse operators are `*|+&?`. They have a precedence which I've not decided yet.
Precedence (stronger-to-weaker): `*|&+!?@$)

Type syntax:

```
A * B: Times
(*): One
A | B: Par
(|): False
A + B: Plus
(+): Zero
A & B: With
(&): True
?A: Why not
!A: Of course
@x A: For all
w$x A: There is
~A: Inverse operator
_: Hole
```

```
function_composition = {
    (~c * ~e) | (e * ~d) | c | d
}
boolean_true = +*
boolean_false = *+
with =
```

```
with_with_context = {
  ~a, a & a
}
```
