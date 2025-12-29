//! Module containing everything related to environment value descriptors.

use std::fmt;

/// Describes a configuration value.
#[derive(Debug)]
pub struct VarDescriptor {
    /// The name of the environment variable this configuration value is read from.
    pub var_name: &'static str,

    pub(crate) description: Option<&'static str>,
    pub(crate) default_val_fmt: Option<&'static str>,
}

impl fmt::Display for VarDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.var_name)?;
        if let Some(desc) = self.description {
            write!(f, ": {desc}")?;
        }
        if let Some(default_val) = self.default_val_fmt {
            write!(f, " (default: {default_val})")?;
        }
        Ok(())
    }
}

/// Represents types able to describe a configuration value.
pub trait ConfigValueDescriptor {
    /// Returns the descriptor of the configuration value.
    fn get_descriptor(&self) -> &VarDescriptor;
}

/// Represents types able to describe a set of [configuration values][1].
///
/// [1]: crate::layers
pub trait ConfigDescriptor: Sized {
    /// Creates the configuration descriptor.
    fn define() -> Self;
}
