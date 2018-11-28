use std::rc::Rc;
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use crate::formula_calculator::FormulaCalc;

/// 解析公式的节点类型，可能有变量、常量、操作符、嵌套的公式类型等
#[derive(Debug)]
pub enum FormulaNode {
    /// 变量节点，可以是定义变量，也可能是引用变量,
    /// 变量可用于所有的计算场景，及作为函数的参数及返回值
    Variant(String),
    /// 常量节点
    /// 定义了在表达式中固定的值
    Constant(f64),
    /// 操作符节点，定义了常用的数学及逻辑操作符
    Operator(Box<OperatorNode>),
    /// 函数调用
    /// 用来描述当前需要调用的函数信息，包括 @name 函数名， @args 调用该函数所传递的参数
    FunctionCall { name: String, args: Vec<Box<FormulaNode>> },
    /// 函数定义
    /// 定义了函数的 @name 名称， @args 函数的参数信息，以及 @expressions 函数体，
    /// 函数体由一系列的表达式组成，表达式可以是任意的表达式节点
    Function { name: String, args: Vec<Box<FormulaNode>>, expressions: Vec<Box<FormulaNode>> },
    /// 函数的参数定义
    /// 其中包括了该参数的名称以及该参数的值
    Arg { name: String, value: Box<FormulaNode> },
    /// 表达式节点，由 FormulaNode 中其他类型的节点组成
    Formula { name: String, formula: Rc<FormulaNode> },
    /// 未知节点，说明表达式出错
    UnKnow(String),
    /// 括号节点，用来明确表示表达式的优先级
    Quote(Box<FormulaNode>),
    None,
}

#[derive(Debug)]
/// 数学及逻辑操作符节点,
/// 其中包括了简单的算术操作：加减乘除，及逻辑操作：大于，大于等于，小于，小于等于，等于，不等于, 及取反
pub enum OperatorNode {
    /// 加法操作节点
    Plus { left: Box<FormulaNode>, right: Box<FormulaNode> },
    /// 减法操作节点
    Minus { left: Box<FormulaNode>, right: Box<FormulaNode> },
    /// 除法操作节点
    Divide { left: Box<FormulaNode>, right: Box<FormulaNode> },
    /// 乘法操作节点
    Multiply { left: Box<FormulaNode>, right: Box<FormulaNode> },
    /// 小于操作节点
    Less { left: Box<FormulaNode>, right: Box<FormulaNode> },
    /// 小于等于操作节点
    LessEqual { left: Box<FormulaNode>, right: Box<FormulaNode> },
    /// 大于操作节点
    Great { left: Box<FormulaNode>, right: Box<FormulaNode> },
    /// 大于等于操作节点
    GreatEqual { left: Box<FormulaNode>, right: Box<FormulaNode> },
    /// 等于操作节点
    Equal { left: Box<FormulaNode>, right: Box<FormulaNode> },
    /// 取反操作节点
    Not(Box<FormulaNode>),
}

/// 表达式解析器
/// 表达式解析器内部包含一个环境变量，用于记录该解析器中所产生的各种表达式节点，
/// 已记录的表达式节点可以在其他的表达式中引用
pub struct Parser {
    env: HashMap<String, Rc<FormulaNode>>
}

impl Parser {
    /// 创建一个新的表达式解析器
    pub fn new() -> Self {
        Self { env: HashMap::new() }
    }

    /// 解析 formula 对应的表达式，并返回其解析后的表达式节点，该节点可直接调用 calc
    /// 用来计算表达式的结果，但需要自己提供执行环境 env, 所以一般是交由 parser 的
    /// calculate 方法来触发表达式的计算
    pub fn parse(&mut self, formula: String) -> Rc<FormulaNode> {
        let mut iter = formula.chars().peekable();
        skip_space(&mut iter);
        if let None = iter.peek() {
            println!("需要解析的表达式为空!");
            return Rc::new(FormulaNode::None);
        }

        let mut node = Rc::new(FormulaNode::None);
        while iter.peek().is_some() {
            let inner_node = scan_node(&mut iter, false);

            node = Rc::new(inner_node);
            match node.as_ref() {
                FormulaNode::Function { name, args: _, expressions: _ } => {
                    self.env.insert(name.clone(), node.clone());
                }
                FormulaNode::Formula { name, formula: _ } => {
                    self.env.insert(name.clone(), node.clone());
                }
                _ => ()
            };
        }
        node
    }

    /// 执行 formula 表达式，表达式所需的各种变量及函数需要在执行前 parse,
    /// 以加入环境变量
    pub fn calculate(&mut self, formula: String) -> f64 {
        let node = self.parse(formula);
        node.as_ref().calc(&self.env)
    }
}

