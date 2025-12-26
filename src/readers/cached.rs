//! Module containing everything related to the [`Cached`] configuration value type.

use std::{fmt, sync::OnceLock};

use crate::{
    descriptor::{ConfValDescriptor, ConfigValDescriptor},
    error::CachedError,
    var_reader::VarReader,
};

/// A cached configuration value.
///
/// Caching means it saves the result of the first read of the inner configuration value, and
/// returns it for every next read.
///
/// Each read returns a reference to the cached result.
///
/// To construct it, see [`cached`][1].
///
/// ## Example
///
/// ```
/// # use mkenv::prelude::*;
/// # unsafe { std::env::set_var("CACHED_VAR", "foo"); }
/// let my_config = TextVar::from_var_name("CACHED_VAR").cached();
/// let res = my_config.try_read_var();
/// # unsafe { std::env::remove_var("CACHED_VAR"); }
/// assert_eq!(res.map(|s| s.as_str()), Ok("foo"));
/// // var "CACHED_VAR" changed to "bar"
/// # unsafe { std::env::set_var("CACHED_VAR", "bar"); }
/// let res = my_config.try_read_var();
/// # unsafe { std::env::remove_var("CACHED_VAR"); }
/// assert_eq!(res.map(|s| s.as_str()), Ok("foo"));
/// ```
///
/// [1]: crate::builder::VarReaderExt::cached
pub struct Cached<V>
where
    V: VarReader,
{
    pub(crate) var: V,
    pub(crate) cached: OnceLock<Result<<V as VarReader>::Output, <V as VarReader>::Error>>,
}

impl<V: VarReader> Cached<V> {
    /// Same as [`VarReader::try_read_var`], re-declared for more convenience with references.
    #[inline(always)]
    pub fn try_read_var(
        &self,
    ) -> Result<&<V as VarReader>::Output, CachedError<'_, <V as VarReader>::Error>> {
        <&Self as VarReader>::try_read_var(&self)
    }

    /// Same as [`VarReader::read_var`], re-declared for more convenience with references.
    #[inline(always)]
    pub fn read_var(&self) -> &<V as VarReader>::Output
    where
        for<'a> &'a Self: ConfigValDescriptor,
        <V as VarReader>::Error: fmt::Display,
    {
        <&Self as VarReader>::read_var(&self)
    }

    /// Takes the ownership of the cached result.
    ///
    /// It returns `None` if the configuration value hasn't been read yet.
    pub fn take(&mut self) -> Option<Result<<V as VarReader>::Output, <V as VarReader>::Error>> {
        self.cached.take()
    }
}

impl<V: VarReader + ConfigValDescriptor> ConfigValDescriptor for Cached<V> {
    #[inline]
    fn describe_config_val(&self) -> &ConfValDescriptor {
        self.var.describe_config_val()
    }
}

impl<'a, V> VarReader for &'a Cached<V>
where
    V: VarReader,
{
    type Output = &'a <V as VarReader>::Output;
    type Error = CachedError<'a, <V as VarReader>::Error>;

    fn try_read_var(&self) -> Result<Self::Output, Self::Error> {
        self.cached
            .get_or_init(|| self.var.try_read_var())
            .as_ref()
            .map_err(CachedError)
    }
}

#[cfg(test)]
mod tests {
    use std::env::VarError;

    use crate::{
        error::{CachedError, ReadVarError},
        prelude::*,
        tests::{assert_matches, with_env},
    };

    #[test]
    fn assert_unset() {
        const VAR_NAME: &str = "__TEST_UNSET_VAR";
        let cached = TextVar::from_var_name(VAR_NAME).cached();

        let res = with_env([], || cached.try_read_var());
        assert_matches!(
            res,
            Err(CachedError(ReadVarError::Var(VarError::NotPresent)))
        );

        let res = with_env([(VAR_NAME, "random value")], || cached.try_read_var());
        assert_matches!(
            res,
            Err(CachedError(ReadVarError::Var(VarError::NotPresent)))
        );
    }

    #[test]
    fn assert_cached_inner_error() {
        const VAR_NAME: &str = "__TEST_CACHED_INNER_ERROR";
        let cached = TextVar::from_var_name(VAR_NAME)
            .parsed_from_str::<u32>()
            .cached();

        let res = with_env([(VAR_NAME, "foobar")], || cached.try_read_var());
        assert_matches!(res, Err(CachedError(ReadVarError::Other(_))));

        let res = with_env([(VAR_NAME, "3")], || cached.try_read_var());
        assert_matches!(res, Err(CachedError(ReadVarError::Other(_))));
    }

    #[test]
    fn assert_cached_val() {
        const VAR_NAME: &str = "__TEST_CACHED_VALUE";
        let cached = TextVar::from_var_name(VAR_NAME).cached();

        let res = with_env([(VAR_NAME, "foo")], || cached.try_read_var());
        assert_matches!(res.map(|s| s.as_str()), Ok("foo"));

        let res = with_env([(VAR_NAME, "bar")], || cached.try_read_var());
        assert_matches!(res.map(|s| s.as_str()), Ok("foo"));
    }
}
