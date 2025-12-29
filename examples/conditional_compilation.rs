//! You can test this example by compiling it in both debug and release mode, and see that
//! it shows a different result, thanks to `#[cfg(...)]` attributes.

use mkenv::{make_config, prelude::*};

#[cfg(debug_assertions)]
make_config! {
    struct SessionKey {
        session_key: {
            var_name: "SESSION_KEY",
            description: "The session key",
        }
    }
}

#[cfg(not(debug_assertions))]
make_config! {
    struct SessionKey {
        session_key: {
            var_name: "SESSION_KEY",
            layers: [file_read()],
            description: "The path to the file containing the session key",
        }
    }
}

make_config! {
    struct Config {
        session_key: { SessionKey },
    }
}

fn main() {
    let config = Config::define();
    config.init();
}
