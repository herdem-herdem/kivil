use crate::parser::{Expr, Operator};

pub fn evaluate(expression: &Expr) -> Result<f64, String> {
    let result = match expression {
        Expr::Number(value) => *value,
        Expr::Negate(operand) => -evaluate(operand)?,
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left = evaluate(left)?;
            let right = evaluate(right)?;

            match operator {
                Operator::Add => left + right,
                Operator::Subtract => left - right,
                Operator::Multiply => left * right,
                Operator::Divide => {
                    if right == 0.0 {
                        return Err("Sıfıra bölme yapılamaz.".into());
                    }
                    left / right
                }
            }
        }
    };

    if !result.is_finite() {
        return Err("İşlem sonucu desteklenen sayı aralığının dışında.".into());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::evaluate;
    use crate::{lexer::tokenize, parser::parse};

    fn calculate(source: &str) -> Result<Vec<f64>, String> {
        let tokens = tokenize(source)?;
        let expressions = parse(&tokens)?;
        expressions.iter().map(evaluate).collect()
    }

    #[test]
    fn evaluates_precedence_parentheses_and_left_associativity() {
        assert_eq!(
            calculate("10 + 20 * 3\n(10 + 20) * 3\n20 - 5 - 3\n24 / 4 / 2"),
            Ok(vec![70.0, 90.0, 12.0, 3.0])
        );
    }

    #[test]
    fn evaluates_signs_decimals_and_nested_parentheses() {
        assert_eq!(
            calculate("-3 * (2 + 4)\n5 / 2\n1.5 + 2\n--3\n+2\n2 * -3\n1 - -2\n((2 + 3) * 4)"),
            Ok(vec![-18.0, 2.5, 3.5, 3.0, 2.0, -6.0, 3.0, 20.0])
        );
    }

    #[test]
    fn rejects_division_by_zero_and_overflow() {
        for source in ["1 / 0", "1 / (2 - 2)", "1 / -0.0"] {
            assert_eq!(calculate(source), Err("Sıfıra bölme yapılamaz.".into()));
        }

        let large_number = format!("{}.0", "9".repeat(200));
        assert!(calculate(&format!("{large_number} * {large_number}")).is_err());
    }
}
