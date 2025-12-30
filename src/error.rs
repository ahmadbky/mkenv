//! Contains all error types.

use std::{env::VarError, error::Error, fmt};

use crate::exec::FmtExecResults;

/// Generic error when reading an environment variable.
#[derive(Debug)]
pub enum ReadVarError {
    Var(VarError),
    Other(Box<dyn Error>),
}

impl PartialEq for ReadVarError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Var(e1), Self::Var(e2)) => e1 == e2,
            (Self::Other(_), Self::Other(_)) => true,
            _ => false,
        }
    }
}

impl fmt::Display for ReadVarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadVarError::Var(var_error) => fmt::Display::fmt(var_error, f),
            ReadVarError::Other(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl Error for ReadVarError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReadVarError::Var(var_error) => Some(var_error),
            ReadVarError::Other(error) => Some(&**error),
        }
    }
}

/// An error during the parsing of a configuration value.
#[derive(Debug)]
pub struct ParseError {
    pub(crate) source: Box<dyn Error>,
}

impl fmt::Display for ParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("parse error: ")?;
        fmt::Display::fmt(&self.source, f)
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.source)
    }
}

/// A cached error when reading the environment with [`Cached`][1].
///
/// [1]: crate::layers::Cached
#[derive(PartialEq)]
pub struct CachedError<'a, E>(pub(crate) &'a E);

// Note: we can't implement Error for CachedError, because it would conflict
// with the std impl `From<E: Error + 'a> for Box<dyn Error + 'a>`.
impl<E> From<CachedError<'_, E>> for Box<dyn Error>
where
    E: fmt::Display,
{
    fn from(value: CachedError<'_, E>) -> Self {
        format!("{}", value.0).into()
    }
}

impl<E: fmt::Display> fmt::Display for CachedError<'_, E> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<E: fmt::Debug> fmt::Debug for CachedError<'_, E> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

pub struct ConfigInitError<'a> {
    pub(crate) error: FmtExecResults<'a>,
}

impl fmt::Debug for ConfigInitError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigInitError")
            .field("correct_vars", &self.error.correct_vars)
            .field("incorrect_vars", &self.error.incorrect_vars)
            .finish()
    }
}

impl fmt::Display for ConfigInitError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error during configuration initialization:")?;
        fmt::Display::fmt(&self.error, f)
    }
}
