use crate::calculator::CalculateOption;
use crate::env::EnvType;
use std::rc::Rc;

/// 内建函数的函数上下文，保存了传递给该函数的所有参数信息, 以及当前执行函数的环境信息, 可修改当前环境变量，
pub struct FuncContext {
    pub args: Vec<Rc<FormulaNode>>,
    pub env: EnvType,
}

impl FuncContext {
    pub fn new(args: &Vec<Rc<FormulaNode>>, env: EnvType) -> Self {
        FuncContext {
            args: args.iter().map(|f| Rc::clone(f)).collect(),
            env,
        }
    }
}

/// 内建函数的声明
pub type BuildInFunctionType = dyn Fn(&FuncContext) -> CalculateOption;

/// 解析公式的节点类型，可能有变量、常量、操作符、嵌套的公式类型等
/// 2019-10-20 加入自定义函数，可以将 Rust 的函数注册到脚本中
#[derive(Debug, Clone)]
pub enum FormulaNode {
    /// 变量节点，可以是定义变量，也可能是引用变量,
    /// 变量可用于所有的计算场景，及作为函数的参数及返回值
    Variant(String),
    /// 常量节点
    /// 定义了在表达式中固定的值
    Constant(f64),
    /// 布尔值节点
    /// 定义逻辑计算的结果
    Bool(bool),
    /// 操作符节点，定义了常用的数学及逻辑操作符
    Operator(Box<OperatorNode>),
    /// 函数调用
    /// 用来描述当前需要调用的函数信息，包括 @param name 函数名， @args 调用该函数所传递的参数
    FunctionCall {
        name: String,
        args: Vec<Rc<FormulaNode>>,
    },
    /// 函数定义
    /// 定义了函数的 @name 名称， @args 函数的参数信息，以及 @expressions 函数体，
    /// 函数体由一系列的表达式组成，表达式可以是任意的表达式节点
    Function {
        name: String,
        args: Vec<Rc<FormulaNode>>,
        expressions: Vec<Rc<FormulaNode>>,
    },

    /// 内置函数，通过从 Env 注册，可以通过该接口为脚本引擎实现各种不同的基础功能,
    BuildInFunction {
        func: String, // 保存的是该函数的全局 ID, 后续可通过该 ID 获取函数体
    },

    /// 函数的参数定义
    /// 其中包括了该参数的名称以及该参数的值
    Arg {
        name: String,
        value: Box<FormulaNode>,
    },
    /// 表达式节点，由 FormulaNode 中其他类型的节点组成
    Formula {
        name: String,
        formula: Rc<FormulaNode>,
    },
    /// 未知节点，说明表达式出错
    UnKnow(String),
    /// 括号节点，用来明确表示表达式的优先级
    Quote(Box<FormulaNode>),
    None,
}

#[derive(Debug, Clone)]
/// 数学及逻辑操作符节点,
/// 其中包括了简单的算术操作：加减乘除，及逻辑操作：大于，大于等于，小于，小于等于，等于，不等于, 及取反
pub enum OperatorNode {
    /// 加法操作节点
    Plus {
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
    /// 减法操作节点
    Minus {
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
    /// 除法操作节点
    Divide {
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
    /// 乘法操作节点
    Multiply {
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
    /// 小于操作节点
    Less {
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
    /// 小于等于操作节点
    LessEqual {
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
    /// 大于操作节点
    Great {
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
    /// 大于等于操作节点
    GreatEqual {
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
    /// 等于操作节点
    Equal {
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
    /// 取反操作节点
    Not(Box<FormulaNode>),

    /// 逻辑与操作
    And {
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
    /// 逻辑或操作
    Or {
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
}
