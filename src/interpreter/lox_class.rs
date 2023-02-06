use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::token::Token;

use super::{
    interpreter_error::RuntimeResult,
    lox_function::LoxFunction,
    runtime_value::{LoxCallable, LoxCallableInstance, LoxInstance, RuntimeValue},
    Interpreter,
};

/// Internal class definiation of a LoxClass.
/// `Note:` This class definition is shared across all the instances of this class.
#[derive(Debug)]
struct LoxClassDefinition {
    name: String,
    methods: HashMap<String, Rc<LoxFunction>>,
    class_methods: HashMap<String, Rc<LoxFunction>>,
    class_fields: HashMap<String, RuntimeValue>,
}

impl LoxClassDefinition {
    fn new(
        name: &str,
        methods: HashMap<String, Rc<LoxFunction>>,
        class_methods: HashMap<String, Rc<LoxFunction>>,
    ) -> LoxClassDefinition {
        LoxClassDefinition {
            name: String::from(name),
            methods,
            class_methods,
            class_fields: HashMap::new(),
        }
    }
    fn find_method(&self, method_name: &str) -> Option<Rc<LoxFunction>> {
        self.methods.get(method_name).map(|method| method.clone())
    }
}

impl fmt::Display for LoxClassDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct LoxClass(Rc<RefCell<LoxClassDefinition>>);

impl LoxClass {
    pub fn new(
        name: &str,
        methods: HashMap<String, Rc<LoxFunction>>,
        class_methods: HashMap<String, Rc<LoxFunction>>,
        interpreter: &mut Interpreter,
    ) -> RuntimeResult<LoxClass> {
        let lox_class = LoxClass(Rc::new(RefCell::new(LoxClassDefinition::new(
            name,
            methods,
            class_methods,
        ))));

        // Call "class 'init'" method if it's there
        if let Some(class_initializer) = lox_class.lookup_class_method("init") {
            assert!(
                class_initializer.arity() == 0,
                "Can't call class init with params"
            ); // ToDo:: ensure this in resolver

            class_initializer.call(interpreter, Vec::new())?;
        }

        Ok(lox_class)
    }
    fn lookup_class_method(&self, class_method_name: &str) -> Option<Rc<LoxFunction>> {
        self.0
            .as_ref()
            .borrow()
            .class_methods
            .get(class_method_name)
            .map(|class_method| Rc::clone(class_method))
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class {}>", self.0.as_ref().borrow())
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        if let Some(initializer) = self.0.as_ref().borrow().find_method("init") {
            initializer.arity()
        } else {
            0
        }
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<RuntimeValue>) -> RuntimeResult {
        let instance = ClassInstance::new(self);
        if let Some(initializer) = self.0.as_ref().borrow().find_method("init") {
            let initializer = initializer.bind(&instance);
            initializer.call(interpreter, arguments)
        } else {
            Ok(RuntimeValue::Instance(Rc::new(instance)))
        }
    }
}

impl LoxInstance for LoxClass {
    fn get(&self, name: &Token) -> Option<RuntimeValue> {
        self.0
            .as_ref()
            .borrow()
            .class_fields
            .get(name.lexeme.as_str())
            .map(|value| value.clone())
            .or_else(|| {
                if let Some(class_method) = self.lookup_class_method(&name.lexeme) {
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
        value
    }
}

impl LoxCallableInstance for LoxClass {}

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
