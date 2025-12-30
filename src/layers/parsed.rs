//! Module containing everything related to the [`Parsed`] configuration value type.

use std::error::Error;

use crate::{
    descriptor::{ConfigValueDescriptor, VarDescriptor},
    error::{ParseError, ReadVarError},
    layer::Layer,
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
/// let res = my_config.try_get();
/// # unsafe { std::env::remove_var("TIMEOUT_MS"); }
/// assert_eq!(res, Ok(30));
/// ```
///
/// [1]: crate::builder::LayerExt::parsed
pub struct Parsed<T, V> {
    pub(crate) var: V,
    pub(crate) parse_fn: ParseFn<T>,
}

impl<T, V: ConfigValueDescriptor> ConfigValueDescriptor for Parsed<T, V> {
    #[inline]
    fn get_descriptor(&self) -> &VarDescriptor {
        self.var.get_descriptor()
    }
}

impl<T, V> Layer for Parsed<T, V>
where
    V: Layer<Output: AsRef<str>>,
    ReadVarError: From<<V as Layer>::Error>,
{
    type Output = T;
    type Error = ReadVarError;

    fn try_get(&self) -> Result<Self::Output, Self::Error> {
        let raw_val = self.var.try_get()?;
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

        let res = with_env([(VAR_NAME, "30")], || config.try_get());
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

        let res = with_env([(VAR_NAME, "foobar")], || config.try_get());
        assert_matches!(res, Err(ReadVarError::Other(e)) if is_parse_error(&*e));
    }

    #[test]
    fn assert_custom_parse_success() {
        const VAR_NAME: &str = "__TEST_CUSTOM_PARSE_SUCCESS";

        let config = TextVar::from_var_name(VAR_NAME).parsed(|input| {
            input
                .parse::<u64>()
                .map(Duration::from_millis)
                .map_err(From::from)
        });

        let res = with_env([(VAR_NAME, "250")], || config.try_get());
        assert_matches!(res, Ok(d) if d.as_millis() == 250);
    }
}
