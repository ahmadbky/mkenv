use mkenv::{make_config, prelude::*};

make_config! {
    struct Foo {
        var_a: {
            var_name: "VAR_A"
        }
    }
}

make_config! {
    struct Bar {
        foo: { Foo },
        var_b: {
            var_name: "VAR_B"
        }
    }
}

fn main() {
    let bar = Bar::define();
    bar.init();
    println!("ok!");

    println!("{:?}", bar.foo.var_a.try_get());
    println!("{:?}", bar.var_b.try_get());
}
