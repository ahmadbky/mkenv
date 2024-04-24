//! `mkenv` is a lightweight Rust crate that provides the [`make_env!`] macro
//! used to generate a struct containing all the necessary environment context.
//! This allows to remove runtime errors when retrieving an environment variable that
//! doesn't exist, by capturing them all at the beginning of the program.
//! It is designed to raise an error with a clear message about all the variables
//! the application uses when the environment initialization fails.
//!
//! ## Example usage
//!
//! For each environment variable declaration, you need to provide at least these fields:
//! - `id`: The identifier of the associated struct.
//! This is generally the same identifier of the struct field but in CamelCase.
//! - `kind`: How to retrieve the environment variable.
//! - `var`: The name of the environment variable as a string literal.
//! - `desc`: A short description of it as a string literal.
//!
//! See the example below:
//!
//! ```
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
//! The above example will generate a struct defined as below:
//! ```
//! struct AppEnv {
//!   db_url: String,
//! }
//! ```
//!
//! This struct implements the [`Env`] trait which allows it to be instanciated via the
//! [`get`](Env::get) method. This method will fill the fields of this struct according
//! to the configuration made in the macro call.
//!
//! You also have other methods if you wish to trace the captured variables during the
//! construction of the struct.
//!
//! The idea is to use the output instance of this struct to initialize a static variable,
//! and use the latter to get the necessary variables from anywhere in your code.
//! A basic example would be to do (using the [`once_cell`](https://docs.rs/once_cell) crate):
//!
//! ```
//! # use once_cell::sync::Lazy; struct AppEnv; impl AppEnv { fn get() -> Self { Self } }
//! static ENV: Lazy<AppEnv> = Lazy::new(AppEnv::get);
//!
//! fn env() -> &'static AppEnv {
//!   &ENV
//! }
//! ```
//!
//! This way, you can access the `db_url` field from anywhere in your code with
//! `crate::env().db_url`.
//!
//! If the construction failed, meaning it couldn't retrieve the required environment variables,
//! then this error is shown:
//!
//! ```txt
//! Cannot initialize environment:
//! Got 0 valid variable
//! Got 1 incorrect variable
//! - Missing `DB_URL`
//! Note: full required environment description:
//! - `DB_URL`: The URL to the database
//! ```
//!
//! ## Features
//!
//! ### Many ways to get an environment variable
//!
//! #### Normally
//!
//! The macro obviously supports retrieving environment variable normally, meaning
//! as a `String` containing the value of the variable. This is done via the `normal` kind.
//!
//! #### By parsing
//!
//! The macro supports parsing using the [`FromStr`](std::str::FromStr) trait implementation of the target type.
//! For this, you need to provide the `parse` kind to the declaration of the environment variable.
//!
//! #### By reading the content of a file
//!
//! You can provide the `file` kind to the declaration, which will read the file at
//! the path given by the environment variable. For now, it is not possible to mix the `file` and `parse` kinds. The output of the `file` kind is a `String`, like the `normal` kind.
//!
//! ### Wrapping types
//!
//! The macro supports wrapping types, or types that can be constructed from a string or
//! a single value that can be parsed from a string. For this, you need to provide an argument
//! to the kind field, which is the method used to construct the wrapping type.
//! See the example below:
//!
//! ```
//! mkenv::make_env! {AppEnv:
//!   timeout: {
//!     id: Timeout(std::time::Duration),
//!     kind: parse(from_secs),
//!     var: "TIMEOUT",
//!     desc: "The duration of the timeout (in seconds)",
//!   }
//! }
//! ```
//!
//! ### Default values
//!
//! The macro supports optional variables, as long as you provide a default value.
//! This is done with the `default` field:
//!
//! ```
//! const DEFAULT_PORT: u16 = 3000;
//!
//! mkenv::make_env! {AppEnv:
//!   port: {
//!     id: Port(u16),
//!     kind: parse,
//!     var: "PORT",
//!     desc: "The port used by the application",
//!     default: DEFAULT_PORT,
//!   }
//! }
//! ```
//!
//! Note that the default value must be an identifier, meaning it should be defined as
//! a constant in advance. The type must implement Debug, because it is printed in case of errors.
//!
//! Also note that if the environment variable is present but the parsing failed, it will raise
//! an error even if it has a default value.
//!
//! ### Conditional compilation
//!
//! The macro supports conditional compilation attributes for the declaration of the environment
//! variables. It does not yet support any other attribute declared on it however.
//!
//! For example:
//!
//! ```ignore
//! mkenv::make_env! {DbEnv:
//!   #[cfg(debug_assertions)]
//!   db_url: {
//!     // ... in debug mode
//!   },
//!   #[cfg(not(debug_assertions))]
//!   db_url: {
//!     // ... in release mode
//!   },
//! }
//! ```
//!
//! ### Composable declarations
//!
//! The macro supports composable declarations, meaning including the declaration of other
//! environment types into another. See the example:
//!
//! ```ignore
//! mkenv::make_env! {DbEnv:
//!   db_url: {
//!     // ...
//!   }
//! }
//!
//! mkenv::make_env! {AppEnv includes [DbEnv as db_env]:
//!   port: {
//!     // ...
//!   }
//! }
//! ```
//!
//! The struct declarations will roughly be like so:
//!
//! ```ignore
//! struct DbEnv {
//!   db_url: ...,
//! }
//!
//! struct AppEnv {
//!   db_env: DbEnv,
//!   port: ...,
//! }
//! ```
//!
//! ### Security
//!
//! For security reasons, you most likely don't want to keep the value of an environment
//! variable in memory. In most of the cases, you want it to be dropped after it is used once,
//! for initialization purposes.
//!
//! To achieve this, you need to do the above composable pattern, by declaring the environment
//! variables you need to drop in a separate macro call:
//!
//! ```ignore
//! mkenv::make_env! {UsedOnce:
//!   sess_key: {
//!     // ...
//!   }
//! }
//!
//! mkenv::make_env! {AppEnv includes [UsedOnce as used_once]:
//!   // ...
//! }
//! ```
//!
//! Then, when constructing the struct, you can call the `split` method from
//! the [`EnvSplitIncluded`] trait. This will give you a tuple containing:
//! - A struct with the fields included in the `includes [...]` clause
//! - A struct with all the other fields, that can safely be kept in memory at runtime
//!
//! By splitting the struct in 2 pieces, you split the ownership in 2: you can drop the
//! first piece after doing your initialization process, and you can initialize a static variable
//! with the second piece:
//!
//! ```ignore
//! fn init_my_env() {
//!   let env = AppEnv::get();
//!   let (used_once, rest) = env.split();
//!   // do things with `used_once`, will be dropped at the end
//!   MY_STATIC_VAR.set(rest).unwrap_or_else(|_| panic!("wtf?"));
//! }
//! ```
//!
//! > But what is the type of the static variable?
//!
//! You can declare your static variable like so (using the `once_cell` crate as an example):
//!
//! ```ignore
//! static MY_STATIC_VAR: OnceCell<mkenv::init_env!(AppEnv)> = OnceCell::new();
//! ```
//!
//! ### Lightness
//!
//! The library is very light, it has **0** dependency!

mod imp;
#[cfg(test)]
mod tests;

pub use imp::*;
