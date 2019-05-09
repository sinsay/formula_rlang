use std::rc::Rc;
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use crate::calculator::FormulaCalc;

/// The formula node on the expression
#[derive(Debug)]
pub enum FormulaNode {
    Variant(String),
    Constant(f64),
    Bool(bool),
    Operator(Box<OperatorNode>),
    FunctionCall { name: String, args: Vec<Box<FormulaNode>> },
    Function { name: String, args: Vec<Box<FormulaNode>>, expressions: Vec<Box<FormulaNode>> },
    Arg { name: String, value: Box<FormulaNode> },
    Formula { name: String, formula: Rc<FormulaNode> },
    UnKnow(String),
    Quote(Box<FormulaNode>),
    None,
}

#[derive(Debug)]
/// The logical operator definition
pub enum OperatorNode {
    Plus { left: Box<FormulaNode>, right: Box<FormulaNode> },
    Minus { left: Box<FormulaNode>, right: Box<FormulaNode> },
    Divide { left: Box<FormulaNode>, right: Box<FormulaNode> },
    Multiply { left: Box<FormulaNode>, right: Box<FormulaNode> },
    Less { left: Box<FormulaNode>, right: Box<FormulaNode> },
    LessEqual { left: Box<FormulaNode>, right: Box<FormulaNode> },
    Great { left: Box<FormulaNode>, right: Box<FormulaNode> },
    GreatEqual { left: Box<FormulaNode>, right: Box<FormulaNode> },
    Equal { left: Box<FormulaNode>, right: Box<FormulaNode> },
    Not(Box<FormulaNode>),

    And { left: Box<FormulaNode>, right: Box<FormulaNode> },
    Or { left: Box<FormulaNode>, right: Box<FormulaNode> },
}

#[derive(Debug)]
/// the calculation result
pub enum CalculateOption {
    Bool(bool),
    Num(f64),
    Err(String),
    None,
}

/// expression parser, it keeps an env info for execution
pub struct Parser {
    env: HashMap<String, Rc<FormulaNode>>
}


impl Parser {
    // constructor
    pub fn new() -> Self {
        Self { env: HashMap::new() }
    }

    /// parse an expression with text
    pub fn parse(&mut self, formula: String) -> Rc<FormulaNode> {
        let mut iter = formula.chars().peekable();
        skip_space(&mut iter);
        if let None = iter.peek() {
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
                FormulaNode::UnKnow(msg) => return Rc::new(FormulaNode::UnKnow(msg.clone())),
                _ => ()
            };
        }
        node
    }

    /// parse an calculate an expression text
    pub fn calculate(&mut self, formula: String) -> CalculateOption {
        let node = self.parse(formula);
        node.as_ref().calc(&self.env)
    }
}

fn parse_formula(formula: String) -> FormulaNode {
    let mut iter = formula.chars().peekable();
    skip_space(&mut iter);

    let mut node = FormulaNode::None;
    while iter.peek().is_some() {
        node = scan_node(&mut iter, false);
    }
    node
}

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

