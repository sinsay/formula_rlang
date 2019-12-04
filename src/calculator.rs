use crate::env::{Env, EnvType};
use crate::formula::*;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::rc::Rc;

pub trait FormulaCalc {
    fn calc(&self, e: &EnvType) -> CalculateOption;
}

impl FormulaCalc for FormulaNode {
    fn calc(&self, env: &EnvType) -> CalculateOption {
        match self {
            FormulaNode::Constant(f) => CalculateOption::Num(*f),
            FormulaNode::Bool(b) => CalculateOption::Bool(*b),
            FormulaNode::Variant(v) => match RefCell::borrow(&env).get(v) {
                Some(v) => {
                    return v.calc(env);
                }
                None => {
                    return CalculateOption::Err(format!("无法从执行环境中获取指定的变量名 {}", v));
                }
            },
            FormulaNode::Operator(op_node) => op_node.calc(env),
            FormulaNode::Formula { name: _, formula } => formula.calc(env),
            FormulaNode::Quote(formula) => formula.calc(env),
            FormulaNode::Function {
                name: _,
                args: _,
                expressions: _,
            } => CalculateOption::Func,
            FormulaNode::FunctionCall { name, args } => {
                let new_env = Env::extend_with_stack(env);

                // record the stack
                RefCell::borrow(&new_env).set_stack("FunctionCall", name, args.clone());

                let func = match RefCell::borrow(&new_env).get(name) {
                    Some(f) => f.clone(),
                    _ => {
                        return CalculateOption::Err(
                            format!("从执行环境中获取函数 {} 时出错，对应的函数不存在环境变量中，是否未定义该函数", name));
                    }
                };

                let mut result = CalculateOption::None;
                match func.as_ref() {
                    FormulaNode::Function {
                        name,
                        args: args_define,
                        expressions,
                    } => {
                        if args.len() != args_define.len() {
                            return CalculateOption::Err(format!(
                                "函数 {} 定义的参数个数为 {}, 与函数调用的参数个数{}不匹配",
                                name,
                                args_define.len(),
                                args.len()
                            ));
                        }

                        // 处理 Args, 将 Args 的值放入函数对应的参数名中
                        for (index, arg) in args.iter().enumerate() {
                            let arg_def: Rc<FormulaNode> = args_define.get(index).cloned().unwrap();
                            let arg_name = match arg_def.as_ref() {
                                FormulaNode::Variant(name) => name,
                                _ => {
                                    return CalculateOption::Err(format!(
                                        "为函数 {} 获取执行变量时出错，错误变量位置为 {}",
                                        name, index
                                    ))
                                }
                            };

                            let v = match arg.calc(env) {
                                CalculateOption::Bool(b) => Rc::new(FormulaNode::Bool(b)),
                                CalculateOption::Num(f) => Rc::new(FormulaNode::Constant(f)),
                                CalculateOption::Func => {
                                    // 这是把函数当为参数传递的情形
                                    match arg.borrow() {
                                        FormulaNode::Variant(s) => {
                                            RefCell::borrow(&new_env).get(&s).expect(&format!("获取不到指定的变量 {}", s))
                                        }
                                        _ => return CalculateOption::Err(format!(
                                            "执行函数 {} 时出错，变量 {} 所绑定的函数 {:?} 不存在。",
                                            name,
                                            arg_name,
                                            arg
                                        ))
                                    }
                                }
                                CalculateOption::Err(s) => {
                                    return CalculateOption::Err(format!(
                                        "为函数 {} 计算参数值时出错，错误信息为 {}",
                                        name, s
                                    ))
                                }
                                CalculateOption::None => {
                                    return CalculateOption::Err(format!(
                                    "为函数 {} 计算参数值时出错，错误信息为该参数返回结果为 None",
                                    name
                                ))
                                }
                            };
                            new_env.borrow_mut().insert(&arg_name, v);
                        }

                        for exp in expressions {
                            result = exp.calc(&new_env);
                            match exp.as_ref() {
                                FormulaNode::Formula { name, formula: _ } => {
                                    new_env.borrow_mut().insert(
                                        &name,
                                        Rc::new(match result {
                                            CalculateOption::Num(f) => FormulaNode::Constant(f),
                                            CalculateOption::Bool(b) => FormulaNode::Bool(b),
                                            _ => {
                                                return CalculateOption::Err(format!(
                                                    "计算函数体时出错！，后续增加具体的错误表达式"
                                                ))
                                            }
                                        }),
                                    );
                                }
                                _ => (),
                            };
                        }
                    }
                    FormulaNode::BuildInFunction { func } => {
                        RefCell::borrow(&env).set_stack("BuildInFunction", func, args.clone());

                        match RefCell::borrow(&env).get_build_in(func) {
                            Some(f) => {
                                let context = FuncContext::new(args, Rc::clone(env));
                                result = f(&context);
                            }
                            None => {
                                return CalculateOption::Err(format!(
                                    "获取内建函数 {} 时出错，运行环境中不存在该函数",
                                    func
                                ))
                            }
                        }
                    }

                    _ => panic!("从函数节点提取表达式时出错，该错误不可能发生"),
                }

                return result;
            }
            _ => CalculateOption::Err(format!("无法计算该表达式，格式出错？")),
        }
    }
}

