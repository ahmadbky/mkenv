use std::{
    error::Error as StdError,
    fmt,
    ops::{Deref, DerefMut},
    str::FromStr,
};

struct CapturedVarsInner<E> {
    vars: Vec<&'static str>,
    errs: Vec<Error>,
    _marker: std::marker::PhantomData<E>,
}

impl<E> Default for CapturedVarsInner<E> {
    fn default() -> Self {
        Self {
            vars: Vec::new(),
            errs: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct CapturedVars<E> {
    inner: CapturedVarsInner<E>,
}

impl<E> Default for CapturedVars<E> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

pub trait FmtReqEnv {
    fn fmt(f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

macro_rules! pluralize {
    ($x:expr) => {{
        if $x > 1 {
            "s"
        } else {
            ""
        }
    }};
}

impl<E: FmtReqEnv> fmt::Display for CapturedVarsInner<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Got {} valid variable{}",
            self.vars.len(),
            pluralize!(self.vars.len())
        )?;
        for var in &self.vars {
            writeln!(f, "- `{var}`")?;
        }
        writeln!(
            f,
            "Got {} incorrect variable{}",
            self.errs.len(),
            pluralize!(self.errs.len())
        )?;
        for err in &self.errs {
            writeln!(f, "- {err}")?;
        }
        f.write_str("Note: full required environment description:\n")?;
        <E as FmtReqEnv>::fmt(f)
    }
}

impl<E> fmt::Debug for CapturedVars<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CapturedVars")
            .field("vars", &self.inner.vars)
            .field("errs", &self.inner.errs)
            .finish()
    }
}

impl<E: FmtReqEnv> fmt::Display for CapturedVars<E> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

pub trait EnvResidual {
    type Residual;

    fn from_residual(_: Self::Residual) -> Self;
}

#[allow(dead_code)]
impl<E> CapturedVars<E> {
    pub fn capture<F>(f: F) -> Result<WithVars<E>, Self>
    where
        E: EnvResidual,
        F: FnOnce(&mut Self) -> <E as EnvResidual>::Residual,
    {
        let mut captured = Self::default();
        let residual = f(&mut captured);
        if captured.failed() {
            Err(captured)
        } else {
            Ok(captured.bind_captured_vars(EnvResidual::from_residual(residual)))
        }
    }

    pub fn captured_vars(&self) -> &[&str] {
        &self.inner.vars
    }

    pub fn errors(&self) -> &[Error] {
        self.inner.errs.as_slice()
    }

    pub fn bind_captured_vars(self, item: E) -> WithVars<E> {
        WithVars {
            item,
            vars: self.inner.vars,
        }
    }

    #[inline(always)]
    pub fn failed(&self) -> bool {
        !self.inner.errs.is_empty()
    }

    pub fn visit_env<T>(&mut self) -> Option<T>
    where
        T: Env,
    {
        T::get_env()
            .map(|WithVars { item, vars }| {
                self.inner.vars.extend(vars);
                item
            })
            .map_err(|captured| {
                self.inner.vars.extend(captured.inner.vars);
                self.inner.errs.extend(captured.inner.errs);
            })
            .ok()
    }

    pub fn visit_opt(&mut self, var: &'static str) -> Option<String> {
        self.inner
            .visit_opt(var)
            .inspect(|_| self.inner.vars.push(var))
    }

    pub fn visit(&mut self, var: &'static str) -> Option<String> {
        self.inner
            .visit_opt(var)
            .map(|x| {
                self.inner.vars.push(var);
                Some(x)
            })
            .unwrap_or_else(|| {
                self.inner.errs.push(Error::Missing(var));
                None
            })
    }

