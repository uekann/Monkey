use crate::ast::*;
use crate::object::{Environment, Object};
use anyhow::{bail, Result};

pub fn eval_program(program: Program) -> Result<Object> {
    /// Evaluate the given program and return the result.
    // Create a new environment for the program.
    let mut env = Environment::new();

    // Evaluate each statement in the program.
    let mut result = Object::Null;
    for statement in program.statements {
        result = eval_statement(statement, &mut env)?;

        // If the result is a ReturnValue, return the value.
        if let Object::ReturnValue(val) = result {
            return Ok(*val);
        }
    }
    Ok(result)
}

fn eval_statement(statement: Statement, env: &mut Environment) -> Result<Object> {
    match statement {
        // If the statement is an expression, evaluate it and return the result.
        Statement::ExpressionStatement(expr) => {
            let val = eval_expression(expr, env)?;
            Ok(val)
        }

        // If the statement is a block statement, evaluate each statement in the block.
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

        // If the statement is a return statement, evaluate the expression and return the result.
        Statement::ReturnStatement(expr) => {
            let val = eval_expression(expr, env)?;
            Ok(Object::ReturnValue(Box::new(val)))
        }

        // If the statement is a let statement, evaluate the expression and store the result in the environment.
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
        // If the expression is an integer literal, return the integer value.
        Expression::IntegerLiteral(i) => Ok(Object::Integer(i)),

        // If the expression is a boolean literal, return the boolean value.
        Expression::Boolean(b) => Ok(Object::Boolean(b)),

        // If the expression is an identifier, look up the value in the environment and return it.
        Expression::Identifier(name) => match env.get(&name) {
            Some(val) => Ok(val.clone()),
            None => bail!("identifier not found: {}", name),
        },

        // If the expression is a prefix expression, evaluate the right expression and apply the operator.
        Expression::PrefixExpression { operator, right } => {
            let right = eval_expression(*right, env)?;
            match operator.as_str() {
                "!" => eval_bang_prefix_expression(right),
                "-" => eval_minus_prefix_operator_expression(right),
                _ => Ok(Object::Null),
            }
        }

        // If the expression is an infix expression, evaluate the left and right expressions and apply the operator.
        Expression::InfixExpression {
            left,
            operator,
            right,
        } => {
            let left = eval_expression(*left, env)?;
            let right = eval_expression(*right, env)?;
            eval_infix_expression(operator, left, right)
        }

        // If the expression is a block expression, evaluate each statement in the block.
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
    /// Evaluate the given prefix expression with the '!' operator and return the result.
    match right {
        Object::Boolean(b) => Ok(Object::Boolean(!b)),
        _ => eval_bang_prefix_expression(right.cast_to_boolean()?),
    }
}

fn eval_minus_prefix_operator_expression(right: Object) -> Result<Object> {
    /// Evaluate the given prefix expression with the '-' operator and return the result.
    match right {
        Object::Integer(i) => Ok(Object::Integer(-i)),
        _ => bail!("cannot use '-' operator on {:?}", right),
    }
}

fn eval_infix_expression(operator: String, left: Object, right: Object) -> Result<Object> {
    /// Evaluate the given infix expression and return the result.
    match (left, right) {
        // If both operands are integers, apply the operator and return the result.
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

        // If both operands are booleans, apply the operator and return the result.
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

        // If both operands are null, apply the operator and return the result.
        (Object::Null, Object::Null) => match operator.as_str() {
            "==" => Ok(Object::Boolean(true)),
            "!=" => Ok(Object::Boolean(false)),
            _ => bail!("unknown operator: Null {} Null", operator),
        },

        // If the operands are different types, return an error.
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
            (
                "let a = 5; let b = a; let c = a + b + 5; c;",
                Object::Integer(15),
            ),
            ("let a = 5; let b = a; let a = 10; b;", Object::Integer(5)),
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
