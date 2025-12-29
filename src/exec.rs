//! Contains everything related to the execution of a full read of a configuration.

use std::{error::Error, fmt, panic};

use crate::{
    descriptor::{ConfigValueDescriptor, VarDescriptor},
    error::ConfigInitError,
    layer::Layer,
};

/// The result of a read of a configuration value.
pub struct ExecResult<'a> {
    #[doc(hidden)]
    pub config: &'a VarDescriptor,
    #[doc(hidden)]
    pub error: Option<Box<dyn Error + 'a>>,
}

impl<'a> ExecResult<'a> {
    #[doc(hidden)]
    pub fn from_config<T>(config: &'a T) -> Self
    where
        &'a T: Layer,
        Box<dyn Error + 'a>: From<<&'a T as Layer>::Error>,
        T: ConfigValueDescriptor,
    {
        Self {
            config: config.get_descriptor(),
            error: config.try_read_var().err().map(From::from),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ExecFailedResult<'a> {
    config: &'a VarDescriptor,
    error: Box<dyn Error + 'a>,
}

/// Formats the results of a whole configuration read.
pub struct FmtExecResults<'a> {
    pub(crate) correct_vars: Vec<&'a VarDescriptor>,
    pub(crate) incorrect_vars: Vec<ExecFailedResult<'a>>,
}

impl fmt::Display for FmtExecResults<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Got {} incorrect variable{}",
            self.incorrect_vars.len(),
            if self.incorrect_vars.len() > 1 {
                "s"
            } else {
                ""
            }
        )?;

        for var in &self.incorrect_vars {
            writeln!(f, "- `{}`: {}", var.config.var_name, var.error)?;
        }

        writeln!(
            f,
            "Got {} valid variable{}",
            self.correct_vars.len(),
            if self.correct_vars.len() > 1 { "s" } else { "" }
        )?;

        for var in &self.correct_vars {
            writeln!(f, "- `{}`", var.var_name)?;
        }

        writeln!(f, "Full required environment description:")?;
        for var_desc in self
            .incorrect_vars
            .iter()
            .map(|v| v.config)
            .chain(self.correct_vars.iter().copied())
        {
            writeln!(f, "- {var_desc}")?;
        }

        Ok(())
    }
}

/// Returns a formatted version of the given configuration value results.
pub fn fmt_exec_results<'a, I>(results: I) -> FmtExecResults<'a>
where
    I: IntoIterator<Item = ExecResult<'a>>,
{
    let mut incorrect_vars = Vec::new();
    let mut correct_vars = Vec::new();

    for result in results {
        if let Some(err) = result.error {
            incorrect_vars.push(ExecFailedResult {
                config: result.config,
                error: err,
            });
        } else {
            correct_vars.push(result.config);
        }
    }

    FmtExecResults {
        correct_vars,
        incorrect_vars,
    }
}

/// Represents types able to read a set of configuration values.
pub trait ConfigInitializer {
    /// The type of the resulting collection of the read.
    type Iter<'a>: IntoIterator<Item = ExecResult<'a>>
    where
        Self: 'a;

    /// Reads the whole configuration values set, and returns the result in the form of an iterator.
    ///
    /// # Note about caching
    ///
    /// Please note that for types using a cached configuration, this method will make it use
    /// the related environment variable for the first time (if not done previously).
    ///
    /// The consequence is that for next reads using the [`Layer`] trait, even if the
    /// environment changed in the mean time, the result will always be the same for the fields
    /// using the cached type.
    ///
    /// # Example
    ///
    /// ```
    /// # use mkenv::prelude::*;
    /// use mkenv::make_config;
    ///
    /// make_config! {
    ///   struct MyConfig {
    ///     user: {
    ///       var_name: "USER",
    ///       layers: [cached()],
    ///     }
    ///   }
    /// }
    ///
    /// let config = MyConfig::define();
    /// // Once this method is called...
    /// let _ = config.init_raw();
    /// // ...the return of the `read_var` for this field
    /// // will always be the same.
    /// let user = config.user.read_var();
    /// ```
    fn init_raw(&self) -> Self::Iter<'_>;

    /// Reads the configuration, and returns a formatted result in case of any error.
    ///
    /// If some fields use a cached layer, consider reading the note about [caching][1]
    ///
    /// # Return
    ///
    /// This method returns `Err(_)` if any configuration value failed to read, and `Ok(_)`
    /// otherwise.
    ///
    /// [1]: ConfigInitializer#note-about-caching
    fn try_init(&self) -> Result<(), ConfigInitError<'_>> {
        let res = fmt_exec_results(self.init_raw());
        if res.incorrect_vars.is_empty() {
            Ok(())
        } else {
            Err(ConfigInitError { error: res })
        }
    }

    /// Reads the configuration, and panics in case of any error.
    ///
    /// If some fields use a cached layer, consider reading the note about [caching][1]
    ///
    /// # Panic
    ///
    /// This method panics if any configuration value failed to read.
    ///
    /// [1]: ConfigInitializer#note-about-caching
    fn init(&self) {
        self.try_init().unwrap_or_else(|e| {
            panic!("{e}");
        });
    }
}
