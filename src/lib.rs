//! mkenv is a lightweight crate that helps you define the configuration your application uses.
//!
//! The configuration is based on environment variables, just like you would fetch them directly
//! with the [`mod@std::env`] module. mkenv provides utility items to help you centralize the
//! environment variables your program needs to run.
//!
//! One benefit of this design is to make your code cleaner when it's about reading an environment
//! variable. It helps you fetch it from anywhere in your code.
//!
//! Another benefit is, by centralizing the definitions of the configuration your app uses, you can
//! initialize the instance on startup (in fact, at any moment you want), and it will throw an error
//! if any environment variable is invalid.
//!
//! > Note: due to a design change, items from version 0.1 are deprecated. They will probably
//! > be removed in a future version.
//!
//! Better to see this through some examples:
//!
//! ## Examples
//!
//! The basic usage of this library is with the [`make_config!`] macro:
//!
//! ```no_run
//! use mkenv::{prelude::*};
//!
//! mkenv::make_config! {
//!   struct AppConfig {
//!     user: { var_name: "USER" },
//!     pwd: { var_name: "PWD" },
//!   }
//! }
//!
//! let config = AppConfig::define();
//! ```
//!
//! This example generates a struct with fields `user` and `pwd`, having types that allow you to
//! fetch environment variables returning a `String`.
//!
//! Later in your code, you may fetch for the `$USER` or `$PWD` environment variables like this:
//!
//! ```no_run
//! # use mkenv::{prelude::*};
//! # mkenv::make_config! {
//! #   struct AppConfig {
//! #     user: { var_name: "USER" },
//! #     pwd: { var_name: "PWD" },
//! #   }
//! # }
//! # let config = AppConfig::define();
//! let user = config.user.get();
//! let pwd = config.pwd.get();
//! ```
//!
//! You may also check for their availability at any time, e.g. when your program starts:
//!
//! ```no_run
//! # use mkenv::{prelude::*};
//! # mkenv::make_config! {
//! #   struct AppConfig {
//! #     user: { var_name: "USER" },
//! #     pwd: { var_name: "PWD" },
//! #   }
//! # }
//! fn main() {
//!   let config = AppConfig::define();
//!   config.init();
//! }
//! ```
//!
//! If the `init()` call fails, meaning it couldn't retrieve the required environment variables,
//! then a message similar to this is shown:
//!
//! ```txt
//! Cannot initialize environment:
//! Got 1 incorrect variable
//! - `PWD`: environment variable not found
//! Got 1 valid variable
//! - `USER`
//! Note: full required environment description:
//! - `USER`
//! - `PWD`
//! ```
//!
//! ## Features
//!
//! ### Layers
//!
//! Layers are an abstraction of the **way** in which an environment variable is retrieved. Indeed,
//! environment variables aren't simply texts (`String`s). They could be a number, a date, or even
//! be optional, or represent the content of a file.
//!
//! Layers are represented by the [`Layer`] trait. The base layer is [`TextVar`][1]. Its output
//! type is `String`, and it simply returns the content of the environment variable.
//!
//! You can wrap this base layer with another one, for example with [`file_read()`][2]. The output
//! type will still be `String`, but the value will be the content of the file pointed at the path
//! of the environment variable:
//!
//! ```no_run
//! # use mkenv::{make_config, prelude::*};
//! make_config! {
//!   struct ConfigWithLayers {
//!     file_content: {
//!       var_name: "FILE_PATH",
//!       layers: [file_read()],
//!     }
//!   }
//! }
//!
//! let config = ConfigWithLayers::define();
//! // Reads the content of $FILE_PATH
//! let content = config.file_content.get();
//! ```
//!
//! You may also parse the value of an environment variable, with the [`parsed()`][3] layer:
//!
//! ```no_run
//! # use mkenv::{make_config, prelude::*};
//! # use std::time::Duration;
//! make_config! {
//!   struct ConfigWithLayers {
//!     timeout: {
//!       var_name: "REQUEST_TIMEOUT",
//!       layers: [
//!         parsed<Duration>(|input| {
//!           input.parse::<u64>()
//!             .map(Duration::from_millis)
//!             .map_err(From::from)
//!         }),
//!       ],
//!     }
//!   }
//! }
//! ```
//!
//! You may have guessed it, but layers are designed to be combined together. This means you can
//! define a configuration value to be parsed from the content of a file:
//!
//! ```no_run
//! # use mkenv::{make_config, prelude::*};
//! make_config! {
//!   struct ConfigWithLayers {
//!     whatever_number: {
//!       var_name: "WHATEVER_NUMBER",
//!       layers: [
//!         file_read(),
//!         parsed_from_str<i32>(),
//!       ],
//!     }
//!   }
//! }
//! ```
//!
//! Find out more about layers in the [module documentation](crate::layers).
//!
//! ### Composable declarations
//!
//! The [`make_config!`] macro supports composable declarations, meaning including the declaration
//! of other configuration types into another. See the example:
//!
//! ```no_run
//! # use mkenv::{make_config, prelude::*};
//! make_config! {
//!   struct DbConfig {
//!     db_url: { var_name: "DB_URL" },
//!   }
//! }
//!
//! make_config! {
//!   struct AppConfig {
//!     db_config: { DbConfig },
//!   }
//! }
//! ```
//!
//! The `AppConfig` struct will have a field `db_config: DbConfig`. Its initialization will include
//! the one of the `DbConfig` struct, meaning all the environment variables the `DbConfig` struct
//! needs, are also needed by the `AppConfig` struct.
//!
//! You may also use the composable pattern for conditional purposes:
//!
//! ```no_run
//! # use mkenv::make_config;
//! #[cfg(debug_assertions)]
//! make_config! {
//!   struct DbConfig {
//!     db_url: { var_name: "DB_URL" },
//!   }
//! }
//! #[cfg(not(debug_assertions))]
//! make_config! {
//!   struct DbConfig {
//!     db_url: {
//!       var_name: "DB_URL_FILE",
//!       layers: [file_read()],
//!     },
//!   }
//! }
//!
//! make_config! {
//!   struct AppConfig {
//!     db_config: { DbConfig },
//!   }
//! }
//! ```
//!
//! ### Lightness
//!
//! The library is very light, it has **0** dependency!
//!
//! ## Migration from v0.1
//!
//! Please use the [`make_config!`] macro instead of the old [`make_env!`]. It generates much less
//! code, and will make your binaries lighter.
//!
//! ```no_run
//! mkenv::make_env! {AppEnv:
//!   db_url: {
//!     id: DbUrl(String),
//!     kind: normal,
//!     var: "DB_URL",
//!     desc: "The URL to the database",
//!   }
//! }
//! ```
//!
//! becomes:
//!
//! ```no_run
//! mkenv::make_config! {
//!   struct AppEnv {
//!     db_url: {
//!       var_name: "DB_URL",
//!       description: "The URL to the database",
//!     }
//!   }
//! }
//! ```
//!
//! The old `kind` key in the macro is transformed into a newer `layers` key:
//!
//! * The `normal` kind is the base layer type, no need to precise it.
//! * The `parse` kind is the new [`parsed()`][3] layer, or [`parsed_from_str()`][4].
//! * The `file` kind is the new [`file_read()`][2] layer.
//! * There is no more concept of "wrapping types" with the `parse` kind. You can already provide
//!   a custom parse function.
//! * Default values are now represented by a new layer: [`or_default_val()`][5] and
//!   [`or_default()`][6]. So there is no need to provide an identifier for the default value anymore,
//!   which was forcing to declare a const in the first place.
//! * Conditional compilation isn't supported anymore on field level. The reason for this is because
//!   the expansion would have been way more complex, especially for the [`Iter`][7] associated type
//!   of the [`ConfigInitializer`][8] trait (which implementation is generated by the new
//!   [`make_config!`] macro). However, conditional compilation is possible to perform, but at the
//!   macro call level, and using the composable pattern, [as mentioned before][9].
//! * New composable pattern is way more flexible. Included configurations are simple fields of the
//!   output struct, and you can declare them in the middle of other regular fields.
//! * A *critical* note about security: the new design doesn't need a way to "split" structs anymore
//!   (previously represented by the [`EnvSplitIncluded`] trait). This is because the previous
//!   design was storing the value of the environment variables *everytime*. Whereas in the new
//!   design, [caching has become its own layer][10]. By default, environment variables are fetched
//!   **only on call** (when calling the [`try_get()`][11] or [`get()`][12] methods of the [`Layer`]
//!   trait). In bulk, security is no longer a concern! 🎉
//!
//! [1]: crate::layers::TextVar
//! [2]: crate::LayerExt::file_read
//! [3]: crate::LayerExt::parsed
//! [4]: crate::LayerExt::parsed_from_str
//! [5]: crate::LayerExt::or_default_val
//! [6]: crate::LayerExt::or_default
//! [7]: crate::exec::ConfigInitializer::Iter
//! [8]: crate::exec::ConfigInitializer
//! [9]: #composable-declarations
//! [10]: crate::layers::Cached
//! [11]: crate::Layer::try_get
//! [12]: crate::Layer::get

#![cfg_attr(feature = "nightly", feature(doc_notable_trait))]

mod builder;
mod descriptor;
pub mod error;
pub mod exec;
mod layer;
pub mod layers;

mod macros;

#[cfg(test)]
pub(crate) mod tests;

pub use builder::LayerExt;
pub use descriptor::{ConfigDescriptor, ConfigValueDescriptor, VarDescriptor};
pub use layer::Layer;

/// Utility module importing the most relevant types and traits.
///
/// It is meant to be imported like this: `use mkenv::prelude::*;`
pub mod prelude {
    pub use super::{
        ConfigDescriptor as _, ConfigValueDescriptor as _, Layer as _, LayerExt as _,
        exec::ConfigInitializer as _, layers::*,
    };
}

#[doc(hidden)]
pub mod __private {
    pub use super::macros::make_config_impl;
    pub use std::{fmt, iter};
}

#[deprecated(
    since = "1.0.0",
    note = "please refer to the crate documentation to use the new API"
)]
mod imp;

#[allow(deprecated)]
pub use imp::*;