    pub fn visit_opt_as<T>(&mut self, var: &'static str) -> Option<T>
    where
        T: FromStr,
    {
        self.inner.visit_opt_as(var).flatten()
    }

    pub fn visit_as<T>(&mut self, var: &'static str) -> Option<T>
    where
        T: FromStr,
    {
        match self.inner.visit_opt_as(var) {
            Some(Some(v)) => Some(v),
            Some(None) => {
                self.inner.errs.push(Error::Missing(var));
                None
            }
            None => None,
        }
    }

    pub fn visit_read(&mut self, var: &'static str) -> Option<String> {
        match self.inner.visit_opt(var).map(std::fs::read_to_string) {
            Some(Ok(out)) => {
                self.inner.vars.push(var);
                Some(out)
            }
            other => {
                self.inner.errs.push(match other {
                    Some(Err(e)) => Error::Io(var, e),
                    None => Error::Missing(var),
                    Some(Ok(_)) => unreachable!(),
                });
                None
            }
        }
    }
}

impl<E> CapturedVarsInner<E> {
    #[inline(always)]
    fn visit_opt(&self, var: &str) -> Option<String> {
        std::env::var(var).ok()
    }

    fn visit_opt_as<T>(&mut self, var: &'static str) -> Option<Option<T>>
    where
        T: FromStr,
    {
        match self.visit_opt(var).map(|x| x.parse()) {
            Some(Ok(out)) => {
                self.vars.push(var);
                Some(Some(out))
            }
            None => Some(None),
            Some(Err(_)) => {
                self.errs.push(Error::Invalid {
                    var,
                    type_name: std::any::type_name::<T>(),
                });
                None
            }
        }
    }
}

pub trait EnvVar {
    const ENV_VAR: &'static str;
    const ENV_DESC: &'static str;
}

pub trait EnvSplitIncluded {
    type OnlyIncluded;
    type WithoutIncluded;

    fn split(self) -> (Self::OnlyIncluded, Self::WithoutIncluded);
}

pub trait Env: Sized + FmtReqEnv + EnvResidual + EnvSplitIncluded {
    fn get_env() -> Result<WithVars<Self>, CapturedVars<Self>>;

    fn get_with_vars() -> Result<Self, CapturedVars<Self>> {
        Self::get_env().map(|WithVars { item, .. }| item)
    }

    fn try_get() -> Result<Self, EnvError<Self>> {
        Self::get_with_vars().map_err(From::from)
    }

    fn get() -> Self {
        Self::try_get().unwrap_or_else(|e| panic!("{e}"))
    }
}

pub struct EnvError<E> {
    inner: CapturedVars<E>,
}

impl<E> AsRef<CapturedVars<E>> for EnvError<E> {
    #[inline(always)]
    fn as_ref(&self) -> &CapturedVars<E> {
        &self.inner
    }
}

impl<E> AsMut<CapturedVars<E>> for EnvError<E> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut CapturedVars<E> {
        &mut self.inner
    }
}

impl<E> Deref for EnvError<E> {
    type Target = CapturedVars<E>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<E> DerefMut for EnvError<E> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<E> From<CapturedVars<E>> for EnvError<E> {
    fn from(inner: CapturedVars<E>) -> Self {
        Self { inner }
    }
}

impl<E: FmtReqEnv> fmt::Display for EnvError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Invalid environment initialization:\n")?;
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<E> fmt::Debug for EnvError<E> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

pub struct WithVars<T> {
    item: T,
    vars: Vec<&'static str>,
}

#[derive(Debug)]
pub enum Error {
    Missing(&'static str),
    Invalid {
        var: &'static str,
        type_name: &'static str,
    },
    Io(&'static str, std::io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Missing(l0), Self::Missing(r0)) => l0 == r0,
            (
                Self::Invalid {
                    var: l_var,
                    type_name: l_type_name,
                },
                Self::Invalid {
                    var: r_var,
                    type_name: r_type_name,
                },
            ) => l_var == r_var && l_type_name == r_type_name,
            (Self::Io(l0, l1), Self::Io(r0, r1)) => l0 == r0 && l1.kind() == r1.kind(),
            _ => false,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Missing(var) => write!(f, "Missing `{var}`"),
            Error::Invalid { var, type_name } => write!(
                f,
                "`{var}` variable has an invalid data type for `{type_name}`"
            ),
            Error::Io(var, err) => write!(f, "IO error when reading `{var}`: {err}"),
        }
    }
}

impl StdError for Error {}

#[doc(hidden)]
pub mod __private {
    pub use std::fmt;
}

#[doc(hidden)]
#[macro_export]
macro_rules! make_env_impl {
    (@__default_span) => { "" };
    (@__default_span $_DEFAULT:ident) => {
        "[Default: {:?}] "
    };

    (@__check_ct_env $_ENV_VAR:literal $_ENV_DESC:literal $_DEFAULT:ident) => {};
    (@__check_ct_env $ENV_VAR:literal $ENV_DESC:literal) => {
        #[cfg(not(debug_assertions))]
        const _: () = {
            let _ = env!(
                $ENV_VAR,
                concat!("Missing env var `", $ENV_VAR, "`: ", $ENV_DESC)
            );
        };
    };

    (@__cap_var $captured:ident [$content:ty] normal ($method:ident)
            $field_ty:ident $DEFAULT:ident) => {{
        <$content>::$method($crate::make_env_impl!(@__cap_var $captured [$content] normal
            $field_ty $DEFAULT))
    }};

    (@__cap_var $captured:ident [$_content:ty] normal
            $field_ty:ident $DEFAULT:ident) => {{
        $captured.visit_opt(<$field_ty as $crate::EnvVar>::ENV_VAR)
            .unwrap_or_else(|| $DEFAULT.to_owned())
    }};

    (@__cap_var $captured:ident [$content:ty] normal ($method:ident)
            $field_ty:ident) => {{
        $crate::make_env_impl!(@__cap_var $captured [$content] normal $field_ty)
            .map(<$content>::$method)
    }};

    (@__cap_var $captured:ident [$_content:ty] normal
            $field_ty:ident) => {{
        $captured.visit(<$field_ty as $crate::EnvVar>::ENV_VAR)
    }};

    (@__cap_var $captured:ident [$content:ty] file ($method:ident)
            $field_ty:ident) => {{
        $crate::make_env_impl!(@__cap_var $captured [$content] file $field_ty)
            .map(<$content>::$method)
    }};

    (@__cap_var $captured:ident [$_content:ty] file
            $field_ty:ident) => {{
        $captured.visit_read(<$field_ty as $crate::EnvVar>::ENV_VAR)
    }};

    (@__cap_var $captured:ident [$content:ty] parse ($method:ident)
            $field_ty:ident $DEFAULT:ident) => {{
        <$content>::$method($crate::make_env_impl!(@__cap_var $captured [$content] parse
            $field_ty $DEFAULT))
    }};

    (@__cap_var $captured:ident [$_content:ty] parse
            $field_ty:ident $DEFAULT:ident) => {{
        $captured.visit_opt_as(<$field_ty as $crate::EnvVar>::ENV_VAR)
            .unwrap_or($DEFAULT)
    }};

    (@__cap_var $captured:ident [$content:ty] parse ($method:ident)
            $field_ty:ident) => {{
        $crate::make_env_impl!(@__cap_var $captured [$content] parse $field_ty)
            .map(<$content>::$method)
    }};

    (@__cap_var $captured:ident [$_content:ty] parse
            $field_ty:ident) => {{
        $captured.visit_as(<$field_ty as $crate::EnvVar>::ENV_VAR)
    }};

    (@__bound_var $field:ident $_DEFAULT:ident) => {{
        Some($field)
    }};

    (@__bound_var $field:ident) => {{
        $field
    }};
}

/// See the [crate documentation](crate) to see how to use this.
#[macro_export]
macro_rules! make_env {
    ($vis:vis $Name:ident $(includes [$($IncludeName:ident as $include_field:ident),*])?: $(
        $(#[cfg($($attr:tt)*)])?
        $field:ident: {
            id: $field_ty:ident($content:ty),
            kind: $kind:ident $(($method:ident))?,
            var: $ENV_VAR:literal,
            desc: $ENV_DESC:literal $(,)?
            $(default: $DEFAULT:ident $(,)?)?
        }
    ),* $(,)?) => {
        $(
            $(#[cfg($($attr)*)])?
            #[derive(Debug)]
            #[repr(transparent)]
            struct $field_ty;

            $(#[cfg($($attr)*)])?
            $crate::make_env_impl!(@__check_ct_env $ENV_VAR $ENV_DESC $($DEFAULT)?);

            $(#[cfg($($attr)*)])?
            impl $crate::EnvVar for $field_ty {
                const ENV_VAR: &'static str = $ENV_VAR;
                const ENV_DESC: &'static str = $ENV_DESC;
            }
        )*

        #[derive(Debug)]
        #[allow(dead_code)]
        $vis struct $Name {
            $($(
                #[allow(dead_code)]
                pub $include_field: $IncludeName,
            )*)?
            $(
                $(#[cfg($($attr)*)])?
                #[allow(dead_code)]
                pub $field: $content,
            )*
        }

        impl $crate::FmtReqEnv for $Name {
            fn fmt(__f: &mut $crate::__private::fmt::Formatter<'_>) -> $crate::__private::fmt::Result {
                $($(<$IncludeName as $crate::FmtReqEnv>::fmt(__f)?;)*)?
                $(
                    $(#[cfg($($attr)*)])?
                    writeln!(
                        __f,
                        concat!(
                            "- `{}`: ",
                            $crate::make_env_impl!(@__default_span $($DEFAULT)?), "{}"
                        ),
                        <$field_ty as $crate::EnvVar>::ENV_VAR,
                        $($DEFAULT ,)?
                        <$field_ty as $crate::EnvVar>::ENV_DESC
                    )?;
                )*
                Ok(())
            }
        }

        const _: () = {
            struct __Residual {
                $($(
                #[allow(dead_code)]
                    $include_field: Option<$IncludeName>,
                )*)?
                $(
                    $(#[cfg($($attr)*)])?
                    #[allow(dead_code)]
                    $field: Option<$content>,
                )*
            }

            struct __OnlyIncluded {$($(
                #[allow(dead_code)]
                $include_field: $IncludeName,
            )*)?}

            struct __WithoutIncluded {$(
                #[allow(dead_code)]
                $(#[cfg($($attr)*)])?
                $field: $content,
            )*}

            impl $crate::EnvSplitIncluded for $Name {
                type OnlyIncluded = __OnlyIncluded;
                type WithoutIncluded = __WithoutIncluded;

                fn split(self) -> (Self::OnlyIncluded, Self::WithoutIncluded) {
                    (
                        Self::OnlyIncluded {$($(
                            $include_field: self.$include_field,
                        )*)?},
                        Self::WithoutIncluded {$(
                            $(#[cfg($($attr)*)])?
                            $field: self.$field,
                        )*}
                    )
                }
            }

            impl $crate::EnvResidual for $Name {
                type Residual = __Residual;

                fn from_residual(__res: Self::Residual) -> Self {
                    Self {
                        $($(
                            $include_field: __res.$include_field.unwrap(),
                        )*)?
                        $(
                            $(#[cfg($($attr)*)])?
                            $field: __res.$field.unwrap(),
                        )*
                    }
                }
            }

            impl $crate::Env for $Name {
                fn get_env() -> Result<$crate::WithVars<Self>, $crate::CapturedVars<Self>> {
                    $crate::CapturedVars::capture(|__captured| {
                        $($(
                            let $include_field = __captured.visit_env();
                        )*)?

                        $(
                            $(#[cfg($($attr)*)])?
                            let $field = $crate::make_env_impl!(@__cap_var __captured
                                [$content] $kind $(($method))? $field_ty $($DEFAULT)?);
                        )*

                        __Residual {
                            $($(
                                $include_field,
                            )*)?
                            $(
                                $(#[cfg($($attr)*)])?
                                $field: $crate::make_env_impl!(@__bound_var $field $($DEFAULT)?),
                            )*
                        }
                    })
                }
            }
        };

    }
}

#[macro_export]
macro_rules! init_env {
    ($Env:path) => {
        <$Env as EnvSplitIncluded>::WithoutIncluded
    };
}
