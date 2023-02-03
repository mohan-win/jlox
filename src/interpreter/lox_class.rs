use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::token::Token;

use super::{
    interpreter_error::RuntimeResult,
    lox_function::LoxFunction,
    runtime_value::{LoxCallable, RuntimeValue},
    Interpreter,
};

/// Internal class definiation of a LoxClass.
/// `Note:` This class definition is shared across all the instances of this class.
#[derive(Debug)]
struct LoxClassDefinition {
    name: String,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClassDefinition {
    fn new(name: &str, methods: HashMap<String, Rc<LoxFunction>>) -> LoxClassDefinition {
        LoxClassDefinition {
            name: String::from(name),
            methods,
        }
    }
    pub fn find_methods(&self, method_name: &str) -> Option<&Rc<LoxFunction>> {
        self.methods.get(method_name)
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
    pub fn new(name: &str, methods: HashMap<String, Rc<LoxFunction>>) -> LoxClass {
        LoxClass(Rc::new(LoxClassDefinition::new(name, methods)))
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class {}>", self.0)
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        if let Some(initializer) = self.0.find_methods("init") {
            initializer.arity()
        } else {
            0
        }
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<RuntimeValue>) -> RuntimeResult {
        let instance = Rc::new(RefCell::new(LoxInstance::new(self)));
        if let Some(initializer) = self.0.find_methods("init") {
            let initializer = initializer.bind(&instance);
            initializer.call(interpreter, arguments)?;
        }
        Ok(RuntimeValue::Instance(instance))
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

    pub fn get(this: &Rc<RefCell<Self>>, name: &Token) -> Option<RuntimeValue> {
        let me = this.as_ref().borrow();
        me.fields
            .get(&name.lexeme)
            .map(|field| field.clone())
            .or_else(|| {
                if let Some(method) = me.lookup_methods(name) {
                    Some(RuntimeValue::Callable(Rc::new(method.bind(this))))
                } else {
                    None
                }
            })
    }

    pub fn set(&mut self, name: &Token, value: RuntimeValue) -> RuntimeValue {
        self.fields.insert(name.lexeme.clone(), value.clone());
        value
    }

    fn lookup_methods(&self, name: &Token) -> Option<&Rc<LoxFunction>> {
        self.kclass.find_methods(&name.lexeme)
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<instance of {}>", self.kclass)
    }
}
