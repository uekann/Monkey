use crate::ast::*;
use crate::object::Object;
use anyhow::{bail, Result};

pub fn eval_program(program: Program) -> Result<Object> {
    let mut result = Object::Null;
    for statement in program.statements {
        result = eval_statement(statement)?;
    }
    Ok(result)
}

fn eval_statement(statement: Statement) -> Result<Object> {
    match statement {
        Statement::ExpressionStatement(expr) => {
            let val = eval_expression(expr)?;
            Ok(val)
        }
        Statement::BlockStatement { statements } => {
            let mut result = Object::Null;
            for statement in statements {
                result = eval_statement(statement)?;
            }
            Ok(result)
        }
        _ => Ok(Object::Null),
    }
}

fn eval_expression(expression: Expression) -> Result<Object> {
    match expression {
        Expression::IntegerLiteral(i) => Ok(Object::Integer(i)),
        Expression::Boolean(b) => Ok(Object::Boolean(b)),
        Expression::PrefixExpression { operator, right } => {
            let right = eval_expression(*right)?;
            match operator.as_str() {
                "!" => eval_bang_prefix_expression(right),
                "-" => eval_minus_prefix_operator_expression(right),
                _ => Ok(Object::Null),
            }
        }
        Expression::InfixExpression {
            left,
            operator,
            right,
        } => {
            let left = eval_expression(*left)?;
            let right = eval_expression(*right)?;
            eval_infix_expression(operator, left, right)
        }
        Expression::IfExpression {
            condition,
            consequence,
            alternative,
        } => {
            let condition = eval_expression(*condition)?.cast_to_boolean()?;
            debug_assert!(matches!(condition, Object::Boolean(_)));
            debug_assert!(matches!(*consequence, Statement::BlockStatement { .. }));
            if condition == Object::Boolean(true) {
                eval_statement(*consequence)
            } else if let Some(alt) = alternative {
                debug_assert!(matches!(*alt, Statement::BlockStatement { .. }));
                eval_statement(*alt)
            } else {
                Ok(Object::Null)
            }
        }
        _ => Ok(Object::Null),
    }
}

fn eval_bang_prefix_expression(right: Object) -> Result<Object> {
    match right {
        Object::Boolean(b) => Ok(Object::Boolean(!b)),
        _ => eval_bang_prefix_expression(right.cast_to_boolean()?),
    }
}

fn eval_minus_prefix_operator_expression(right: Object) -> Result<Object> {
    match right {
        Object::Integer(i) => Ok(Object::Integer(-i)),
        _ => bail!("cannot use '-' operator on {:?}", right),
    }
}

fn eval_infix_expression(operator: String, left: Object, right: Object) -> Result<Object> {
    match (left, right) {
        (Object::Integer(left), Object::Integer(right)) => match operator.as_str() {
            "+" => Ok(Object::Integer(left + right)),
            "-" => Ok(Object::Integer(left - right)),
            "*" => Ok(Object::Integer(left * right)),
            "/" => Ok(Object::Integer(left / right)),
            "<" => Ok(Object::Boolean(left < right)),
            ">" => Ok(Object::Boolean(left > right)),
            "==" => Ok(Object::Boolean(left == right)),
            "!=" => Ok(Object::Boolean(left != right)),
            _ => bail!("unknown operator: {} {} {}", left, operator, right),
        },
        (Object::Boolean(left), Object::Boolean(right)) => match operator.as_str() {
            "==" => Ok(Object::Boolean(left == right)),
            "!=" => Ok(Object::Boolean(left != right)),
            _ => bail!("unknown operator: {} {} {}", left, operator, right),
        },
        _ => bail!("type mismatch: {:?} {} {:?}", left, operator, right),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::object::Object;
    use crate::parser::Parser;

    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![
            ("5", Object::Integer(5)),
            ("10", Object::Integer(10)),
            ("5 + 5 + 5 + 5 - 10", Object::Integer(10)),
            ("2 * 2 * 2 * 2 * 2", Object::Integer(32)),
            ("-50 + 100 + -50", Object::Integer(0)),
            ("5 * 2 + 10", Object::Integer(20)),
            ("5 + 2 * 10", Object::Integer(25)),
            ("20 + 2 * -10", Object::Integer(0)),
            ("50 / 2 * 2 + 10", Object::Integer(60)),
            ("2 * (5 + 10)", Object::Integer(30)),
            ("3 * 3 * 3 + 10", Object::Integer(37)),
            ("3 * (3 * 3) + 10", Object::Integer(37)),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Object::Integer(50)),
        ];
        for (input, expected) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program().unwrap();
            let evaluated = eval_program(program).unwrap();
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = vec![
            ("true", Object::Boolean(true)),
            ("false", Object::Boolean(false)),
            ("1 < 2", Object::Boolean(true)),
            ("1 > 2", Object::Boolean(false)),
            ("1 < 1", Object::Boolean(false)),
            ("1 > 1", Object::Boolean(false)),
            ("1 == 1", Object::Boolean(true)),
            ("1 != 1", Object::Boolean(false)),
            ("1 == 2", Object::Boolean(false)),
            ("1 != 2", Object::Boolean(true)),
            ("true == true", Object::Boolean(true)),
            ("false == false", Object::Boolean(true)),
            ("true == false", Object::Boolean(false)),
            ("true != false", Object::Boolean(true)),
            ("false != true", Object::Boolean(true)),
            ("(1 < 2) == true", Object::Boolean(true)),
            ("(1 < 2) == false", Object::Boolean(false)),
            ("(1 > 2) == true", Object::Boolean(false)),
            ("(1 > 2) == false", Object::Boolean(true)),
        ];
        for (input, expected) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program().unwrap();
            let evaluated = eval_program(program).unwrap();
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_prefix_expression() {
        let tests = vec![
            ("!true", Object::Boolean(false)),
            ("!false", Object::Boolean(true)),
            ("!5", Object::Boolean(false)),
            ("!!true", Object::Boolean(true)),
            ("!!false", Object::Boolean(false)),
            ("-5", Object::Integer(-5)),
            ("-10", Object::Integer(-10)),
            ("--5", Object::Integer(5)),
            ("--10", Object::Integer(10)),
        ];
        for (input, expected) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program().unwrap();
            let evaluated = eval_program(program).unwrap();
            assert_eq!(evaluated, expected);
        }
    }

    #[test]
    fn test_if_else_expression() {
        let tests = vec![
            ("if (true) { 10 }", Object::Integer(10)),
            ("if (false) { 10 }", Object::Null),
            ("if (1) { 10 }", Object::Integer(10)),
            ("if (1 < 2) { 10 }", Object::Integer(10)),
            ("if (1 > 2) { 10 }", Object::Null),
            ("if (1 > 2) { 10 } else { 20 }", Object::Integer(20)),
            ("if (1 < 2) { 10 } else { 20 }", Object::Integer(10)),
        ];
        for (input, expected) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program().unwrap();
            let evaluated = eval_program(program).unwrap();
            assert_eq!(evaluated, expected);
        }
    }
}
