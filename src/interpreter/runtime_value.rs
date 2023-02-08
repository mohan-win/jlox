use super::interpreter_error::{RuntimeError, RuntimeResult};
use super::Interpreter;
use crate::ast::LitralValue;
use crate::token::Token;
use std::any::Any;
use std::cmp::{Ordering, PartialOrd};
use std::fmt::{self, Debug};
use std::ops::{Add, Div, Mul, Neg, Not, Sub};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoxCallableType {
    NativeFunction,
    Function,
    Class,
}

pub trait AsAny: 'static {
    fn as_any(self: Rc<Self>) -> Rc<dyn Any>;
}

impl<T: 'static> AsAny for T {
    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}

pub trait LoxCallable: AsAny + fmt::Display + Debug {
    fn callable_type(&self) -> LoxCallableType;
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<RuntimeValue>) -> RuntimeResult;
}

pub trait LoxInstance: AsAny + fmt::Display + Debug {
    fn get(&self, name: &Token) -> Option<RuntimeValue>;
    fn set(&self, name: &Token, value: RuntimeValue) -> RuntimeValue;
}

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    Callable(Rc<dyn LoxCallable>),
    Instance(Rc<dyn LoxInstance>),
}

impl Neg for RuntimeValue {
    type Output = RuntimeResult;
    fn neg(self) -> Self::Output {
        if let Self::Number(val) = self {
            Ok(Self::Number(val * -1.0))
        } else {
            Err(RuntimeError::new_with_message(
                "Can't negate anything other than number",
            ))
        }
    }
}

impl Mul for RuntimeValue {
    type Output = RuntimeResult;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs * rhs)),
            _ => Err(RuntimeError::new_with_message(
                "Multiplication is allowed only between numbers",
            )),
        }
    }
}

impl Div for RuntimeValue {
    type Output = RuntimeResult;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => {
                if rhs.total_cmp(&-0.0) == Ordering::Equal || rhs.total_cmp(&0.0) == Ordering::Equal
                {
                    Err(RuntimeError::new_with_message("divide by zero error"))
                } else {
                    Ok(Self::Number(lhs / rhs))
                }
            }
            _ => Err(RuntimeError::new_with_message(
                "division is allowed only between numbers",
            )),
        }
    }
}

impl Add for RuntimeValue {
    type Output = RuntimeResult;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs + rhs)),
            (Self::String(lhs), Self::String(rhs)) => Ok(Self::String(format!("{}{}", lhs, rhs))),
            _ => Err(RuntimeError::new_with_message(
                "addition is allowed only between numbers",
            )),
        }
    }
}

impl Sub for RuntimeValue {
    type Output = RuntimeResult;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs - rhs)),
            _ => Err(RuntimeError::new_with_message(
                "subtraction is allowed only between numbers",
            )),
        }
    }
}

impl Not for RuntimeValue {
    type Output = RuntimeResult;
    fn not(self) -> Self::Output {
        if let Self::Boolean(val) = self {
            Ok(Self::Boolean(!val))
        } else {
            Err(RuntimeError::new_with_message(
                "Not is allowed only on booleans",
            ))
        }
    }
}

impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(lhs), Self::String(rhs)) => lhs == rhs,
            (Self::Number(lhs), Self::Number(rhs)) => lhs == rhs,
            (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs == rhs,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(lhs), Self::String(rhs)) => lhs != rhs,
            (Self::Number(lhs), Self::Number(rhs)) => lhs != rhs,
            (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs != rhs,
            (Self::Nil, Self::Nil) => false,
            _ => true,
        }
    }
}

impl PartialOrd for RuntimeValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::String(lhs), Self::String(rhs)) => lhs.partial_cmp(rhs),
            (Self::Number(lhs), Self::Number(rhs)) => lhs.partial_cmp(rhs),
            (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs.partial_cmp(rhs),
            (Self::Nil, Self::Nil) => Some(Ordering::Equal),
            _ => None,
        }
    }
    fn lt(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(lhs), Self::String(rhs)) => lhs < rhs,
            (Self::Number(lhs), Self::Number(rhs)) => lhs < rhs,
            (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs < rhs,
            _ => false,
        }
    }
    fn le(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(lhs), Self::String(rhs)) => lhs <= rhs,
            (Self::Number(lhs), Self::Number(rhs)) => lhs <= rhs,
            (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs <= rhs,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
    fn gt(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(lhs), Self::String(rhs)) => lhs > rhs,
            (Self::Number(lhs), Self::Number(rhs)) => lhs > rhs,
            (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs > rhs,
            _ => false,
        }
    }
    fn ge(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(lhs), Self::String(rhs)) => lhs >= rhs,
            (Self::Number(lhs), Self::Number(rhs)) => lhs >= rhs,
            (Self::Boolean(lhs), Self::Boolean(rhs)) => lhs >= rhs,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl RuntimeValue {
    pub fn is_truthy(&self) -> Self {
        match self {
            Self::Nil | Self::Boolean(false) => Self::Boolean(false),
            _ => Self::Boolean(true),
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RuntimeValue::*;
        match self {
            Nil => write!(f, "Nil"),
            Number(value) => write!(f, "{}", value),
            String(value) => write!(f, "{}", value),
            Boolean(value) => write!(f, "{}", value),
            Callable(ptr) => write!(f, "{}", ptr),
            Instance(ptr) => write!(f, "{}", ptr),
        }
    }
}

impl From<LitralValue> for RuntimeValue {
    fn from(value: LitralValue) -> Self {
        match value {
            LitralValue::NUMBER(litral_value) => RuntimeValue::Number(litral_value),
            LitralValue::STRING(litral_value) => RuntimeValue::String(litral_value),
            LitralValue::True => RuntimeValue::Boolean(true),
            LitralValue::False => RuntimeValue::Boolean(false),
            LitralValue::Nil => RuntimeValue::Nil,
        }
    }
}

impl From<&RuntimeValue> for bool {
    fn from(value: &RuntimeValue) -> Self {
        if let RuntimeValue::Boolean(true) = value.is_truthy() {
            true
        } else {
            false
        }
    }
}

impl From<RuntimeValue> for bool {
    fn from(value: RuntimeValue) -> Self {
        if let RuntimeValue::Boolean(true) = value.is_truthy() {
            true
        } else {
            false
        }
    }
}
