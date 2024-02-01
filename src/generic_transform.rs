//! Generic transformation of Lua types into Rust equivalents.

use core::fmt;
use std::collections::HashMap;

use full_moon::ast;
use smallstr::SmallString;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A Stack String is a string variant which, for values 16 bytes in size or smaller, is stored on the stack.
/// Larger values fall back to the heap.
///
/// The 16 byte size is chosen so that the actual sise of a `SString` is the same as the actual size of a basic heap-allocated `String`.
type SString = SmallString<[u8; 16]>;
pub type Table = HashMap<SString, Value>;
pub type List = Vec<Value>;

#[derive(Debug, Clone, PartialEq, derive_more::From, derive_more::TryInto)]
pub enum Value {
    Table(Table),
    List(List),
    String(SString),
    Number(f64),
    Bool(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Table(table) => {
                f.write_str("{")?;
                for (k, v) in table.iter() {
                    write!(f, "{k}={v},")?;
                }
                f.write_str("}")?;
                Ok(())
            }
            Value::List(list) => {
                f.write_str("{")?;
                for item in list.iter() {
                    write!(f, "{item},")?;
                }
                f.write_str("}")?;
                Ok(())
            }
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Number(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => f.write_str("nil"),
        }
    }
}

impl From<Value> for serde_json::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Table(table) => serde_json::Value::Object(
                table
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.into()))
                    .collect(),
            ),
            Value::List(list) => {
                serde_json::Value::Array(list.into_iter().map(Into::into).collect())
            }
            Value::String(s) => s.to_string().into(),
            Value::Number(n) => serde_json::Number::from_f64(n).into(),
            Value::Bool(b) => b.into(),
            Value::Nil => serde_json::Value::Null,
        }
    }
}

fn parse_token_ref(token_ref: &full_moon::tokenizer::TokenReference) -> Result<Value> {
    match token_ref.token_type() {
        full_moon::tokenizer::TokenType::StringLiteral { literal: value, .. }
        | full_moon::tokenizer::TokenType::Identifier { identifier: value } => {
            Ok(SString::from(value.as_str()).into())
        }
        full_moon::tokenizer::TokenType::Number { text } => Ok(text
            .as_str()
            .parse::<f64>()
            .map_err(Error::wrap("failed to parse as f64"))?
            .into()),
        full_moon::tokenizer::TokenType::Symbol { symbol } => match symbol {
            full_moon::tokenizer::Symbol::True => Ok(Value::Bool(true)),
            full_moon::tokenizer::Symbol::False => Ok(Value::Bool(false)),
            full_moon::tokenizer::Symbol::Nil => Ok(Value::Nil),
            _ => Err(Error::InvalidSymbol(symbol.to_string())),
        },
        full_moon::tokenizer::TokenType::Eof => Err(Error::TokenInvalidType("Eof")),
        full_moon::tokenizer::TokenType::MultiLineComment { .. } => {
            Err(Error::TokenInvalidType("MultiLineComment"))
        }
        full_moon::tokenizer::TokenType::Shebang { .. } => Err(Error::TokenInvalidType("Shebang")),
        full_moon::tokenizer::TokenType::SingleLineComment { .. } => {
            Err(Error::TokenInvalidType("SingleLineComment"))
        }
        full_moon::tokenizer::TokenType::Whitespace { .. } => {
            Err(Error::TokenInvalidType("Whitespace"))
        }
        _ => Err(Error::TokenInvalidType("Unknown")),
    }
}

pub(crate) fn parse_value(value: &ast::Expression) -> Result<Value> {
    match value {
        ast::Expression::Number(token_ref)
        | ast::Expression::String(token_ref)
        | ast::Expression::Symbol(token_ref) => parse_token_ref(token_ref),
        ast::Expression::Parentheses { expression, .. } => parse_value(expression),
        ast::Expression::TableConstructor(tc) => {
            if tc
                .fields()
                .iter()
                .all(|field| matches!(field, ast::Field::NoKey(_)))
            {
                Ok(parse_list(tc)?.into())
            } else {
                Ok(parse_table(tc)?.into())
            }
        }
        ast::Expression::UnaryOperator {
            unop: ast::UnOp::Minus(_),
            expression,
        } => {
            let Value::Number(number) = parse_value(expression)? else {
                return Err(Error::ExpressionInvalidType(expression.to_string()));
            };

            Ok(Value::Number(-number))
        }
        ast::Expression::BinaryOperator { lhs, binop, rhs } => {
            let lhs = parse_value(lhs)
                .map_err(Error::wrap("parsing lhs of binary operator expression"))?;
            let rhs = parse_value(rhs)
                .map_err(Error::wrap("parsing rhs of binary operator expression"))?;

            match (lhs, binop, rhs) {
                (lhs, ast::BinOp::TildeEqual(_), rhs) => Ok(Value::Bool(lhs != rhs)),
                (lhs, ast::BinOp::TwoEqual(_), rhs) => Ok(Value::Bool(lhs == rhs)),
                (Value::Bool(lhs), ast::BinOp::And(_), Value::Bool(rhs)) => {
                    Ok(Value::Bool(lhs && rhs))
                }
                (Value::Bool(lhs), ast::BinOp::Or(_), Value::Bool(rhs)) => {
                    Ok(Value::Bool(lhs || rhs))
                }
                (Value::Number(lhs), ast::BinOp::LessThan(_), Value::Number(rhs)) => {
                    Ok(Value::Bool(lhs < rhs))
                }
                (Value::Number(lhs), ast::BinOp::LessThanEqual(_), Value::Number(rhs)) => {
                    Ok(Value::Bool(lhs <= rhs))
                }
                (Value::Number(lhs), ast::BinOp::GreaterThan(_), Value::Number(rhs)) => {
                    Ok(Value::Bool(lhs > rhs))
                }
                (Value::Number(lhs), ast::BinOp::GreaterThanEqual(_), Value::Number(rhs)) => {
                    Ok(Value::Bool(lhs >= rhs))
                }
                (Value::Number(lhs), ast::BinOp::Minus(_), Value::Number(rhs)) => {
                    Ok(Value::Number(lhs - rhs))
                }
                (Value::Number(lhs), ast::BinOp::Percent(_), Value::Number(rhs)) => {
                    Ok(Value::Number(lhs % rhs))
                }
                (Value::Number(lhs), ast::BinOp::Plus(_), Value::Number(rhs)) => {
                    Ok(Value::Number(lhs + rhs))
                }
                (Value::Number(lhs), ast::BinOp::Slash(_), Value::Number(rhs)) => {
                    Ok(Value::Number(lhs / rhs))
                }
                (Value::Number(lhs), ast::BinOp::Star(_), Value::Number(rhs)) => {
                    Ok(Value::Number(lhs * rhs))
                }
                (Value::Number(lhs), ast::BinOp::Caret(_), Value::Number(rhs)) => {
                    Ok(Value::Number(lhs.powf(rhs)))
                }
                (Value::String(mut lhs), ast::BinOp::TwoDots(_), Value::String(rhs)) => {
                    lhs.push_str(&rhs);
                    Ok(Value::String(lhs))
                }
                _ => Err(Error::ExpressionInvalidType(format!(
                    "invalid binary expression: \"{value}\""
                ))),
            }
        }
        _ => Err(Error::ExpressionInvalidType(value.to_string())),
    }
}

