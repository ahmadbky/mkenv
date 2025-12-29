use std::{env, time::Duration};

use mkenv::{make_config, prelude::*};

make_config! {
    struct MyConfig {
        user: { var_name: "USER" },
        timeout: {
            var_name: "REQUEST_TIMEOUT",
            layers: [
                parsed<Duration>(|input| {
                    input.parse::<u64>().map(Duration::from_millis).map_err(From::from)
                }),
            ],
        }
    }
}

fn main() {
    // SAFETY: we're in a single thread program.
    unsafe {
        env::set_var("REQUEST_TIMEOUT", "foobar");
    }

    let config = MyConfig::define();
    config.init();
    println!("everything is fine");
}
