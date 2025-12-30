# mkenv

[![](https://img.shields.io/crates/v/mkenv?style=flat-square)](https://crates.io/crates/mkenv) [![](https://img.shields.io/docsrs/mkenv?style=flat-square)](https://docs.rs/mkenv)

mkenv is a lightweight Rust crate that helps you define the configuration your app needs.

Configurations are based on environment variables. The goal of this library is to make your app centralize the fetching of these environment variables. It greatly improves readability in your code.

Another benefit is that it allows you to fetch for all the environment variables at any time, e.g. on the program startup. This way, you may remove runtime errors when retrieving an invalid environment variable, because the error would be caught earlier. The library is designed to make the error message clear, specifying all the variables the program needs.

## Example

```rust
use std::time::Duration;
use mkenv::prelude::*;

// In debug mode
#[cfg(debug_assertions)]
mkenv::make_config! {
  struct DbConfig {
    db_url: {
      var_name: "DB_URL",
      description: "The DB URL",
    }
  }
}

// In release mode
#[cfg(not(debug_assertions))]
mkenv::make_config! {
  struct DbConfig {
    db_url: {
      var_name: "DB_URL_FILE",
      layers: [file_read()],
      description: "The path to the file containing the DB URL",
    }
  }
}

mkenv::make_config! {
  struct AppConfig {
    db: { DbConfig },
    user: { var_name: "USER" },
    request_timeout: {
      var_name: "REQUEST_TIMEOUT",
      layers: [
        parsed<Duration>(|input| {
          input.parse().map(Duration::from_millis).map_err(From::from)
        }),
        or_default_val(|| Duration::from_secs(5)),
        cached(),
      ]
    }
  }
}

let config = AppConfig::define();
// Optional: initialize the config, to early-panic if there is
// any missing/invalid environment variable.
config.init();

let timeout = config.request_timeout.get();
```

If the instruction with the `init()` call fails, an error similar to this is shown:
```txt
Error during configuration initialization:
Got 2 incorrect variables
- `DB_URL`: environment variable not found
- `REQUEST_TIMEOUT`: environment variable not found
Got 1 valid variable
- `USER`
Full required environment description:
- `DB_URL`: The DB URL
- `USER`
- `REQUEST_TIMEOUT`
```

You may find more complete examples [here](./examples), or read the [crate documentation](https://docs.rs/mkenv).
