use std::fmt;

use crate::descriptor::ConfigValueDescriptor;

/// Represents types able to read a value from the process environment.
#[cfg_attr(docsrs, doc(notable_trait))]
pub trait Layer {
    /// The type of the read output.
    type Output;
    /// The type of the error.
    type Error;

    /// Reads and returns the value from the environment, or returns an error.
    fn try_read_var(&self) -> Result<Self::Output, Self::Error>;

    /// Reads and returns the value from the environment, or panics on error.
    ///
    /// # Panic
    ///
    /// This function panics if it couldn't read the value from the environment, by printing
    /// a formatted message about the variable that failed.
    fn read_var(&self) -> Self::Output
    where
        Self: ConfigValueDescriptor,
        Self::Error: fmt::Display,
    {
        self.try_read_var().unwrap_or_else(|e| {
            let val_config = <Self as ConfigValueDescriptor>::get_descriptor(self);
            panic!(
                "couldn't get env var `{}` (expected type `{}`): {e}",
                val_config.var_name,
                std::any::type_name::<Self::Output>(),
            );
        })
    }
}

impl<T: Layer> Layer for &T {
    type Output = <T as Layer>::Output;
    type Error = <T as Layer>::Error;

    fn try_read_var(&self) -> Result<Self::Output, Self::Error> {
        <T as Layer>::try_read_var(self)
    }
}
