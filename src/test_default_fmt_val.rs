#[cfg(test)]
mod tests {
    use crate::make_config;
    use crate::prelude::*;

    make_config! {
        struct MyConfig {
          url: {
            var_name: "MY_URL",
            description: "Some URL",
          }
        }
    }

    #[test]
    fn test_default_fmt_val() {
        unsafe {
            std::env::set_var("MY_URL", "asd");
        }
        let config = MyConfig::define();
        config.init();
        assert_eq!(config.url.get(), "asd");
        assert_eq!(config.url.get_descriptor().description, Some("Some URL"));
        assert_eq!(config.url.get_descriptor().var_name, "MY_URL");
        assert_eq!(config.url.get_descriptor().default_val_fmt, None);

        let url = config.url.default_fmt_val("hi");
        assert_eq!(url.get_descriptor().default_val_fmt, Some("hi"));
        assert_eq!(url.get_descriptor().description, Some("Some URL"));
        assert_eq!(url.get_descriptor().var_name, "MY_URL");
    }
}
