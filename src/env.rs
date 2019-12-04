use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::calculator::StackInfo;
use crate::calculator::{CalculateOption, FormulaCalc};
use crate::formula::BuildInFunctionType;
use crate::formula::FormulaNode;
use std::time::{Duration, Instant};

pub type EnvType = Rc<RefCell<Env>>;

pub struct DelayInfo {
    point: Instant,
    delay: Duration,
    delayed: bool,
}

impl DelayInfo {
    pub fn new(delay: Duration) -> Self {
        DelayInfo {
            point: Instant::now(),
            delayed: false,
            delay,
        }
    }

    pub fn check_delay(&mut self) -> bool {
        match self.delayed {
            true => true,
            false => {
                if Instant::now().duration_since(self.point.clone()) > self.delay {
                    self.delayed = true;
                }
                self.delayed
            }
        }
    }
}

pub enum EnvValueOption {
    Value {
        value: CalculateOption,
        delay: Option<DelayInfo>,
    },
    None,
}

#[derive(Clone)]
pub struct EnvValue {
    pub node: Rc<FormulaNode>,
    pub value: RefCell<CalculateOption>,
    pub hist_value: RefCell<Vec<CalculateOption>>,
}

impl FormulaCalc for EnvValue {
    fn calc(&self, env: &EnvType) -> CalculateOption {
        let value = self.node.calc(env);
        self.hist_value.borrow_mut().push(value.clone());
        value
    }
}

pub struct Env {
    prev: Option<Rc<RefCell<Env>>>,
    env: HashMap<String, EnvValue>,
    build_in_map: Option<HashMap<String, Rc<BuildInFunctionType>>>,
    stack: Rc<RefCell<Vec<StackInfo>>>,
}

impl Env {
    pub fn new() -> EnvType {
        Rc::new(RefCell::new(Env {
            prev: None,
            env: HashMap::new(),
            build_in_map: Some(HashMap::new()),
            stack: Rc::new(RefCell::new(Vec::new())),
        }))
    }

    pub fn extend(env: &EnvType) -> EnvType {
        Rc::new(RefCell::new(Env {
            prev: Some(Rc::clone(env)),
            env: HashMap::new(),
            build_in_map: None,
            stack: Rc::new(RefCell::new(Vec::new())),
        }))
    }

    pub fn extend_with_stack(env: &EnvType) -> EnvType {
        Rc::new(RefCell::new(Env {
            prev: Some(Rc::clone(env)),
            env: HashMap::new(),
            build_in_map: None,
            stack: Rc::clone(&RefCell::borrow(env).stack),
        }))
    }

    /// 从 Env 中获取 BuildIn 函数，只有最上级的 Env 才会保存注册的函数，其他的子集 Env build_in_map 中保存的都是 None
    pub fn get_build_in(&self, func_key: &str) -> Option<Rc<BuildInFunctionType>> {
        match self.build_in_map {
            Some(ref m) => m.get(func_key).cloned(),
            None => self
                .prev
                .as_ref()
                .map_or(None, |prev| RefCell::borrow(prev).get_build_in(func_key)),
        }
    }

    pub fn set_build_in(&mut self, func_key: &str, f: Rc<BuildInFunctionType>) {
        match self.build_in_map.as_mut() {
            Some(m) => {
                m.insert(func_key.to_string(), f.clone());

                self.set(
                    func_key,
                    Rc::new(FormulaNode::BuildInFunction {
                        func: func_key.to_string(),
                    }),
                );
            }
            None => (),
        }
    }

    /// 从 当前执行环境中根据变量名获取信息，获取到的结果可以是脚本允许的任意一种类型, 如变量，函数等
    /// 如果从当前层次的上下文中获取不到，则尝试从上级上下文中获取, 具体实现的能力体现为：获取变量优先从当前
    /// 作用域获取，如果没有则从上一级作用域获取，直到最后一级，也就是获取全局变量
    pub fn get(&self, key: &str) -> Option<Rc<FormulaNode>> {
        self.env
            .get(key)
            .map(|e| e.node.clone())
            .or_else(|| match self.prev {
                Some(ref prev) => prev.borrow_mut().get(key),
                None => None,
            })
    }

    /// 将 value 指定的信息保存到环境变量中
    pub fn set(&mut self, key: &str, value: Rc<FormulaNode>) -> Option<EnvValue> {
        self.env.insert(
            key.to_string(),
            EnvValue {
                hist_value: RefCell::new(Vec::new()),
                node: Rc::clone(&value),
                value: RefCell::new(CalculateOption::None),
            },
        )
    }

    /// 将 key 对应 FormulaNode 节点的当前计算结果保存到 Env 中, 并返回旧的计算结果
    pub fn set_node_value(&mut self, key: &str, value: CalculateOption) -> CalculateOption {
        match self.env.get(key) {
            Some(ev) => {
                let mut v = ev.value.borrow_mut();
                let olv_v = v.clone();
                *v = value;
                olv_v
            }
            None => match self.prev {
                Some(ref prev) => prev.borrow_mut().set_node_value(key, value),
                None => CalculateOption::None,
            },
        }
    }

    /// 用于保持向下兼容的函数，后续考虑移除
    pub fn insert(&mut self, key: &str, value: Rc<FormulaNode>) -> Option<EnvValue> {
        self.set(key, value)
    }

    /// 保存当前调用的堆栈信息
    pub fn set_stack(&self, op: &str, func: &str, args: Vec<Rc<FormulaNode>>) {
        self.stack.borrow_mut().push(StackInfo {
            op: op.to_string(),
            func: func.to_string(),
            args,
        })
    }

    /// 消费自身，得到该 env 调用的堆栈信息
    pub fn call_stack(&self) -> Vec<StackInfo> {
        self.stack.borrow().clone()
    }
}
