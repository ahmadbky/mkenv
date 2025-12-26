//! Contains everything related to the execution of a full read of a configuration.

use std::{error::Error, fmt};

use crate::{
    descriptor::{ConfValDescriptor, ConfigValDescriptor},
    var_reader::VarReader,
};

/// The result of a read of a configuration value.
pub struct ExecResult<'a> {
    #[doc(hidden)]
    pub config: &'a ConfValDescriptor,
    #[doc(hidden)]
    pub error: Option<Box<dyn Error + 'a>>,
}

impl<'a> ExecResult<'a> {
    #[doc(hidden)]
    pub fn from_config<T>(config: &'a T) -> Self
    where
        &'a T: VarReader,
        Box<dyn Error + 'a>: From<<&'a T as VarReader>::Error>,
        T: ConfigValDescriptor,
    {
        Self {
            config: config.describe_config_val(),
            error: config.try_read_var().err().map(From::from),
        }
    }
}

struct ExecFailedResult<'a> {
    config: &'a ConfValDescriptor,
    error: Box<dyn Error + 'a>,
}

/// Formats the results of a whole configuration read.
pub struct FmtExecResults<'a> {
    correct_vars: Vec<&'a ConfValDescriptor>,
    incorrect_vars: Vec<ExecFailedResult<'a>>,
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
pub trait ConfigExecutor {
    /// The type of the resulting collection of the read.
    type Iter<'a>: IntoIterator<Item = ExecResult<'a>>
    where
        Self: 'a;

    /// Reads the whole configuration values set, and returns the result.
    fn try_exec(&self) -> Self::Iter<'_>;
}
