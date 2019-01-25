use std::collections::HashMap;
use std::rc::Rc;
use crate::formula::*;

type Env = HashMap<String, Rc<FormulaNode>>;

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
                        return CalculateOption::Err(format!("无法从执行环境中获取指定的变量名 {}", v));
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
                            format!("从执行环境中获取函数 {} 时出错，对应的函数不存在环境变量中，是否未定义该函数", name));
                    }
                };

                let mut result = CalculateOption::None;
                match func.as_ref() {
                    FormulaNode::Function { name: _, args: args_define, expressions } => {
                        if args.len() != args_define.len() {
                            return CalculateOption::Err(
                                format!("函数 {} 定义的参数个数为 {}, 与函数调用的参数个数{}不匹配", name, args_define.len(), args.len()));
                        }

                        // 处理 Args, 将 Args 的值放入函数对应的参数名中
                        for (index, arg) in args.iter().enumerate() {
                            let arg_def: &Box<FormulaNode> = args_define.get(index).unwrap();
                            let arg_name = match arg_def.as_ref() {
                                FormulaNode::Variant(name) => name,
                                _ => return CalculateOption::Err(format!("为函数 {} 获取执行变量时出错，错误变量位置为 {}", name, index))
                            };

                            let v = match arg.calc(env) {
                                CalculateOption::Bool(b) => FormulaNode::Bool(b),
                                CalculateOption::Num(f) => FormulaNode::Constant(f),
                                CalculateOption::Err(s) => return CalculateOption::Err(format!("为函数 {} 计算参数值时出错，错误信息为 {}", name, s)),
                                CalculateOption::None => return CalculateOption::Err(format!("为函数 {} 计算参数值时出错，错误信息为该参数返回结果为 None", name)),
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
                                        _ => return CalculateOption::Err(format!("计算函数体时出错！，后续增加具体的错误表达式"))
                                    }));
                                }
                                _ => ()
                            };
                        }
                    }

                    _ => panic!("从函数节点提取表达式时出错，该错误不可能发生")
                }

                return result;
            }
            _ => CalculateOption::Err(format!("无法计算该表达式，格式出错？"))
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
                    _ => return CalculateOption::Err(format!("尝试使用加法来计算非数值类型"))
                }
            }
            OperatorNode::Minus { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Num(l - r),
                    _ => return CalculateOption::Err(format!("尝试使用减法来计算非数值类型"))
                }
            }
            OperatorNode::Divide { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Num(l / r),
                    _ => return CalculateOption::Err(format!("尝试使用除法来计算非数值类型"))
                }
            }
            OperatorNode::Multiply { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Num(l * r),
                    _ => return CalculateOption::Err(format!("尝试使用乘法来计算非数值类型"))
                }
            }
            OperatorNode::Less { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Bool(l < r),
                    _ => return CalculateOption::Err(format!("尝试用 < 比较两个非数值类型"))
                }
            }
            OperatorNode::LessEqual { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Bool(l <= r),
                    _ => return CalculateOption::Err(format!("尝试用 <= 比较两个非数值类型"))
                }
            }
            OperatorNode::Great { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Bool(l > r),
                    _ => return CalculateOption::Err(format!("尝试用 > 比较两个非数值类型"))
                }
            }
            OperatorNode::GreatEqual { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Bool(l >= r),
                    _ => return CalculateOption::Err(format!("尝试用 >= 比较两个非数值类型"))
                }
            }
            OperatorNode::Equal { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => return CalculateOption::Bool(l == r),
                    _ => return CalculateOption::Err(format!("尝试用 == 比较两个非数值类型"))
                }
            }
            OperatorNode::Not(node) => {
                let node = node.calc(env);
                match node {
                    CalculateOption::Bool(b) => return CalculateOption::Bool(!b),
                    _ => return CalculateOption::Err(format!("尝试对非逻辑结果取反"))
                }
            }
            OperatorNode::And { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Bool(l), CalculateOption::Bool(r)) => return CalculateOption::Bool(l && r),
                    _ => return CalculateOption::Err(format!("尝试对两个非数值类型使用逻辑与操作"))
                }
            }
            OperatorNode::Or { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Bool(l), CalculateOption::Bool(r)) => return CalculateOption::Bool(l || r),
                    _ => return CalculateOption::Err(format!("尝试对两个非数值类型使用逻辑或操作"))
                }
            }
        }
    }
}
