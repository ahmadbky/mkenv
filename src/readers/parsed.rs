//! Module containing everything related to the [`Parsed`] configuration value type.

use std::error::Error;

use crate::{
    descriptor::{ConfValDescriptor, ConfigValDescriptor},
    error::{ParseError, ReadVarError},
    var_reader::VarReader,
};

/// The type of the parsing function.
pub type ParseFn<T> = fn(&str) -> Result<T, Box<dyn Error>>;

/// A configuration value that parses the content of the inner configuration value to `T`.
///
/// To construct it, see [`parsed`][1].
///
/// ## Example
///
/// ```
/// # use mkenv::prelude::*;
/// # unsafe { std::env::set_var("TIMEOUT_MS", "30"); }
/// let my_config = TextVar::from_var_name("TIMEOUT_MS")
///   .parsed_from_str::<u64>();
/// let res = my_config.try_read_var();
/// # unsafe { std::env::remove_var("TIMEOUT_MS"); }
/// assert_eq!(res, Ok(30));
/// ```
///
/// [1]: crate::builder::VarReaderExt::parsed
pub struct Parsed<T, V> {
    pub(crate) var: V,
    pub(crate) parse_fn: ParseFn<T>,
}

impl<T, V: ConfigValDescriptor> ConfigValDescriptor for Parsed<T, V> {
    #[inline]
    fn describe_config_val(&self) -> &ConfValDescriptor {
        self.var.describe_config_val()
    }
}

impl<T, V> VarReader for Parsed<T, V>
where
    V: VarReader<Output: AsRef<str>>,
    ReadVarError: From<<V as VarReader>::Error>,
{
    type Output = T;
    type Error = ReadVarError;

    fn try_read_var(&self) -> Result<Self::Output, Self::Error> {
        let raw_val = self.var.try_read_var()?;
        let parse_res = (self.parse_fn)(raw_val.as_ref());
        parse_res.map_err(|source| ReadVarError::Other(Box::new(ParseError { source })))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        num::{IntErrorKind, ParseIntError},
        time::Duration,
    };

    use crate::{
        error::{ParseError, ReadVarError},
        prelude::*,
        tests::{assert_matches, with_env},
    };

    #[test]
    fn assert_parse_success() {
        const VAR_NAME: &str = "__TEST_PARSE_SUCCESS";

        let config = TextVar::from_var_name(VAR_NAME).parsed_from_str::<i32>();

        let res = with_env([(VAR_NAME, "30")], || config.try_read_var());
        assert_matches!(res, Ok(30));
    }

    #[test]
    fn assert_parse_fail() {
        const VAR_NAME: &str = "__TEST_PARSE_FAIL";

        let config = TextVar::from_var_name(VAR_NAME).parsed_from_str::<i32>();

        fn is_parse_error(e: &(dyn Error + 'static)) -> bool {
            e.downcast_ref::<ParseError>()
                .and_then(|e| e.source())
                .and_then(|e| e.downcast_ref::<ParseIntError>())
                .filter(|e| matches!(e.kind(), IntErrorKind::InvalidDigit))
                .is_some()
        }

        let res = with_env([(VAR_NAME, "foobar")], || config.try_read_var());
        assert_matches!(res, Err(ReadVarError::Other(e)) if is_parse_error(&*e));
    }

    #[test]
    fn assert_custom_parse_success() {
        const VAR_NAME: &str = "__TEST_CUSTOM_PARSE_SUCCESS";

        let config = TextVar::from_var_name(VAR_NAME).parsed(|input| {
            input
                .parse::<u64>()
                .map(|n| Duration::from_millis(n))
                .map_err(From::from)
        });

        let res = with_env([(VAR_NAME, "250")], || config.try_read_var());
        assert_matches!(res, Ok(d) if d.as_millis() == 250);
    }
}
