//! Module containing the [`LayerExt`] utility trait.

use std::{error::Error, str::FromStr};

use crate::{
    layer::Layer,
    layers::{parsed::ParseFn, Cached, FileRead, OrDefault, Parsed},
};

/// Utility trait for building configuration value types.
///
/// ## Example
///
/// ```
/// # use mkenv::prelude::*;
/// # use std::time::Duration;
/// let my_config = TextVar::from_var_name("OPT_TIMEOUT_MS")
///   .parsed(|input| {
///     input.parse::<u64>()
///       .map(|ms| Duration::from_millis(ms))
///       .map_err(From::from)
///   })
///   .or_default_val(|| Duration::from_secs(3));
/// ```
pub trait LayerExt: Sized {
    /// Marks the configuration value to be cached.
    fn cached(self) -> Cached<Self>
    where
        Self: Layer,
    {
        Cached {
            var: self,
            cached: Default::default(),
        }
    }

    /// Marks the configuration value to be read from a file.
    fn file_read(self) -> FileRead<Self> {
        FileRead { var: self }
    }

    /// Marks the configuration value to be parsed, using the provided function.
    ///
    /// Note: if you wish to use the [`FromStr`] trait implementation for `T`, you may use the
    /// [`parsed_from_str`][1] method instead.
    ///
    /// [1]: LayerExt::parsed_from_str
    fn parsed<T>(self, parse_fn: ParseFn<T>) -> Parsed<T, Self>
    where
        Self: Layer,
    {
        Parsed {
            parse_fn,
            var: self,
        }
    }

    /// Marks the configuration value to be parsed, using the [`FromStr`] trait implementation.
    ///
    /// Note: if you wish to use a custom parsing function, you may use the [`parsed`][1] method
    /// instead.
    ///
    /// [1]: LayerExt::parsed
    fn parsed_from_str<T>(self) -> Parsed<T, Self>
    where
        Self: Layer,
        T: FromStr<Err: Error + 'static>,
    {
        self.parsed(|input| {
            input
                .parse::<T>()
                .map_err(|e| Box::new(e) as Box<dyn Error>)
        })
    }

    /// Marks the configuration value to fallback to a default value on read, using the provided
    /// function.
    fn or_default_val(self, default_fn: fn() -> <Self as Layer>::Output) -> OrDefault<Self>
    where
        Self: Layer,
    {
        OrDefault {
            var: self,
            default_fn,
        }
    }

    /// Marks the configuration value to fallback to the default value of the output on read.
    #[inline]
    fn or_default(self) -> OrDefault<Self>
    where
        Self: Layer<Output: Default>,
    {
        self.or_default_val(Default::default)
    }
}

impl<T: Layer> LayerExt for T {}
