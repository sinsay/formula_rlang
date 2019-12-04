pub mod calculator;
pub mod env;
pub mod formula;
pub mod parser;

#[cfg(test)]
mod test {
    use crate::calculator::{CalculateOption, FormulaCalc};
    use crate::parser;

    #[test]
    fn test_parser() {
        let mut parser = parser::Parser::new();
        parser.parse("A := 1".to_string());
        parser.parse("B := 2".to_string());
        assert!(parser
            .calculate("A + B".to_string())
            .value
            .eq(&CalculateOption::Num(3.0)));
    }

    #[test]
    fn test_build_in() {
        let mut parser = parser::Parser::new();
        parser.parse("A := 1; B := 2;".to_string());

        parser.reg_build_in("Add", |c| {
            assert_eq!(c.args.len(), 2);
            match (
                c.args.get(0).unwrap().calc(&c.env),
                c.args.get(1).unwrap().calc(&c.env),
            ) {
                (CalculateOption::Num(f1), CalculateOption::Num(f2)) => {
                    return CalculateOption::Num(f1 + f2);
                }
                _ => {
                    panic!("Add 函数接收了错误的参数信息");
                }
            }
        });
        let result = parser.calculate("Add(A, B)".to_string());
        assert_eq!(result.value, CalculateOption::Num(4.0));
    }

    #[test]
    fn test_delay() {}
}
