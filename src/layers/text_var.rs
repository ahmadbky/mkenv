//! Module containing everything related to the [`TextVar`] configuration value type.

use std::env;

use crate::{
    descriptor::{ConfigValueDescriptor, VarDescriptor},
    error::ReadVarError,
    layer::Layer,
};

/// A configuration value that simply returns the content of the environment variable.
///
/// ## Example
///
/// ```
/// # use mkenv::prelude::*;
/// # unsafe { std::env::set_var("USER_FIRSTNAME", "foobar"); }
/// let my_config = TextVar::from_var_name("USER_FIRSTNAME");
/// let res = my_config.try_get();
/// # unsafe { std::env::remove_var("USER_FIRSTNAME"); }
/// assert_eq!(res.as_deref(), Ok("foobar"));
/// ```
pub struct TextVar {
    descriptor: VarDescriptor,
}

impl TextVar {
    /// Creates a [`TextVar`] from the environment variable key.
    pub fn from_var_name(var_name: &'static str) -> Self {
        Self {
            descriptor: VarDescriptor {
                var_name,
                description: None,
                default_val_fmt: None,
            },
        }
    }

    /// Changes the description of the configuration descriptor.
    pub fn description(mut self, description: &'static str) -> Self {
        self.descriptor.description = Some(description);
        self
    }

    /// Changes the default value shown in the configuration descriptor.
    ///
    /// Note: the content of the text is only used as information to the user. It is up to you
    /// to really provide a default value with e.g. [`or_default_val`][1].
    ///
    /// [1]: crate::builder::LayerExt::or_default_val
    pub fn default_fmt_val(mut self, default_fmt_val: &'static str) -> Self {
        self.descriptor.default_val_fmt = Some(default_fmt_val);
        self
    }
}

impl ConfigValueDescriptor for TextVar {
    #[inline(always)]
    fn get_descriptor(&self) -> &VarDescriptor {
        &self.descriptor
    }
}

impl Layer for TextVar {
    type Output = String;
    type Error = ReadVarError;

    fn try_get(&self) -> Result<Self::Output, Self::Error> {
        env::var(self.descriptor.var_name).map_err(ReadVarError::Var)
    }
}

#[cfg(test)]
mod tests {
    use std::env::VarError;

    use crate::{
        error::ReadVarError,
        prelude::*,
        tests::{assert_matches, with_env},
    };

    #[test]
    fn assert_var_non_present() {
        const VAR_NAME: &str = "__TEST_VAR_NON_PRESENT";

        let config = TextVar::from_var_name(VAR_NAME);

        let res = with_env([], || config.try_get());
        assert_matches!(res, Err(ReadVarError::Var(VarError::NotPresent)));
    }

    #[test]
    fn assert_var_present() {
        const VAR_NAME: &str = "__TEST_VAR_PRESENT";

        let config = TextVar::from_var_name(VAR_NAME);

        let res = with_env([(VAR_NAME, "hello there")], || config.try_get());
        assert_matches!(res.as_deref(), Ok("hello there"));
    }
}