/// find and formula node
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
            ':' => {
                return scan_naming_node(iter, node);
            }
            '^' => {
                iter.next();
                let next_node = scan_node(iter, true);
                node = Some(FormulaNode::Operator(Box::new(
                    OperatorNode::Not(Box::new(next_node)))))
            }
            '(' | '[' => {
                node = Some(find_end_brace(iter));
            }
            'A'...'Z' | 'a'...'z' => {
                let var_node = scan_variant(iter);
                skip_space(iter);
                let n = match iter.peek() {
                    Some(c) if c == &'(' => {
                        let sub_formula = find_end_brace_without_parse(iter);
                        let args = scan_split_node(sub_formula, '(', ')', ',');
                        let func_node = match var_node {
                            FormulaNode::Variant(name) => FormulaNode::FunctionCall {
                                name,
                                args,
                            },
                            _ => panic!("wrong type with syntax!")
                        };

                        func_node
                    }
                    _ => var_node
                };

                skip_space(iter);
                let n = match iter.peek() {
                    Some(c) if c == &'{' => {
                        let sub_formula = find_end_brace_without_parse(iter);
                        let expressions = scan_split_node(sub_formula, '{', '}', ';');
                        match n {
                            FormulaNode::FunctionCall { name, args } => {
                                FormulaNode::Function { name, args, expressions }
                            }
                            _ => panic!("create an sub expression without FunctionCall")
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
                node = Some(scan_math(iter, node));
            }
            '>' | '<' | '=' => {
                node = Some(scan_compare(iter, node));
            }
            ';' => {
                iter.next();
            }
            '&' => {
                iter.next(); // skip first &
                match iter.peek() {
                    Some('&') => {
                        iter.next(); // skip second &
                        node = Some(scan_logic_and(iter, node));
                    }
                    _ => {
                        // maybe mathematical &, but not support yet
                        return FormulaNode::UnKnow(format!("missing the second & for logical operator &&"));
                    }
                }
            }
            '|' => {
                iter.next(); // skip first &
                match iter.peek() {
                    Some('|') => {
                        iter.next(); // skip second &
                        node = Some(scan_logic_or(iter, node));
                    }
                    _ => {
// maybe mathematical &, but not support yet
                        return FormulaNode::UnKnow(format!("missing the second | for the logical operator ||"));
                    }
                }
            }
            _ => panic!("got and unvalid syntax: {}！", iter.peek().unwrap())
        }

        if node.is_some() & &limit {
            return node.unwrap();
        }
    }

    node.unwrap()
}

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
                // recursive brace
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

fn find_end_brace(iter: &mut Peekable<Chars>) -> FormulaNode {
    let sub_formula = find_end_brace_without_parse(iter);
    if sub_formula.len() != 0 {
        parse_formula(sub_formula)
    } else {
        FormulaNode::None
    }
}

fn scan_naming_node(iter: &mut Peekable<Chars>, node: Option<FormulaNode>) -> FormulaNode {
    if node.is_none() {
        panic!("naming an node should use Variant := Expression");
    }

    iter.next();

    match iter.peek() {
        None => panic!("got nothing after an naming node??"),
        Some(c) if c != &'=' => panic!("missing the = operator after :"),
// c is =
        _ => iter.next()
    };

    match node.unwrap() {
        FormulaNode::Variant(name) => {
            if iter.peek().is_none() {
                panic!("got nothing after an naming node?? ");
            }
            let formula = Rc::new(scan_node(iter, false));
            return FormulaNode::Formula {
                name,
                formula,
            };
        }
        _ => panic!("naming an node error")
    }
}

/// 处理公式的数学运算
fn scan_math(iter: &mut Peekable<Chars>, left: Option<FormulaNode>) -> FormulaNode {
    if left.is_none() {
        panic!("binary operator with wrong argument");
    }

    let op = iter.next().unwrap();
    let left = Box::new(left.unwrap());
    let right = Box::new(scan_node(iter, false));
    let op_node = match op {
        '+' => OperatorNode::Plus { left, right },
        '-' => OperatorNode::Minus { left, right },
        '*' => OperatorNode::Multiply { left, right },
        '/' => OperatorNode::Divide { left, right },
        _ => panic!("got an unknow syntax")
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
        _ => panic!("scanning expression with an unvalid syntax")
    };

    FormulaNode::Operator(Box::new(op_node))
}

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

fn scan_logic_and(iter: &mut Peekable<Chars>, left: Option<FormulaNode>) -> FormulaNode {
    let right = scan_node(iter, true);
    let left = left.unwrap();
    return FormulaNode::Operator(Box::new(OperatorNode::And {
        left: Box::new(left),
        right: Box::new(right),
    }));
}

fn scan_logic_or(iter: &mut Peekable<Chars>, left: Option<FormulaNode>) -> FormulaNode {
    let right = scan_node(iter, true);
    let left = left.unwrap();
    return FormulaNode::Operator(Box::new(OperatorNode::Or {
        left: Box::new(left),
        right: Box::new(right),
    }));
}