/// 解析 formula，并返回该公式的预解析结果，即将公式解析为各种算子
/// 同时会将具有名称的节点加入 env
fn parse_formula(formula: String) -> FormulaNode {
    let mut iter = formula.chars().peekable();
    skip_space(&mut iter);

    let mut node = FormulaNode::None;
    while iter.peek().is_some() {
        node = scan_node(&mut iter, false);
    }
    node
}

/// 删除无用的空格
fn skip_space(iter: &mut Peekable<Chars>) {
    loop {
        match iter.peek() {
            Some(c) => {
                match c {
                    ' ' | '\r' | '\n' => {
                        iter.next();
                    }
                    _ => break
                }
            }
            None => {
                break;
            }
        }
    }
}

/// 扫描当前公式，尝试得到一个节点
fn scan_node(iter: &mut Peekable<Chars>, limit: bool) -> FormulaNode {
    if iter.peek().is_none() {
        return FormulaNode::None;
    }

    let mut node = None;
    while iter.peek().is_some() {
        skip_space(iter);

        if iter.peek().is_none() {
            break;
        }

        match iter.peek().unwrap() {
            // 公式定义: 命名
            ':' => {
                return scan_naming_node(iter, node);
            }
            // 处理一元计算
            '^' => {
                // 处理一元计算节点，一元计算节点需要用到该节点之后的后置节点
                iter.next();
                let next_node = scan_node(iter, true);
                node = Some(FormulaNode::Operator(Box::new(
                    OperatorNode::Not(Box::new(next_node)))))
            }
            '(' | '[' => {
                // 开始处理嵌套的 Brace
                node = Some(find_end_brace(iter));
            }
            'A'...'Z' | 'a'...'z' => {
                // 可能是 Variant 也可能是 Formula
                let var_node = scan_variant(iter);

                // 如果一个变量后续是括号，则说明它是一个函数
                skip_space(iter);
                let n = match iter.peek() {
                    Some(c) if c == &'(' => {
                        let sub_formula = find_end_brace_without_parse(iter);
                        // 处理函数的参数
                        let args = scan_split_node(sub_formula, '(', ')', ',');
                        let func_node = match var_node {
                            FormulaNode::Variant(name) => FormulaNode::FunctionCall {
                                name,
                                args,
                            },
                            _ => panic!("当前节点类型错误，该错误不应发生！")
                        };

                        func_node
                    }
                    _ => var_node
                };

                // 检查是否函数定义, 如果是函数定义，则需要确认 args 中的元素必须都是 Variant 类型
                skip_space(iter);
                let n = match iter.peek() {
                    Some(c) if c == &'{' => {
                        let sub_formula = find_end_brace_without_parse(iter);
                        // 解析出函数体中的多个表达式，每个表达式之间使用 ; 进行分割
                        let expressions = scan_split_node(sub_formula, '{', '}', ';');
                        match n {
                            FormulaNode::FunctionCall { name, args } => {
                                FormulaNode::Function { name, args, expressions }
                            }
                            _ => panic!("当前节点类型不为 FunctionCall， 该错误不应发生")
                        }
                    }
                    _ => n
                };

                node = Some(n);
            }
            '0'...'9' | '.' => {
                node = Some(scan_const(iter));
            }
            '+' | '-' | '*' | '/' => {
                // 处理二元计算节点，计算节点的话可能会需要用到前置节点以及后置节点
                node = Some(scan_math(iter, node));
            }
            '>' | '<' | '=' => {
                node = Some(scan_compare(iter, node));
            }
            ';' => {
                iter.next();
                break;
            }
            _ => panic!("扫描公式时遇到非法符号: {}！", iter.peek().unwrap())
        }

        if node.is_some() && limit {
            return node.unwrap();
        }
    }

    node.unwrap()
}

/// 获取括号中的表达式，支持获取嵌套的表达式
fn find_end_brace_without_parse(iter: &mut Peekable<Chars>) -> String {
    let mut sub_formula = String::new();
    let mut brace_count = 1;

    iter.next();  // skip the first brace
    while let Some(c) = iter.next() {
        match c {
            ')' | ']' | '}' => {
                brace_count -= 1;
                if brace_count != 0 {
                    sub_formula.push(c);
                    continue;
                }
                return sub_formula;
            }
            '(' | '[' | '{' => {
                // 找到了嵌套的 Quote
                brace_count += 1;
                sub_formula.push(c);
            }
            _ => {
                sub_formula.push(c);
            }
        }
    }

    sub_formula
}

/// 处理括号中的表达式, 并将表达式的字符串解析为 表达式节点
fn find_end_brace(iter: &mut Peekable<Chars>) -> FormulaNode {
    let sub_formula = find_end_brace_without_parse(iter);
    if sub_formula.len() != 0 {
        parse_formula(sub_formula)
    } else {
        FormulaNode::None
    }
}

