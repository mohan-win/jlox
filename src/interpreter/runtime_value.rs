use std::cmp::{Ordering, PartialOrd};
use std::ops::{Add, Div, Mul, Neg, Not, Sub};

pub enum RuntimeValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Neg for RuntimeValue {
    type Output = Self;
    fn neg(self) -> Self::Output {
        if let Self::Number(val) = self {
            Self::Number(val * -1.0)
        } else {
            panic!("Can't negate anything other than number") // ToDo:: replace it with runtime error
        }
    }
}

impl Mul for RuntimeValue {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Self::Number(lhs * rhs),
            _ => panic!("Multiplication is allowed only between numbers"), // ToDo::
        }
    }
}

impl Div for RuntimeValue {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Self::Number(lhs / rhs),
            _ => panic!("division is allowed only between numbers"), // ToDo::
        }
    }
}

impl Add for RuntimeValue {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Self::Number(lhs + rhs),
            (Self::String(lhs), Self::String(rhs)) => Self::String(format!("{}{}", lhs, rhs)),
            _ => panic!("addition is allowed only between numbers"), // ToDo::
        }
    }
}

impl Sub for RuntimeValue {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl Not for RuntimeValue {
    type Output = Self;
    fn not(self) -> Self::Output {
        if let Self::Boolean(val) = self {
            Self::Boolean(!val)
        } else {
            panic!("Not is allowed only on booleans")
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

use crate::ast::LitralValue;
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
