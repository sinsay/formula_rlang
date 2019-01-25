use std::env as Env;

pub mod formula;
pub mod calculator;

use crate::formula::Parser;
//use crate::calculator::FormulaCalc;

fn main() {
    let args: Vec<String> = Env::args().collect();
    if args.len() == 1 {
        println!("Formula syntax:
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
        ");
        println!("Usage: {}  <filename> [--from_std]", args[0]);
        return;
    }

    let mut parser = Parser::new();

    if args[1] == "--from_std" {
        exec_cmd(&mut parser);
    } else {
        for arg in args.iter().skip(1) {
            std::fs::read_to_string(arg).and_then(|formula| {
                parser.calculate(formula.clone());
                println!("成功解析表达式 {}", formula);
                Ok(1)
            }).expect(&format!("处理表达式 {} 时出错", arg));
        }

        println!("进入交互式环境？(yes/no) (default: yes)");
        let mut get_into = String::new();
        std::io::stdin().read_line(&mut get_into).expect("读取进入交互式环境的命令出错");
        if get_into.trim() == "yes" || get_into.trim().len() == 0 {
            exec_cmd(&mut parser);
        }
    }
}

fn exec_cmd(parser: &mut Parser) {
    let mut lines = String::new();
    loop {
        println!("输入表达式:");
        loop {
            let mut formula = String::new();
            std::io::stdin().read_line(&mut formula).expect("从标准输入中读取数据时出错");
            if formula.trim().len() == 0 && lines.len() == 0 {
                println!("输入的表达式为空!");
                continue;
            }

            if formula.trim().len() == 0 {
                break;
            }

            lines.push_str(&formula);
            formula.clear();
        }

        println!("{:?}", parser.calculate(lines.clone()));
        lines.clear();
    }
}
