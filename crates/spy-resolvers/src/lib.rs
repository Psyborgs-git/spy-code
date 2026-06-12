mod go;
mod python;
mod rust;
mod ts;
mod java;
mod asset;

pub use go::GoResolver;
pub use python::PythonResolver;
pub use rust::RustResolver;
pub use ts::{JavaScriptResolver, TypeScriptResolver};
pub use java::JavaResolver;
pub use asset::AssetResolver;

pub fn get_resolver(lang: spy_core::Language) -> Option<Box<dyn spy_core::Resolver>> {
    match lang {
        spy_core::Language::Rust => Some(Box::new(RustResolver)),
        spy_core::Language::Python => Some(Box::new(PythonResolver)),
        spy_core::Language::TypeScript => Some(Box::new(TypeScriptResolver)),
        spy_core::Language::JavaScript => Some(Box::new(JavaScriptResolver)),
        spy_core::Language::Go => Some(Box::new(GoResolver)),
        spy_core::Language::Java => Some(Box::new(JavaResolver)),
        spy_core::Language::Markdown
        | spy_core::Language::Text
        | spy_core::Language::Image
        | spy_core::Language::Pdf
        | spy_core::Language::Docx
        | spy_core::Language::Video
        | spy_core::Language::Svg
        | spy_core::Language::Other => Some(Box::new(AssetResolver::new(lang))),
    }
}
