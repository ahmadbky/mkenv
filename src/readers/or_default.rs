//! Module containing everything related to the [`OrDefault`] configuration value type.

use std::convert::Infallible;

use crate::{
    descriptor::{ConfValDescriptor, ConfigValDescriptor},
    var_reader::VarReader,
};

/// Reads the inner configuration value, or returns a default value.
///
/// The read of this type can never fail.
///
/// To construct it, see [`or_default_val`][1].
///
/// ## Example
///
/// ```
/// # use mkenv::prelude::*;
/// let my_config = TextVar::from_var_name("LASTNAME")
///   .or_default_val(|| "hello there".to_owned());
/// let res = my_config.try_read_var();
/// assert_eq!(res.as_deref(), Ok("hello there"));
/// ```
///
/// [1]: crate::builder::VarReaderExt::or_default_val
pub struct OrDefault<V: VarReader> {
    pub(crate) var: V,
    pub(crate) default_fn: fn() -> <V as VarReader>::Output,
}

impl<V: VarReader + ConfigValDescriptor> ConfigValDescriptor for OrDefault<V> {
    fn describe_config_val(&self) -> &ConfValDescriptor {
        self.var.describe_config_val()
    }
}

impl<V> VarReader for OrDefault<V>
where
    V: VarReader,
{
    type Output = <V as VarReader>::Output;
    type Error = Infallible;

    fn try_read_var(&self) -> Result<Self::Output, Self::Error> {
        Ok(self
            .var
            .try_read_var()
            .unwrap_or_else(|_| (self.default_fn)()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{prelude::*, tests::with_env};

    #[test]
    fn assert_default_val() {
        const VAR_NAME: &str = "__TEST_DEFAULT_VAL";

        let config = TextVar::from_var_name(VAR_NAME).or_default_val(|| "hello".to_owned());

        let Ok(res) = with_env([], || config.try_read_var());
        assert_eq!(res, "hello");
    }

    #[test]
    fn assert_non_default_val() {
        const VAR_NAME: &str = "__TEST_NON_DEFAULT_VAL";

        let config = TextVar::from_var_name(VAR_NAME).or_default_val(|| "hello".to_owned());

        let Ok(res) = with_env([(VAR_NAME, "hi")], || config.try_read_var());
        assert_eq!(res, "hi");
    }
}