impl FormulaCalc for OperatorNode {
    fn calc(&self, env: &EnvType) -> CalculateOption {
        match self {
            OperatorNode::Plus { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => {
                        return CalculateOption::Num(l + r)
                    }
                    (CalculateOption::Err(e), _) => return CalculateOption::Err(e),
                    (_, CalculateOption::Err(e)) => return CalculateOption::Err(e),
                    _ => return CalculateOption::Err(format!("尝试使用加法来计算非数值类型")),
                }
            }
            OperatorNode::Minus { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => {
                        return CalculateOption::Num(l - r)
                    }
                    (CalculateOption::Err(e), _) => return CalculateOption::Err(e),
                    (_, CalculateOption::Err(e)) => return CalculateOption::Err(e),
                    _ => return CalculateOption::Err(format!("尝试使用减法来计算非数值类型")),
                }
            }
            OperatorNode::Divide { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => {
                        return CalculateOption::Num(l / r)
                    }
                    (CalculateOption::Err(e), _) => return CalculateOption::Err(e),
                    (_, CalculateOption::Err(e)) => return CalculateOption::Err(e),
                    _ => return CalculateOption::Err(format!("尝试使用除法来计算非数值类型")),
                }
            }
            OperatorNode::Multiply { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => {
                        return CalculateOption::Num(l * r)
                    }
                    (CalculateOption::Err(e), _) => return CalculateOption::Err(e),
                    (_, CalculateOption::Err(e)) => return CalculateOption::Err(e),
                    _ => return CalculateOption::Err(format!("尝试使用乘法来计算非数值类型")),
                }
            }
            OperatorNode::Less { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => {
                        return CalculateOption::Bool(l < r)
                    }
                    (CalculateOption::Err(e), _) => return CalculateOption::Err(e),
                    (_, CalculateOption::Err(e)) => return CalculateOption::Err(e),
                    _ => return CalculateOption::Err(format!("尝试用 < 比较两个非数值类型")),
                }
            }
            OperatorNode::LessEqual { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => {
                        return CalculateOption::Bool(l <= r)
                    }
                    (CalculateOption::Err(e), _) => return CalculateOption::Err(e),
                    (_, CalculateOption::Err(e)) => return CalculateOption::Err(e),
                    _ => return CalculateOption::Err(format!("尝试用 <= 比较两个非数值类型")),
                }
            }
            OperatorNode::Great { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => {
                        return CalculateOption::Bool(l > r)
                    }
                    (CalculateOption::Err(e), _) => return CalculateOption::Err(e),
                    (_, CalculateOption::Err(e)) => return CalculateOption::Err(e),
                    _ => return CalculateOption::Err(format!("尝试用 > 比较两个非数值类型")),
                }
            }
            OperatorNode::GreatEqual { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => {
                        return CalculateOption::Bool(l >= r)
                    }
                    (CalculateOption::Err(e), _) => return CalculateOption::Err(e),
                    (_, CalculateOption::Err(e)) => return CalculateOption::Err(e),
                    _ => return CalculateOption::Err(format!("尝试用 >= 比较两个非数值类型")),
                }
            }
            OperatorNode::Equal { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => {
                        return CalculateOption::Bool(l == r)
                    }
                    (CalculateOption::Err(e), _) => return CalculateOption::Err(e),
                    (_, CalculateOption::Err(e)) => return CalculateOption::Err(e),
                    _ => return CalculateOption::Err(format!("尝试用 == 比较两个非数值类型")),
                }
            }
            OperatorNode::Not(node) => {
                let node = node.calc(env);
                match node {
                    CalculateOption::Bool(b) => return CalculateOption::Bool(!b),
                    CalculateOption::Num(n) => return CalculateOption::Bool(n != 0.0),
                    CalculateOption::Err(e) => return CalculateOption::Err(e),
                    _ => return CalculateOption::Err(format!("尝试对非逻辑结果取反")),
                }
            }
            OperatorNode::And { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Bool(l), CalculateOption::Bool(r)) => {
                        return CalculateOption::Bool(l && r)
                    }
                    (CalculateOption::Bool(l), CalculateOption::Num(r)) => {
                        return match (l, r != 0.0) {
                            (true, _) => CalculateOption::Num(r),
                            (false, _) => CalculateOption::Bool(false),
                        }
                    }
                    (CalculateOption::Num(l), CalculateOption::Bool(r)) => {
                        return match (l != 0.0, r) {
                            (true, _) => CalculateOption::Bool(r),
                            (false, _) => CalculateOption::Num(l), // 0.0
                        };
                    }
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => {
                        return match (l != 0.0, r != 0.0) {
                            (true, _) => CalculateOption::Num(r),
                            (false, _) => CalculateOption::Num(l),
                        }
                    }
                    (CalculateOption::Err(e), _) => return CalculateOption::Err(e),
                    (_, CalculateOption::Err(e)) => return CalculateOption::Err(e),
                    _ => {
                        return CalculateOption::Err(format!("尝试对两个非数值类型使用逻辑与操作"))
                    }
                }
            }
            OperatorNode::Or { left, right } => {
                let left = left.calc(env);
                let right = right.calc(env);
                match (left, right) {
                    (CalculateOption::Bool(l), CalculateOption::Bool(r)) => {
                        return CalculateOption::Bool(l || r)
                    }
                    (CalculateOption::Bool(l), CalculateOption::Num(r)) => {
                        return match (l, r != 0.0) {
                            (true, _) => CalculateOption::Bool(l),
                            (false, _) => CalculateOption::Num(r),
                        }
                    }
                    (CalculateOption::Num(l), CalculateOption::Bool(r)) => {
                        return match (l != 0.0, r) {
                            (true, _) => CalculateOption::Num(l),
                            (false, _) => CalculateOption::Bool(r),
                        }
                    }
                    (CalculateOption::Num(l), CalculateOption::Num(r)) => {
                        return match (l != 0.0, r != 0.0) {
                            (true, _) => CalculateOption::Num(l),
                            (false, _) => CalculateOption::Num(r),
                        }
                    }
                    (CalculateOption::Err(e), _) => return CalculateOption::Err(e),
                    (_, CalculateOption::Err(e)) => return CalculateOption::Err(e),
                    _ => {
                        return CalculateOption::Err(format!("尝试对两个非数值类型使用逻辑或操作"))
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackInfo {
    /// 当前有保存信息的 op 有 FunctionCall 跟 BuildInFunction
    pub op: String,
    /// 调用的函数名称
    pub func: String,
    /// 调用函数所使用的参数
    pub args: Vec<Rc<FormulaNode>>,
}

/// 表达式计算的结果， value 保存了表达式计算的最终结果， more 保存了当前表达式中执行过程中的调用信息,
/// 调用的信息主要包括，当前操作名称，函数名、函数调用的参数
/// 这些调用信息一般只有自定义或内建函数才会保存，简单的 Num、Var 等操作都还没保存到其中
#[derive(Debug, Clone)]
pub struct CalculateResult {
    /// 本次计算的结果
    pub value: CalculateOption,
    /// 用于保存调用信息
    /// TODO: 暂时使用 hash map，如果需要完整的堆栈信息，则改为树
    pub more: Vec<StackInfo>,
}

#[derive(Debug, Clone)]
/// 公式计算的结果值
pub enum CalculateOption {
    Bool(bool),
    Num(f64),
    Err(String),
    /// 如果计算的结果是函数定义，说明要调用
    Func,
    /// None 表示该计算没有结果
    None,
}

impl CalculateOption {
    pub fn eq(&self, other: &CalculateOption) -> bool {
        use CalculateOption::*;
        match (self, other) {
            (Bool(a), Bool(b)) => a == b,
            (Num(f1), Num(f2)) => f1 == f2,
            (_, _) => false,
        }
    }
}

impl PartialEq for CalculateOption {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}
