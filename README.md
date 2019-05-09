

# Overview

formula scaner can parse an expression to an syntax tree, and then you can execute it whenever you want.
it keep's an environment for every expression, so you can use it as as an function, but dynamic!

for now it support operator below:

- mathematical
  - `+、-、*、/`
- logical
  - `>、>=、<、<=、!=、=`
- priority with brace
- define an variant
- define function
- call an function

## syntax
Formula syntax:
    Variant: String,
    Constant: Number,
    Value: Variant, Constant, Formula
    Name: Constant := Formula
    BinaryOp: +, -, *, /
    UnaryOp: ^
    LogicOp: >, >=, <, <=, !=
    Function Definition: Variant(Variant, ...) {{ Exp; ... }}
    Function Call: Variant(Variant|Constant, ...)
    Exp: UnaryOp Value
         Value BinaryOp Value
         (Exp)

## example
A := 1
B := 2
C := !(A + 3) * B
F(a, b) { 
    c :=  a + b; 
    c * 2  // the last expression means return value, it ends without semicolon!
}
