use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{
    interpreter::{
        interpreter_error::RuntimeResult,
        lox_function::LoxFunction,
        runtime_value::{LoxCallable, LoxInstance, RuntimeValue},
        Interpreter,
    },
    token::Token,
};

#[derive(Debug)]
struct LoxMetaClassDefinition {
    name: String,
    class_methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxMetaClassDefinition {
    fn new(name: &str, class_methods: HashMap<String, Rc<LoxFunction>>) -> LoxMetaClassDefinition {
        LoxMetaClassDefinition {
            name: String::from(name),
            class_methods,
        }
    }
    fn find_class_method(&self, class_method_name: &str) -> Option<Rc<LoxFunction>> {
        self.class_methods
            .get(class_method_name)
            .map(|class_method| Rc::clone(class_method))
    }
}

impl fmt::Display for LoxMetaClassDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct LoxMetaClass(Rc<LoxMetaClassDefinition>);

impl LoxMetaClass {
    pub fn new(name: &str, class_methods: HashMap<String, Rc<LoxFunction>>) -> LoxMetaClass {
        LoxMetaClass(Rc::new(LoxMetaClassDefinition::new(name, class_methods)))
    }
}

impl fmt::Display for LoxMetaClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<metaclass {}>", self.0)
    }
}

impl LoxCallable for LoxMetaClass {
    fn arity(&self) -> usize {
        if let Some(class_initializer) = self.0.find_class_method("init") {
            class_initializer.arity()
        } else {
            0
        }
    }

    // ToDo:: Instantiate meta_class when a class statement is parsed ??
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<RuntimeValue>) -> RuntimeResult {
        let instance = MetaClassInstance::new(self);
        if let Some(initializer) = self.0.find_class_method("init") {
            let initializer = initializer.bind(&instance);
            initializer.call(interpreter, arguments)
        } else {
            Ok(RuntimeValue::Instance(Rc::new(instance)))
        }
    }
}

#[derive(Debug)]
struct MetaClassInstanceData {
    meta_kclass: Rc<LoxMetaClassDefinition>,
    class_fields: HashMap<String, RuntimeValue>,
}

#[derive(Debug, Clone)]
pub struct MetaClassInstance(Rc<RefCell<MetaClassInstanceData>>);

impl MetaClassInstance {
    pub fn new(meta_kclass: &LoxMetaClass) -> MetaClassInstance {
        MetaClassInstance(Rc::new(RefCell::new(MetaClassInstanceData {
            meta_kclass: Rc::clone(&meta_kclass.0),
            class_fields: HashMap::new(),
        })))
    }

    fn lookup_class_method(&self, class_method_name: &Token) -> Option<Rc<LoxFunction>> {
        self.0
            .as_ref()
            .borrow()
            .meta_kclass
            .find_class_method(&class_method_name.lexeme)
    }
}

impl LoxInstance for MetaClassInstance {
    fn get(&self, name: &Token) -> Option<RuntimeValue> {
        self.0
            .as_ref()
            .borrow()
            .class_fields
            .get(&name.lexeme)
            .map(|class_field| class_field.clone())
            .or_else(|| {
                if let Some(class_method) = self.lookup_class_method(name) {
                    Some(RuntimeValue::Callable(Rc::new(class_method.bind(self))))
                } else {
                    None
                }
            })
    }
    fn set(&self, name: &Token, value: RuntimeValue) -> RuntimeValue {
        self.0
            .as_ref()
            .borrow_mut()
            .class_fields
            .insert(name.lexeme.clone(), value.clone());
        value.clone()
    }
}

impl fmt::Display for MetaClassInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<meta class instance of {} class>",
            self.0.as_ref().borrow().meta_kclass.name
        )
    }
}
