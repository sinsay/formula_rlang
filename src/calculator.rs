use std::collections::HashMap;
use std::rc::Rc;
use crate::formula::*;

type Env = HashMap<String, Rc<FormulaNode>>;

/// The trait for every formula node
pub trait FormulaCalc {
    fn calc(&self, e: &Env) -> CalculateOption;
}

impl FormulaCalc for FormulaNode {
    fn calc(&self, env: &HashMap<String, Rc<FormulaNode>>) -> CalculateOption {
        match self {
            FormulaNode::Constant(f) => CalculateOption::Num(*f),
            FormulaNode::Bool(b) => CalculateOption::Bool(*b),
            FormulaNode::Variant(v) => {
                match env.get(v) {
                    Some(v) => {
                        return v.as_ref().calc(env);
                    }
                    None => {
                        return CalculateOption::Err(format!("the variant named {} can not find in current environment", v));
                    }
                }
            }
            FormulaNode::Operator(op_node) => op_node.calc(env),
            FormulaNode::Formula { name: _, formula } => formula.calc(env),
            FormulaNode::Quote(formula) => formula.calc(env),
            FormulaNode::FunctionCall { name, args } => {
                let mut new_env = env.clone();

                let func = match env.get(name) {
                    Some(f) => f.clone(),
                    _ => {
                        return CalculateOption::Err(
                            format!("error occure while execute an function named {}, it's the function exists?", name));
                    }
                };

                let mut result = CalculateOption::None;
                match func.as_ref() {
                    FormulaNode::Function { name: _, args: args_define, expressions } => {
                        if args.len() != args_define.len() {
                            return CalculateOption::Err(
                                format!("the argument length for function {} doesn't match, expect {} but found {}.", name, args_define.len(), args.len()));
                        }

                        // 处理 Args, 将 Args 的值放入函数对应的参数名中
                        for (index, arg) in args.iter().enumerate() {
                            let arg_def: &Box<FormulaNode> = args_define.get(index).unwrap();
                            let arg_name = match arg_def.as_ref() {
                                FormulaNode::Variant(name) => name,
                                _ => return CalculateOption::Err(format!("error occure while extracting argument for function {}, the argument index is {}", name, index))
                            };

                            let v = match arg.calc(env) {
                                CalculateOption::Bool(b) => FormulaNode::Bool(b),
                                CalculateOption::Num(f) => FormulaNode::Constant(f),
                                CalculateOption::Err(s) => return CalculateOption::Err(format!("error occure while calculate the function {}'s argument, error message is:{} ", name, s)),
                                CalculateOption::None => return CalculateOption::Err(format!("error occure while calculate the function {}'s argument, error message is:None", name)),
                            };
                            new_env.insert(arg_name.clone(), Rc::new(v));
                        }

                        for exp in expressions {
                            result = exp.calc(&new_env);
                            match exp.as_ref() {
                                FormulaNode::Formula { name, formula: _ } => {
                                    new_env.insert(name.clone(), Rc::new(match result {
                                        CalculateOption::Num(f) => FormulaNode::Constant(f),
                                        CalculateOption::Bool(b) => FormulaNode::Bool(b),
                                        // TODO: add more exactly message
                                        _ => return CalculateOption::Err(format!("the body of function has syntax error!"))
                                    }));
                                }
                                _ => ()
                            };
                        }
                    }

                    _ => panic!("error occure while extracing expression, something mysterious happend?")
                }

                return result;
            }
            _ => CalculateOption::Err(format!("syntax error?"))
        }
    }
}

impl FormulaCalc for OperatorNode {
    fn calc(&self, env: &HashMap<String, Rc<FormulaNode>>) -> CalculateOption {
        match self {
            OperatorNode::Plus { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Num(l + r),
                    _ => return CalculateOption::Err(format!("adding with non number argument"))
                }
            }
            OperatorNode::Minus { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Num(l - r),
                    _ => return CalculateOption::Err(format!("minus with non number argument"))
                }
            }
            OperatorNode::Divide { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Num(l / r),
                    _ => return CalculateOption::Err(format!("divide with non number argument"))
                }
            }
            OperatorNode::Multiply { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Num(l * r),
                    _ => return CalculateOption::Err(format!("multiply with non number argument"))
                }
            }
            OperatorNode::Less { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Bool(l < r),
                    _ => return CalculateOption::Err(format!("less operater only take number argument"))
                }
            }
            OperatorNode::LessEqual { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Bool(l <= r),
                    _ => return CalculateOption::Err(format!("less equal operater only take number argument"))
                }
            }
            OperatorNode::Great { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Bool(l > r),
                    _ => return CalculateOption::Err(format!("grater operator only take number argument"))
                }
            }
            OperatorNode::GreatEqual { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Bool(l >= r),
                    _ => return CalculateOption::Err(format!("grate equal operator only take number argumnet"))
                }
            }
            OperatorNode::Equal { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Bool(l == r),
                    _ => return CalculateOption::Err(format!("equal operator only take number argument"))
                }
            }
            OperatorNode::Not(node) => {
                let node = node.calc(env);
                match node {
                    CalculateOption::Bool(b) => return CalculateOption::Bool(!b),
                    _ => return CalculateOption::Err(format!("not operator only works with logical argument"))
                }
            }
            OperatorNode::And { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Bool(l), CalculateOption::Bool(r)) => return CalculateOption::Bool(l && r),
                    _ => return CalculateOption::Err(format!("and operator only works with logical argument"))
                }
            }
            OperatorNode::Or { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Bool(l), CalculateOption::Bool(r)) => return CalculateOption::Bool(l || r),
                    _ => return CalculateOption::Err(format!("or operator only works with logical argumnet"))
                }
            }
        }
    }
}
