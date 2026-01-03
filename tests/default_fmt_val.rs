#[cfg(test)]
mod tests {
    use mkenv::make_config;
    use mkenv::prelude::*;

    #[test]
    fn test_default_fmt_val() {
        make_config! {
            struct MyConfig {
              url: {
                var_name: "MY_URL",
                description: "Some URL",
              }
            }
        }

        unsafe {
            std::env::set_var("MY_URL", "asd");
        }
        let config = MyConfig::define();
        config.init();
        assert_eq!(config.url.get(), "asd");
        assert_eq!(
            format!("{}", config.url.get_descriptor()),
            "`MY_URL`: Some URL"
        );

        let url = config.url.default_fmt_val("hi");
        assert_eq!(
            format!("{}", url.get_descriptor()),
            "`MY_URL`: Some URL (default: hi)"
        );
    }
}
