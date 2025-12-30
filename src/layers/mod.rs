//! Module containing configuration value types.
//!
//! Configuration value types are types implementing the [`Layer`][1] trait.
//!
//! [1]: crate::layer::Layer

pub mod cached;
pub mod file_read;
pub mod or_default;
pub mod parsed;
pub mod text_var;

pub use cached::Cached;
pub use file_read::FileRead;
pub use or_default::OrDefault;
pub use parsed::Parsed;
pub use text_var::TextVar;
