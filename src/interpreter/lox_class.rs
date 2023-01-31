use std::fmt;

#[derive(Debug)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: &str) -> LoxClass {
        LoxClass {
            name: String::from(name),
        }
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
