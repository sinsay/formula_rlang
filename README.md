

# Overview

formula scaner can parse an expression to an syntax tree, and then you can execute it whenever you want.
it keep's an environment for every expression, so you can use it as as an calculator or dynamic function tool, 
withh dynamic ability!

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
```

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
         
```         

## example

the script demo:

```

# define variable
A := 1
B := 2
C := !(A + 3) * B

# logic oprator
X := A + B
X := A - B
X := A * B
X := A / B
X := A > B
X := A >= B
X := A < B
X := A <= B
X := !(A > B)
X := (A > B) || (A == B)
X := (A < B) && (B < C)

# define function 
F(a, b) { 
    c :=  a + b; 
    c * 2  // the last expression means return value, it ends without semicolon!
}

# function as argument
func_as_arg(f1, arg1, arg2) {
  f1(arg1, arg2)
}

# call function
F(1, 2)
F(A, B)
F(1, B)

func_as_arg(F, 1, 2)
func_as_arg(F, A, B)

```

and the script environment api

```rust
// init an parser
let mut p = formula_parser::parser::Parser::new();

// parse an expression, return the node or the full syntax tree
// FromulaNode::Variant { name: 'A', node: NumericNode { value: 1} }
let parsed_node = p.parse("A := 1".to_string()); 

// calculate the expression
let calc_result = p.calculate("A".to_string());  // CalculateResult { value: 1 }

// get the calculate tree
let mut converter = syntax_tree::SyntaxConverter::new(parser);
let node = p.parse("A > B || ((A > C) && (C < D))".to_string());
converter.with_calculate();
let tree = converter.convert_from(&node);

// got the full path of calculate procedure
//   Result = Or(true)  -> (A > B) == True
//                      -> And(False) -> (A > C) == False
//                                    -> (C < D) == True
//

```
