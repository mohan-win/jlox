use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::token::Token;

use super::{
    interpreter_error::RuntimeResult,
    lox_function::LoxFunction,
    runtime_value::{LoxCallable, LoxInstance, RuntimeValue},
    Interpreter,
};

/// Internal class definiation of a LoxClass.
/// `Note:` This class definition is shared across all the instances of this class.
#[derive(Debug)]
struct LoxClassDefinition {
    name: String,
    super_class: Option<Rc<LoxClass>>,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClassDefinition {
    fn new(
        name: &str,
        super_class: Option<Rc<LoxClass>>,
        methods: HashMap<String, Rc<LoxFunction>>,
    ) -> LoxClassDefinition {
        LoxClassDefinition {
            name: String::from(name),
            super_class,
            methods,
        }
    }
    pub fn find_method(&self, method_name: &str) -> Option<Rc<LoxFunction>> {
        self.methods
            .get(method_name)
            .map(|method| method.clone())
            .or_else(|| {
                if let Some(super_class) = &self.super_class {
                    super_class.find_method(method_name)
                } else {
                    None
                }
            })
    }
}

impl fmt::Display for LoxClassDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct LoxClass(Rc<RefCell<LoxClassDefinition>>); // Why LoxClassDefinition should be mutable ? Its because the class definition can be extended using `extension methods`

impl LoxClass {
    pub fn new(
        name: &str,
        super_class: Option<Rc<LoxClass>>,
        methods: HashMap<String, Rc<LoxFunction>>,
    ) -> LoxClass {
        LoxClass(Rc::new(RefCell::new(LoxClassDefinition::new(
            name,
            super_class,
            methods,
        ))))
    }
    pub fn find_method(&self, method_name: &str) -> Option<Rc<LoxFunction>> {
        self.0.as_ref().borrow().find_method(method_name)
    }
    pub fn add_extension_methods(&self, methods: HashMap<String, Rc<LoxFunction>>) {
        self.0
            .as_ref()
            .borrow_mut()
            .methods
            .extend(methods.into_iter());
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class {}>", self.0.as_ref().borrow())
    }
}

impl LoxCallable for LoxClass {
    fn callable_type(&self) -> super::runtime_value::LoxCallableType {
        super::runtime_value::LoxCallableType::Class
    }
    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<RuntimeValue>) -> RuntimeResult {
        let instance = ClassInstance::new(self);
        if let Some(initializer) = self.find_method("init") {
            let initializer = initializer.bind(&instance);
            initializer.call(interpreter, arguments)
        } else {
            Ok(RuntimeValue::Instance(Rc::new(instance)))
        }
    }
}

#[derive(Debug)]
struct ClassInstanceData {
    kclass: Rc<RefCell<LoxClassDefinition>>,
    fields: HashMap<String, RuntimeValue>,
}

#[derive(Debug, Clone)]
pub struct ClassInstance(Rc<RefCell<ClassInstanceData>>);

impl ClassInstance {
    pub fn new(kclass: &LoxClass) -> ClassInstance {
        ClassInstance(Rc::new(RefCell::new(ClassInstanceData {
            kclass: Rc::clone(&kclass.0),
            fields: HashMap::new(),
        })))
    }
    fn lookup_method(&self, name: &Token) -> Option<Rc<LoxFunction>> {
        self.0
            .as_ref()
            .borrow()
            .kclass
            .as_ref()
            .borrow()
            .find_method(&name.lexeme)
    }
}

impl LoxInstance for ClassInstance {
    fn get(&self, name: &Token) -> Option<RuntimeValue> {
        self.0
            .as_ref()
            .borrow()
            .fields
            .get(&name.lexeme)
            .map(|field| field.clone())
            .or_else(|| {
                if let Some(method) = self.lookup_method(name) {
                    Some(RuntimeValue::Callable(Rc::new(method.bind(self))))
                } else {
                    None
                }
            })
    }

    fn set(&self, name: &Token, value: RuntimeValue) -> RuntimeValue {
        self.0
            .borrow_mut()
            .fields
            .insert(name.lexeme.clone(), value.clone());
        value
    }
}

impl fmt::Display for ClassInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<instance of {}>",
            self.0.as_ref().borrow().kclass.as_ref().borrow()
        )
    }
}