fn parse_table(table: &ast::TableConstructor) -> Result<Table> {
    fn key_of(idx: usize, field: &ast::Field) -> Result<SString> {
        match field {
            ast::Field::NameKey { key: token_ref, .. }
            | ast::Field::ExpressionKey {
                key: ast::Expression::String(token_ref),
                ..
            }
            | ast::Field::ExpressionKey {
                key: ast::Expression::Number(token_ref),
                ..
            }
            | ast::Field::ExpressionKey {
                key: ast::Expression::Symbol(token_ref),
                ..
            } => match parse_token_ref(token_ref)? {
                Value::String(s) => Ok(s),
                otherwise => Ok(otherwise.to_string().into()),
            },
            ast::Field::ExpressionKey { key, .. } => match key {
                ast::Expression::BinaryOperator { .. } => {
                    Err(Error::FieldInvalidKeyType("BinaryOperator"))
                }
                ast::Expression::Parentheses { .. } => {
                    Err(Error::FieldInvalidKeyType("Parentheses"))
                }
                ast::Expression::UnaryOperator { .. } => {
                    Err(Error::FieldInvalidKeyType("UnaryOperator"))
                }
                ast::Expression::Function(_) => Err(Error::FieldInvalidKeyType("Function")),
                ast::Expression::FunctionCall(_) => Err(Error::FieldInvalidKeyType("FunctionCall")),
                ast::Expression::TableConstructor(_) => {
                    Err(Error::FieldInvalidKeyType("TableConstructor"))
                }
                ast::Expression::Var(_) => Err(Error::FieldInvalidKeyType("Var")),
                _ => Err(Error::FieldInvalidKeyType("Unknown")),
            },
            ast::Field::NoKey(_) => Ok(idx.to_string().into()),

            _ => Err(Error::FieldUnknownKeyType),
        }
    }

    fn value_of(field: &ast::Field) -> Result<&ast::Expression> {
        match field {
            ast::Field::ExpressionKey { value, .. }
            | ast::Field::NameKey { value, .. }
            | ast::Field::NoKey(value) => Ok(value),
            _ => Err(Error::FieldUnknownKeyType),
        }
    }

    let mut out = Table::with_capacity(table.fields().len());

    for (idx, field) in table.fields().iter().enumerate() {
        let key = key_of(idx, field)?;
        let value = parse_value(value_of(field)?)?;
        out.insert(key, value);
    }

    Ok(out)
}

fn parse_list(list: &ast::TableConstructor) -> Result<List> {
    let mut out = List::with_capacity(list.fields().len());

    for field in list.fields().iter() {
        let ast::Field::NoKey(expr) = field else {
            return Err(Error::FieldUnexpectedKey);
        };

        let value = parse_value(expr)?;
        out.push(value);
    }

    Ok(out)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("field had unknown key type")]
    FieldUnknownKeyType,
    #[error("field had invalid key type: {0}")]
    FieldInvalidKeyType(&'static str),
    #[error("field unexpectedly specified a key")]
    FieldUnexpectedKey,
    #[error("expression has invalid type: \"{0}\"")]
    ExpressionInvalidType(String),
    #[error("token has invalid type: {0}")]
    TokenInvalidType(&'static str),
    #[error("invalid symbol: {0}")]
    InvalidSymbol(String),
    #[error("{context}")]
    Wrap {
        context: String,
        #[source]
        inner: Box<dyn 'static + std::error::Error + Send + Sync>,
    },
    #[error("{0}")]
    AdHoc(String),
}

static_assertions::assert_impl_all!(Error: std::error::Error, Send, Sync);

impl Error {
    fn wrap<E>(msg: impl ToString) -> impl Fn(E) -> Self
    where
        E: 'static + std::error::Error + Send + Sync,
    {
        move |err| {
            let context = msg.to_string();
            Self::Wrap {
                context,
                inner: Box::new(err),
            }
        }
    }

    #[allow(dead_code)]
    fn ad_hoc(msg: impl ToString) -> impl Fn() -> Self {
        move || Self::AdHoc(msg.to_string())
    }
}
