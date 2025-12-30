//! Module containing everything related to the [`Cached`] configuration value type.

use std::{fmt, sync::OnceLock};

use crate::{
    descriptor::{ConfigValueDescriptor, VarDescriptor},
    error::CachedError,
    layer::Layer,
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
/// let res = my_config.try_get();
/// # unsafe { std::env::remove_var("CACHED_VAR"); }
/// assert_eq!(res.map(|s| s.as_str()), Ok("foo"));
/// // var "CACHED_VAR" changed to "bar"
/// # unsafe { std::env::set_var("CACHED_VAR", "bar"); }
/// let res = my_config.try_get();
/// # unsafe { std::env::remove_var("CACHED_VAR"); }
/// assert_eq!(res.map(|s| s.as_str()), Ok("foo"));
/// ```
///
/// [1]: crate::builder::LayerExt::cached
pub struct Cached<V>
where
    V: Layer,
{
    pub(crate) var: V,
    pub(crate) cached: OnceLock<Result<<V as Layer>::Output, <V as Layer>::Error>>,
}

impl<V: Layer> Cached<V> {
    /// Same as [`Layer::try_get`], re-declared for more convenience with references.
    #[inline(always)]
    pub fn try_get(&self) -> Result<&<V as Layer>::Output, CachedError<'_, <V as Layer>::Error>> {
        <&Self as Layer>::try_get(&self)
    }

    /// Same as [`Layer::get`], re-declared for more convenience with references.
    #[inline(always)]
    pub fn get(&self) -> &<V as Layer>::Output
    where
        for<'a> &'a Self: ConfigValueDescriptor,
        <V as Layer>::Error: fmt::Display,
    {
        <&Self as Layer>::get(&self)
    }

    /// Takes the ownership of the cached result.
    ///
    /// It returns `None` if the configuration value hasn't been read yet.
    pub fn take(&mut self) -> Option<Result<<V as Layer>::Output, <V as Layer>::Error>> {
        self.cached.take()
    }
}

impl<V: Layer + ConfigValueDescriptor> ConfigValueDescriptor for Cached<V> {
    #[inline]
    fn get_descriptor(&self) -> &VarDescriptor {
        self.var.get_descriptor()
    }
}

impl<'a, V> Layer for &'a Cached<V>
where
    V: Layer,
{
    type Output = &'a <V as Layer>::Output;
    type Error = CachedError<'a, <V as Layer>::Error>;

    fn try_get(&self) -> Result<Self::Output, Self::Error> {
        self.cached
            .get_or_init(|| self.var.try_get())
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

        let res = with_env([], || cached.try_get());
        assert_matches!(
            res,
            Err(CachedError(ReadVarError::Var(VarError::NotPresent)))
        );

        let res = with_env([(VAR_NAME, "random value")], || cached.try_get());
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

        let res = with_env([(VAR_NAME, "foobar")], || cached.try_get());
        assert_matches!(res, Err(CachedError(ReadVarError::Other(_))));

        let res = with_env([(VAR_NAME, "3")], || cached.try_get());
        assert_matches!(res, Err(CachedError(ReadVarError::Other(_))));
    }

    #[test]
    fn assert_cached_val() {
        const VAR_NAME: &str = "__TEST_CACHED_VALUE";
        let cached = TextVar::from_var_name(VAR_NAME).cached();

        let res = with_env([(VAR_NAME, "foo")], || cached.try_get());
        assert_matches!(res.map(|s| s.as_str()), Ok("foo"));

        let res = with_env([(VAR_NAME, "bar")], || cached.try_get());
        assert_matches!(res.map(|s| s.as_str()), Ok("foo"));
    }
}
