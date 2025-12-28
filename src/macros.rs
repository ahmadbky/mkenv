#[doc(hidden)]
#[macro_export]
macro_rules! make_config_impl {
    // ---------------
    // --------------- field config -> type
    // ---------------

    (@__field_config_ty $Config:ty) => {
        $Config
    };

    (@__field_config_ty
        var_name: $_var_name:literal
        $(, $($rest:tt)*)?
    ) => {
        $crate::__private::make_config_impl!(@__field_config_ty_layers $($($rest)*)?)
    };

    (@__field_config_ty_layers layers: [] $($_rest:tt)*) => {
        $crate::readers::TextVar
    };

    (@__field_config_ty_layers layers: [$($layers_tt:tt)+] $($_rest:tt)*) => {
        $crate::__private::make_config_impl!(@__field_config_ty_layers_content $($layers_tt)+)
    };

    // Case where the `layers` key wasn't provided
    (@__field_config_ty_layers $($_rest:tt)*) => {
        $crate::readers::TextVar
    };

    (@__field_config_ty_layers_content
        $($func_ident:ident $(<$($func_gen:ty),* $(,)?>)? ($($func_content:tt)*)),* $(,)?
    ) => {
        $crate::__private::make_config_impl!(@__field_config_ty_layer
            [$([ $func_ident $(<$($func_gen),*>)? ($($func_content)*) ])*]
            $crate::readers::TextVar
        )
    };

    // ---------------
    // --------------- layer kind -> type
    // ---------------

    // End case
    (@__field_config_ty_layer [] $($wrapped:tt)* ) => {
        $($wrapped)*
    };

    (@__field_config_ty_layer [[cached()] $([$($rest:tt)*])*] $($wrapped:tt)* ) => {
        $crate::__private::make_config_impl!(@__field_config_ty_layer
            [$([$($rest)*])*]
            $crate::readers::Cached<$($wrapped)*>
        )
    };

    (@__field_config_ty_layer [[file_read()] $([$($rest:tt)*])*] $($wrapped:tt)* ) => {
        $crate::__private::make_config_impl!(@__field_config_ty_layer
            [$([$($rest)*])*]
            $crate::readers::FileRead<$($wrapped)*>
        )
    };

    (@__field_config_ty_layer [[parsed<$parse_ty:ty>($($_content:tt)*)] $([$($rest:tt)*])*] $($wrapped:tt)* ) => {
        $crate::__private::make_config_impl!(@__field_config_ty_layer
            [$([$($rest)*])*]
            $crate::readers::Parsed<$parse_ty, $($wrapped)*>
        )
    };

    (@__field_config_ty_layer [[parsed_from_str<$parse_ty:ty>()] $([$($rest:tt)*])*] $($wrapped:tt)* ) => {
        $crate::__private::make_config_impl!(@__field_config_ty_layer
            [$([$($rest)*])*]
            $crate::readers::Parsed<$parse_ty, $($wrapped)*>
        )
    };

    (@__field_config_ty_layer [[or_default_val($($_content:tt)*)] $([$($rest:tt)*])*] $($wrapped:tt)* ) => {
        $crate::__private::make_config_impl!(@__field_config_ty_layer
            [$([$($rest)*])*]
            $crate::readers::OrDefault<$($wrapped)*>
        )
    };

    (@__field_config_ty_layer [[or_default()] $([$($rest:tt)*])*] $($wrapped:tt)* ) => {
        $crate::__private::make_config_impl!(@__field_config_ty_layer
            [$([$($rest)*])*]
            $crate::readers::OrDefault<$($wrapped)*>
        )
    };

    // ---------------
    // --------------- field config -> construction
    // ---------------

    (@__field_config_def $Config:ty) => {{
        <$Config as $crate::ConfigDescriptor>::define()
    }};

    (@__field_config_def
        var_name: $var_name:literal
        $(, layers: [$($layers:tt)*])?
        $(, description: $description:literal)?
        $(, default_val_fmt: $default_val_fmt:literal)?
        $(,)?
    ) => {{
        let __config = $crate::readers::TextVar::from_var_name($var_name)
            $(.description($description))?
            $(.default_fmt_val($default_val_fmt))?;
        $crate::__private::make_config_impl!(@__field_config_def_layers __config $($($layers)*)?)
    }};

    (@__field_config_def_layers $binding:ident) => {
        $binding
    };

    // ---------------
    // --------------- layers -> method calls
    // ---------------

    (@__field_config_def_layers $binding:ident
        $head:ident $(<$($head_gen_content:ty)*>)? ($($head_content:tt)*)
        $(, $tail:ident $(<$($tail_gen_content:ty)*>)? ($($tail_content:tt)*))* $(,)?
    ) => {{
        let $binding = $crate::__private::make_config_impl!(@__field_config_def_layer $binding
            $head $(<$($head_gen_content)*>)? ($($head_content)*)
        );
        $crate::__private::make_config_impl!(@__field_config_def_layers $binding
            $($tail $(<$($tail_gen_content)*>)? ($($tail_content)*)),*
        )
    }};

    (@__field_config_def_layer $binding:ident cached()) => {
        $binding.cached()
    };

    (@__field_config_def_layer $binding:ident file_read()) => {
        $binding.file_read()
    };

    (@__field_config_def_layer $binding:ident parsed<$parse_ty:ty>($($parse_content:tt)*)) => {
        $binding.parsed::<$parse_ty>($($parse_content)*)
    };

    (@__field_config_def_layer $binding:ident parsed_from_str<$parse_ty:ty>()) => {
        $binding.parsed_from_str::<$parse_ty>()
    };

    (@__field_config_def_layer $binding:ident or_default_val($($or_default_val_content:tt)*)) => {
        $binding.or_default_val($($or_default_val_content)*)
    };

    (@__field_config_def_layer $binding:ident or_default()) => {
        $binding.or_default()
    };

    // ---------------
    // --------------- field kinds -> iter type
    // ---------------

    (@__field_kind $lt:lifetime [$($head_config:tt)*] $([$($tail_config:tt)*])*) => {
        $crate::__private::iter::Chain<
            $crate::__private::make_config_impl!(@__field_kind_ty $lt $($head_config)*),
            $crate::__private::make_config_impl!(@__field_kind $lt $([$($tail_config)*])*),
        >
    };


    (@__field_kind $lt:lifetime) => {
        $crate::__private::iter::Empty<$crate::exec::ExecResult<$lt>>
    };

    // ---------------
    // --------------- field kind -> iter type
    // ---------------

    (@__field_kind_ty $lt:lifetime $Config:ty) => {
        <<$Config as $crate::exec::ConfigExecutor>::Iter<$lt> as
            $crate::__private::iter::IntoIterator>::IntoIter
    };

    (@__field_kind_ty $lt:lifetime var_name $($_rest:tt)*) => {
        $crate::__private::iter::Once<$crate::exec::ExecResult<$lt>>
    };

    // ---------------
    // --------------- field kind -> iter calls
    // ---------------

    (@__field_kind_calls $self:ident [$head_field:ident: $($head_config:tt)*] $([$($tail:tt)*])*) => {
        $crate::__private::make_config_impl!(@__field_kind_call $self $head_field $($head_config)*)
            .chain($crate::__private::make_config_impl!(@__field_kind_calls $self $([$($tail)*])*))
    };

    (@__field_kind_calls $_self:ident) => {
        $crate::__private::iter::empty::<$crate::exec::ExecResult<'_>>()
    };

    // ---------------
    // --------------- field kind -> iter call
    // ---------------

    (@__field_kind_call $self:ident $field:ident $Config:ty) => {
        <$Config as $crate::exec::ConfigExecutor>::exec_raw(&$self.$field)
            .into_iter()
    };

    (@__field_kind_call $self:ident $field:ident var_name $($_rest:tt)*) => {
        $crate::__private::iter::once(
            $crate::exec::ExecResult {
                config: $self.$field.describe_config_val(),
                error: $self.$field.try_read_var().err().map(From::from),
            }
        )
    };
}

