//! Module containing everything related to the [`FileRead`] configuration value type.

use std::path::Path;

use crate::{
    descriptor::{ConfigValueDescriptor, VarDescriptor},
    error::ReadVarError,
    layer::Layer,
};

/// A configuration value that reads the content of the specified file.
///
/// The path to the file is given by the output of the inner configuration value.
///
/// To construct it, see [`file_read`][1].
///
/// ## Example
///
/// ```
/// # use mkenv::prelude::*;
/// # unsafe { std::env::set_var("CUSTOM_FILE_PATH", "./__doctest_file"); }
/// # std::fs::write("./__doctest_file", "hello").unwrap();
/// let my_config = TextVar::from_var_name("CUSTOM_FILE_PATH").file_read();
/// let res = my_config.try_get();
/// # unsafe { std::env::remove_var("CUSTOM_FILE_PATH"); }
/// # std::fs::remove_file("./__doctest_file").unwrap();
/// assert_eq!(res.as_deref(), Ok("hello"));
/// ```
///
/// [1]: crate::builder::LayerExt::file_read
pub struct FileRead<V> {
    pub(crate) var: V,
}

impl<V> ConfigValueDescriptor for FileRead<V>
where
    V: ConfigValueDescriptor,
{
    #[inline]
    fn get_descriptor(&self) -> &VarDescriptor {
        self.var.get_descriptor()
    }
}

impl<V> Layer for FileRead<V>
where
    V: Layer,
    <V as Layer>::Output: AsRef<Path>,
    ReadVarError: From<<V as Layer>::Error>,
{
    type Output = String;
    type Error = ReadVarError;

    fn try_get(&self) -> Result<Self::Output, Self::Error> {
        let path = self.var.try_get()?;
        std::fs::read_to_string(path.as_ref()).map_err(|e| ReadVarError::Other(Box::new(e)))
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs, io, path::Path};

    use crate::{
        error::{ParseError, ReadVarError},
        prelude::*,
        tests::{assert_matches, with_env},
    };

    fn with_file<F, R, P, C>(file_path: P, file_content: C, f: F) -> R
    where
        F: FnOnce() -> R,
        P: AsRef<Path>,
        C: AsRef<[u8]>,
    {
        fs::write(&file_path, file_content).unwrap();
        let r = f();
        fs::remove_file(file_path).unwrap();
        r
    }

    #[test]
    fn assert_not_found() {
        const VAR_NAME: &str = "__TEST_FILE_NOT_FOUND";
        let config = TextVar::from_var_name(VAR_NAME).file_read();

        fn is_not_found_err(e: &(dyn Error + 'static)) -> bool {
            e.downcast_ref::<io::Error>()
                .filter(|e| matches!(e.kind(), io::ErrorKind::NotFound))
                .is_some()
        }

        let res = with_env([(VAR_NAME, "./__test_file_not_found")], || config.try_get());
        assert_matches!(
            res,
            Err(ReadVarError::Other(e)) if is_not_found_err(&*e)
        );
    }

    #[test]
    fn assert_file_content() {
        const VAR_NAME: &str = "__TEST_FILE_CONTENT";
        const FILE_PATH: &str = "./__test_file_content";

        let config = TextVar::from_var_name(VAR_NAME).file_read();

        let res = with_file(FILE_PATH, "hello there", || {
            with_env([(VAR_NAME, FILE_PATH)], || config.try_get())
        });
        assert_matches!(res.as_deref(), Ok("hello there"));
    }

    #[test]
    fn assert_parsed_content() {
        const VAR_NAME: &str = "__TEST_PARSED_FILE_CONTENT";
        const FILE_PATH: &str = "./__test_parsed_file_content";

        let config = TextVar::from_var_name(VAR_NAME)
            .file_read()
            .parsed_from_str::<i32>();

        let res = with_file(FILE_PATH, "3", || {
            with_env([(VAR_NAME, FILE_PATH)], || config.try_get())
        });
        assert_matches!(res, Ok(3));
    }

    #[test]
    fn assert_file_content_parse_error() {
        const VAR_NAME: &str = "__TEST_FILE_CONTENT_PARSE_ERROR";
        const FILE_PATH: &str = "./__test_file_content_parse_ERROR";

        let config = TextVar::from_var_name(VAR_NAME)
            .file_read()
            .parsed_from_str::<i32>();

        fn is_parse_error(e: &(dyn Error + 'static)) -> bool {
            e.downcast_ref::<ParseError>().is_some()
        }

        let res = with_file(FILE_PATH, "foobar", || {
            with_env([(VAR_NAME, FILE_PATH)], || config.try_get())
        });
        assert_matches!(res, Err(ReadVarError::Other(e)) if is_parse_error(&*e));
    }

    #[test]
    fn assert_parsed_path() {
        const VAR_NAME: &str = "__TEST_PARSED_FILE_PATH";
        const FILE_PATH: &str = "./__test_parsed_file_path";

        let config = TextVar::from_var_name(VAR_NAME)
            .parsed(|input| Ok(input.split(":").nth(1).unwrap().to_owned()))
            .file_read();

        let res = with_file(FILE_PATH, "hello", || {
            with_env([(VAR_NAME, format!("booga:{FILE_PATH}").as_str())], || {
                config.try_get()
            })
        });
        assert_matches!(res.as_deref(), Ok("hello"));
    }

    #[test]
    fn assert_parsed_path_failed() {
        const VAR_NAME: &str = "__TEST_PARSED_FILE_PATH_FAILED";

        let config = TextVar::from_var_name(VAR_NAME)
            .parsed(|input| {
                input
                    .split(":")
                    .nth(1)
                    .ok_or_else(|| "this is bad".into())
                    .map(|s| s.to_owned())
            })
            .file_read();

        fn is_parse_error(e: &(dyn Error + 'static)) -> bool {
            e.downcast_ref::<ParseError>()
                .and_then(|e| e.source())
                .filter(|s| s.to_string() == "this is bad")
                .is_some()
        }

        let res = with_env([(VAR_NAME, "im not expected")], || config.try_get());
        assert_matches!(res, Err(ReadVarError::Other(e)) if is_parse_error(&*e));
    }
}
