use std::fmt;

/// A simple animal trait
pub trait Animal {
    fn name(&self) -> &str;
    fn sound(&self) -> &str;
    fn describe(&self) -> String {
        format!("{} says {}", self.name(), self.sound())
    }
}

pub struct Dog {
    pub name: String,
}

impl Animal for Dog {
    fn name(&self) -> &str {
        &self.name
    }

    fn sound(&self) -> &str {
        "woof"
    }
}

impl fmt::Display for Dog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dog({})", self.name)
    }
}
