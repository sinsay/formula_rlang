use std::collections::HashMap;
use std::rc::Rc;
use crate::formula_scaner::*;

type Env = HashMap<String, Rc<FormulaNode>>;

pub trait FormulaCalc {
    fn calc(&self, e: &Env) -> f64;
}

impl FormulaCalc for FormulaNode {
    fn calc(&self, env: &HashMap<String, Rc<FormulaNode>>) -> f64 {
        match self {
            FormulaNode::Constant(f) => *f,
            FormulaNode::Variant(v) => env.get(v).expect("获取不到指定的变量名").calc(env),
            FormulaNode::Operator(op_node) => op_node.calc(env),
            FormulaNode::Formula { name: _, formula } => formula.calc(env),
            FormulaNode::Quote(formula) => formula.calc(env),
            FormulaNode::FunctionCall { name, args } => {
                let mut new_env = env.clone();

                let func = match env.get(name) {
                    Some(f) => f.clone(),
                    _ => panic!("从 env 中提取函数节点时出错, 该错误不可能发生")
                };

                let mut result = 0.0;
                match func.as_ref() {
                    FormulaNode::Function { name: _, args: args_define, expressions } => {
                        assert_eq!(args.len(), args_define.len(), "参数个数与函数定义的参数个数不匹配");

                        // 处理 Args, 将 Args 的值放入函数对应的参数名中
                        for (index, arg) in args.iter().enumerate() {
                            let arg_def: &Box<FormulaNode> = args_define.get(index).unwrap();
                            let arg_name = match arg_def.as_ref() {
                                FormulaNode::Variant(name) => name,
                                _ => panic!("获取变量名时出错")
                            };

                            let v = arg.calc(env);
                            new_env.insert(arg_name.clone(), Rc::new(FormulaNode::Constant(v)));
                        }

                        for exp in expressions {
                            result = exp.calc(&new_env);
                            match exp.as_ref() {
                                FormulaNode::Formula {name, formula: _} => {
                                    new_env.insert(name.clone(), Rc::new(FormulaNode::Constant(result)));
                                }
                                _ => ()
                            };
                        }
                    }

                    _ => panic!("从函数节点提取表达式时出错，该错误不可能发生")
                }

                return result;
            }
            _ => 0.0
        }
    }
}

fn bool_to_f64(b: bool) -> f64 {
    match b {
        true => 1.0,
        false => 0.0
    }
}

impl FormulaCalc for OperatorNode {
    fn calc(&self, env: &HashMap<String, Rc<FormulaNode>>) -> f64 {
        match self {
            OperatorNode::Plus { left, right } => left.calc(env) + right.calc(env),
            OperatorNode::Minus { left, right } => left.calc(env) - right.calc(env),
            OperatorNode::Divide { left, right } => left.calc(env) / right.calc(env),
            OperatorNode::Multiply { left, right } => left.calc(env) * right.calc(env),
            OperatorNode::Less { left, right } => bool_to_f64(left.calc(env) < right.calc(env)),
            OperatorNode::LessEqual { left, right } => bool_to_f64(left.calc(env) <= right.calc(env)),
            OperatorNode::Great { left, right } => bool_to_f64(left.calc(env) > right.calc(env)),
            OperatorNode::GreatEqual { left, right } => bool_to_f64(left.calc(env) >= right.calc(env)),
            OperatorNode::Equal { left, right } => bool_to_f64(left.calc(env) == right.calc(env)),
            OperatorNode::Not(node) => bool_to_f64(node.calc(env) == 0.0),
        }
    }
}

/// 计算 formula 的值, 相关联的参数及公式通过全局的 env 参数获取
pub fn calc(formula: FormulaNode, env: &HashMap<String, Rc<FormulaNode>>) -> f64 {
    formula.calc(env)
}