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
        env::set_var("TIMEOUT", "5000");
    }

    let config = MyConfig::define();
    let res = config.try_exec();
    if let Err(res) = res {
        eprintln!("error during env init: {res}");
    } else {
        println!("everything is fine");
    }
}
