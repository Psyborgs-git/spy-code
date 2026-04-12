mod go;
mod python;
mod rust;
mod ts;

pub use go::GoResolver;
pub use python::PythonResolver;
pub use rust::RustResolver;
pub use ts::{JavaScriptResolver, TypeScriptResolver};

pub fn get_resolver(lang: spy_core::Language) -> Option<Box<dyn spy_core::Resolver>> {
    match lang {
        spy_core::Language::Rust => Some(Box::new(RustResolver)),
        spy_core::Language::Python => Some(Box::new(PythonResolver)),
        spy_core::Language::TypeScript => Some(Box::new(TypeScriptResolver)),
        spy_core::Language::JavaScript => Some(Box::new(JavaScriptResolver)),
        spy_core::Language::Go => Some(Box::new(GoResolver)),
    }
}
