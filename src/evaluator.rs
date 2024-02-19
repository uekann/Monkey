use crate::ast::*;
use crate::object::{Environment, Object};
use anyhow::{bail, Result};

pub fn eval_program(program: Program) -> Result<Object> {
    let mut env = Environment::new();

    let mut result = Object::Null;
    for statement in program.statements {
        result = eval_statement(statement, &mut env)?;
        if let Object::ReturnValue(val) = result {
            return Ok(*val);
        }
    }
    Ok(result)
}

fn eval_statement(statement: Statement, env: &mut Environment) -> Result<Object> {
    match statement {
        Statement::ExpressionStatement(expr) => {
            let val = eval_expression(expr, env)?;
            Ok(val)
        }
        Statement::BlockStatement { statements } => {
            let mut result = Object::Null;
            for statement in statements {
                result = eval_statement(statement, env)?;
                if let Object::ReturnValue(_) = result {
                    return Ok(result);
                }
            }
            Ok(result)
        }
        Statement::ReturnStatement(expr) => {
            let val = eval_expression(expr, env)?;
            Ok(Object::ReturnValue(Box::new(val)))
        }
        Statement::LetStatement { name, value } => {
            let val = eval_expression(value, env)?;
            env.set(name, val);
            Ok(Object::Null)
        }
        _ => Ok(Object::Null),
    }
}

fn eval_expression(expression: Expression, env: &mut Environment) -> Result<Object> {
    match expression {
        Expression::IntegerLiteral(i) => Ok(Object::Integer(i)),
        Expression::Boolean(b) => Ok(Object::Boolean(b)),
        Expression::Identifier(name) => match env.get(&name) {
            Some(val) => Ok(val.clone()),
            None => bail!("identifier not found: {}", name),
        },
        Expression::PrefixExpression { operator, right } => {
            let right = eval_expression(*right, env)?;
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
            let left = eval_expression(*left, env)?;
            let right = eval_expression(*right, env)?;
            eval_infix_expression(operator, left, right)
        }
        Expression::IfExpression {
            condition,
            consequence,
            alternative,
        } => {
            let condition = eval_expression(*condition, env)?.cast_to_boolean()?;
            debug_assert!(matches!(condition, Object::Boolean(_)));
            debug_assert!(matches!(*consequence, Statement::BlockStatement { .. }));
            if condition == Object::Boolean(true) {
                eval_statement(*consequence, env)
            } else if let Some(alt) = alternative {
                debug_assert!(matches!(*alt, Statement::BlockStatement { .. }));
                eval_statement(*alt, env)
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
            _ => bail!(
                "unknown operator: {:?} {} {:?}",
                Object::Integer(left),
                operator,
                Object::Integer(right)
            ),
        },
        (Object::Boolean(left), Object::Boolean(right)) => match operator.as_str() {
            "==" => Ok(Object::Boolean(left == right)),
            "!=" => Ok(Object::Boolean(left != right)),
            _ => bail!(
                "unknown operator: {:?} {} {:?}",
                Object::Boolean(left),
                operator,
                Object::Boolean(right)
            ),
        },
        (Object::Null, Object::Null) => match operator.as_str() {
            "==" => Ok(Object::Boolean(true)),
            "!=" => Ok(Object::Boolean(false)),
            _ => bail!("unknown operator: Null {} Null", operator),
        },
        (left, right) => bail!("type mismatch: {:?} {} {:?}", left, operator, right),
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

    #[test]
    fn test_return_statement() {
        let tests = vec![
            ("return 10;", Object::Integer(10)),
            ("return 10; 9;", Object::Integer(10)),
            ("return 2 * 5; 9;", Object::Integer(10)),
            ("9; return 2 * 5; 9;", Object::Integer(10)),
            (
                "if (10 > 1) {
                    if (10 > 1) {
                        return 10;
                    }
                    return 1;
                }",
                Object::Integer(10),
            ),
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
    fn test_error_handling() {
        let tests = vec![
            ("5 + true;", "type mismatch: Integer(5) + Boolean(true)"),
            ("5 + true; 5;", "type mismatch: Integer(5) + Boolean(true)"),
            ("-true", "cannot use '-' operator on Boolean(true)"),
            (
                "true + false;",
                "unknown operator: Boolean(true) + Boolean(false)",
            ),
            (
                "5; true + false; 5",
                "unknown operator: Boolean(true) + Boolean(false)",
            ),
            (
                "if (10 > 1) { true + false; }",
                "unknown operator: Boolean(true) + Boolean(false)",
            ),
            (
                r#"
                if (10 > 1) {
                    if (10 > 1) {
                        return true + false;
                    }
                    return 1;
                }
                "#,
                "unknown operator: Boolean(true) + Boolean(false)",
            ),
            ("foobar", "identifier not found: foobar"),
        ];
        for (input, expected) in tests {
            let l = Lexer::new(input);
            let mut p = Parser::new(l);
            let program = p.parse_program().unwrap();
            let evaluated = eval_program(program);
            assert!(evaluated.is_err());
            assert_eq!(evaluated.err().unwrap().to_string(), expected);
        }
    }

    #[test]
    fn test_let_statement() {
        let tests = vec![
            ("let a = 5; a;", Object::Integer(5)),
            ("let a = 5 * 5; a;", Object::Integer(25)),
            ("let a = 5; let b = a; b;", Object::Integer(5)),
            ("let a = 5; let b = a; let c = a + b + 5; c;", Object::Integer(15)),
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