#[doc(hidden)]
pub use make_config_impl;

#[macro_export]
macro_rules! make_config {
    {
        $(#[$main_attr:meta])*
        $vis:vis struct $Name:ident {$(
            $(#[$field_attr:meta])*
            $field_vis:vis $field:ident: { $($field_config:tt)* }
        ),* $(,)?}
    } => {
        $(#[$main_attr])*
        $vis struct $Name {$(
            $(#[$field_attr])*
            $field_vis $field: $crate::__private::make_config_impl!(@__field_config_ty $($field_config)*),
        )*}

        const _: () = {
            #[automatically_derived]
            impl $crate::ConfigDescriptor for $Name {
                fn define() -> Self {
                    #[allow(unused_imports)]
                    use $crate::prelude::*;
                    Self {$(
                        $field: $crate::__private::make_config_impl!(@__field_config_def $($field_config)*)
                    ),*}
                }
            }

            #[automatically_derived]
            impl $crate::exec::ConfigExecutor for $Name {
                type Iter<'a> = $crate::__private::make_config_impl!(@__field_kind
                    'a $([$($field_config)*])*
                );

                fn exec_raw(&self) -> Self::Iter<'_> {
                    #[allow(unused_imports)]
                    use $crate::prelude::*;

                    $crate::__private::make_config_impl!(
                        @__field_kind_calls self $([ $field: $($field_config)* ])*
                    )
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn assert_result_iter_coherent_no_flattening() {
        make_config! {
            struct TestConfig {
                var_a: {
                    var_name: "VAR_A",
                },
                var_b: {
                    var_name: "VAR_B",
                },
                var_c: {
                    var_name: "VAR_C",
                    layers: [
                        parsed_from_str<i32>(),
                        or_default(),
                        cached(),
                    ],
                    description: "hello",
                },
            }
        }

        let config = TestConfig::define();
        let res = config.exec_raw();

        itertools::assert_equal(
            res.map(|res| res.config.var_name),
            ["VAR_A", "VAR_B", "VAR_C"],
        );
    }

    #[test]
    fn assert_result_iter_coherent_with_flattening() {
        make_config! {
            struct Foo {
                var_a: {
                    var_name: "VAR_A",
                },
                var_b: {
                    var_name: "VAR_B",
                },
            }
        }

        make_config! {
            struct Bar {
                foo: { Foo },
                var_c: {
                    var_name: "VAR_C",
                },
                var_d: {
                    var_name: "VAR_D",
                },
            }
        }

        let config = Bar::define();
        let res = config.exec_raw();

        itertools::assert_equal(
            res.map(|res| res.config.var_name),
            ["VAR_A", "VAR_B", "VAR_C", "VAR_D"],
        );
    }

    /// Contains declarations made with the macro, to make sure the code still compiles
    /// with some tweaks.
    #[cfg(debug_assertions)]
    #[allow(unexpected_cfgs, unused)]
    mod __assert_compiles {
        // empty
        make_config! {
            struct Foo0 {}
        }

        // pub struct
        make_config! {
            pub struct Foo1 {}
        }

        // pub field
        make_config! {
            pub struct Foo2 {
                pub bar: { Foo1 },
            }
        }

        // doc comment on struct
        make_config! {
            /// Hello
            struct Foo3 {
            }
        }

        // doc comment on field
        make_config! {
            struct Foo4 {
                /// Hello
                bar: { Foo1 },
            }
        }

        // attribute on struct
        make_config! {
            #[derive(Debug)]
            struct Foo5 {
            }
        }

        // cfg on macro
        #[cfg(not(feature = "foo"))]
        make_config! {
            struct Foo6 {
                bar: { Foo1 },
            }
        }
        // neg cfg on macro
        #[cfg(feature = "foo")]
        make_config! {
            struct Foo6 {
                bar: { Foo2 },
            }
        }

        // only specify var name
        make_config! {
            struct Foo7 {
                foo: {
                    var_name: "HI",
                }
            }
        }

        // layers in whatever order
        make_config! {
            struct Foo8 {
                foo: {
                    var_name: "HI",
                    layers: [
                        file_read(),
                        parsed<std::time::Duration>(|input| todo!()),
                        or_default(),
                        cached(),
                    ],
                }
            }
        }

        // flattened config in whatever field order
        make_config! {
            struct Foo9 {
                foo: { Foo8 },
                bar: {
                    var_name: "HEY",
                },
                foobar: { Foo8 },
            }
        }

        // provide description and default_fmt_val - without layers
        make_config! {
            struct Foo10 {
                foo: {
                    var_name: "HEY",
                    description: "hey",
                    default_val_fmt: "no default :(",
                }
            }
        }

        // provide description and default_fmt_val - with layers
        make_config! {
            struct Foo11 {
                foo: {
                    var_name: "HEY",
                    layers: [cached()],
                    description: "hey",
                    default_val_fmt: "no default :(",
                }
            }
        }

        // provide description alone
        make_config! {
            struct Foo12 {
                foo: {
                    var_name: "HEY",
                    description: "hey",
                }
            }
        }

        // provide default_val_fmt alone
        make_config! {
            struct Foo13 {
                foo: {
                    var_name: "HEY",
                    default_val_fmt: "no default :(",
                }
            }
        }

        // empty layers
        make_config! {
            struct Foo14 {
                foo: {
                    var_name: "HEY",
                    layers: []
                }
            }
        }
    }
}