/// 处理公式命名
fn scan_naming_node(iter: &mut Peekable<Chars>, node: Option<FormulaNode>) -> FormulaNode {
    // 处理公式的命名, 前置节点应为一个 Variant 节点
    if node.is_none() {
        panic!("公式的格式出错，命名公式的格式为 公式名 := 表达式");
    }

    iter.next();

    match iter.peek() {
        None => panic!("公式格式出错，等号后没有后续的表达式"),
        Some(c) if c != &'=' => panic!("公式格式出错，命名公式时缺少了 : 之后的 = 号"),
        // c is =
        _ => iter.next()
    };

    match node.unwrap() {
        FormulaNode::Variant(name) => {
            if iter.peek().is_none() {
                panic!("公式格式出错，公式名称之后没有任何表达式");
            }
            let formula = Rc::new(scan_node(iter, false));
            return FormulaNode::Formula {
                name,
                formula,
            };
        }
        _ => panic!("公式的格式出错，命名的节点应为 Variant 类型，命名公式的格式为 公式名 = 表达式")
    }
}

/// 处理公式的数学运算
fn scan_math(iter: &mut Peekable<Chars>, left: Option<FormulaNode>) -> FormulaNode {
    if left.is_none() {
        panic!("公式的格式错误，二元操作符前没有合法的计算节点");
    }

    let op = iter.next().unwrap();
    let left = Box::new(left.unwrap());
    let right = Box::new(scan_node(iter, false));
    let op_node = match op {
        '+' => OperatorNode::Plus { left, right },
        '-' => OperatorNode::Minus { left, right },
        '*' => OperatorNode::Multiply { left, right },
        '/' => OperatorNode::Divide { left, right },
        _ => panic!("扫描公式时遇到未知的操作符")
    };

    FormulaNode::Operator(Box::new(op_node))
}

fn scan_compare(iter: &mut Peekable<Chars>, node: Option<FormulaNode>) -> FormulaNode {
    let op = iter.next().unwrap();
    let next_op = *iter.peek().unwrap();
    if next_op == '=' {
        iter.next();
    }
    skip_space(iter);

    let left = Box::new(node.unwrap());
    let right = Box::new(scan_node(iter, false));
    let op_node = match op {
        '>' => match next_op {
            '=' => OperatorNode::GreatEqual { left, right },
            _ => OperatorNode::Great { left, right }
        }
        '<' => match next_op {
            '=' => OperatorNode::LessEqual { left, right },
            _ => OperatorNode::Less { left, right }
        }
        '=' => OperatorNode::Equal { left, right },
        _ => panic!("扫描公式时遇到未知的操作符")
    };

    FormulaNode::Operator(Box::new(op_node))
}

/// 处理公式的变量
fn scan_variant(iter: &mut Peekable<Chars>) -> FormulaNode {
    let mut node = String::new();
    while let Some(c) = iter.peek() {
        match c {
            'A'...'Z' | 'a'...'z' | ' ' | '0'...'9' => {
                node.push(*c);
                iter.next();
            }
            _ => break
        }
    }

    if node.len() == 0 {
        return FormulaNode::None;
    }


    FormulaNode::Variant(node.trim().to_string())
}

/// 处理公式的常量
fn scan_const(iter: &mut Peekable<Chars>) -> FormulaNode {
    let mut node = String::new();

    while let Some(c) = iter.peek() {
        match c {
            '0'...'9' | '.' => {
                node.push(*c);
                iter.next();
            }
            _ => {
                break;
            }
        }
    }
    if node.ends_with('.') {
        node.pop();
    }

    if node.len() == 0 {
        return FormulaNode::None;
    }
    FormulaNode::Constant(node.parse::<f64>().unwrap())
}

/// 处理函数的参数
/// 通过扫描字符串并根据 , 分割，把分割后的字符串再次处理为 公式的节点类型
fn scan_split_node(formula_str: String, begin_brace: char, end_brace: char, splitter: char) -> Vec<Box<FormulaNode>> {
    let mut args = vec![];
    let mut arg = String::new();
    let mut iter = formula_str.chars();
    let mut brace_count = 0;
    while let Some(c) = iter.next() {
        match c {
            n if n == begin_brace => brace_count += 1,
            n if n == end_brace => brace_count -= 1,
            n if n == splitter => {
                if brace_count == 0 {
                    let formula = parse_formula(arg.clone());
                    args.push(Box::new(formula));
                    arg.clear();
                }
            }
            _ => arg.push(c)
        }
    }

    if arg.len() != 0 {
        let formula = parse_formula(arg);
        args.push(Box::new(formula));
    }

    args
}
