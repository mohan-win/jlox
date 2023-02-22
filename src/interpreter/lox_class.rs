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

    /// Finds a given method with `method_name` in the inheritance hierarchy starting from base.
    pub fn find_method(&self, method_name: &str) -> Option<Rc<LoxFunction>> {
        let mut method = None;
        if let Some(super_class) = self.super_class.as_ref() {
            method = super_class.0.find_method(method_name);
        } else if method.is_none() {
            method = self
                .methods
                .get(method_name)
                .map(|method| Rc::clone(method));
        }
        method
    }

    /// Finds the nearest inner method of the given class.
    /// # Arguments
    /// * `class_name` - Name of the class
    /// * `method_name` - Name of the method
    pub fn find_inner_method(
        &self,
        class_name: &str,
        method_name: &str,
    ) -> Option<Rc<LoxFunction>> {
        let mut current_class = self;
        if self.name == class_name {
            None // Note: Inner method of the current class, is `None`
        } else {
            let mut sub_classes = Vec::new();
            loop {
                if let Some(super_class) = current_class.super_class.as_ref() {
                    sub_classes.push(current_class);
                    current_class = super_class.0.as_ref();
                    if current_class.name == class_name {
                        break;
                    }
                } else {
                    break;
                }
            }

            let inner_method = sub_classes.into_iter().rev().try_for_each(|sub_class| {
                if let Some(method) = sub_class.methods.get(method_name) {
                    Err(Rc::clone(method))
                } else {
                    Ok(())
                }
            });

            inner_method.map_or_else(|inner_method| Some(inner_method), |_| None)
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
    pub fn new(
        name: &str,
        super_class: Option<Rc<LoxClass>>,
        methods: HashMap<String, Rc<LoxFunction>>,
    ) -> LoxClass {
        LoxClass(Rc::new(LoxClassDefinition::new(name, super_class, methods)))
    }
    pub fn find_method(&self, method_name: &str) -> Option<Rc<LoxFunction>> {
        self.0.find_method(method_name)
    }
    pub fn find_inner_method(
        &self,
        class_name: &str,
        method_name: &str,
    ) -> Option<Rc<LoxFunction>> {
        self.0.find_inner_method(class_name, method_name)
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class {}>", self.0)
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
    kclass: Rc<LoxClassDefinition>,
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
        self.0.as_ref().borrow().kclass.find_method(&name.lexeme)
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
    fn get_inner(&self, class: &Token, method: &Token) -> Option<RuntimeValue> {
        self.0
            .as_ref()
            .borrow()
            .kclass
            .find_inner_method(&class.lexeme, &method.lexeme)
            .map(|method| RuntimeValue::Callable(Rc::new(method.bind(self))))
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
        write!(f, "<instance of {}>", self.0.as_ref().borrow().kclass)
    }
}
