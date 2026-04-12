mod rust;

pub use rust::RustResolver;

pub fn get_resolver(lang: spy_core::Language) -> Option<Box<dyn spy_core::Resolver>> {
    match lang {
        spy_core::Language::Rust => Some(Box::new(RustResolver)),
        _ => None,
    }
}
