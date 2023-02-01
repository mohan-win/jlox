use std::{collections::HashMap, fmt, rc::Rc};

use crate::token::Token;

use super::runtime_value::{LoxCallable, RuntimeValue};

/// Internal class definiation of a LoxClass.
/// `Note:` This class definition is shared across all the instances of this class.
#[derive(Debug)]
struct LoxClassDefinition {
    name: String,
}

impl LoxClassDefinition {
    fn new(name: &str) -> LoxClassDefinition {
        LoxClassDefinition {
            name: String::from(name),
        }
    }
}

impl fmt::Display for LoxClassDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct LoxClass(Rc<LoxClassDefinition>);

impl LoxClass {
    pub fn new(name: &str) -> LoxClass {
        LoxClass(Rc::new(LoxClassDefinition::new(name)))
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class {}>", self.0)
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut super::Interpreter,
        _arguments: Vec<super::runtime_value::RuntimeValue>,
    ) -> super::interpreter_error::RuntimeResult {
        let instance = LoxInstance::new(self);
        Ok(RuntimeValue::Instance(Rc::new(instance)))
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    kclass: Rc<LoxClassDefinition>,
    fields: HashMap<String, RuntimeValue>,
}

impl LoxInstance {
    pub fn new(kclass: &LoxClass) -> LoxInstance {
        LoxInstance {
            kclass: Rc::clone(&kclass.0),
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Option<RuntimeValue> {
        self.fields.get(&name.lexeme).map(|value| value.clone())
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<instance of {}>", self.kclass)
    }
}
