use std::{env, sync::Mutex};

macro_rules! assert_matches {
    ($left:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        match $left {
            $( $pattern )|+ $( if $guard )? => {}
            ref left => {
                let right = stringify!($($pattern)|+ $(if $guard)?);
                panic!(
                    r#"assertion `left matches right` failed
  left: {left:?}
 right: {right:?}"#);
            }
        }
    };
}

pub(crate) use assert_matches;

/// Locks the environment to execute the provided function.
///
/// This is to avoid having the environment being accessed by two tests at the same time.
pub(crate) fn with_env<'a, I, F, R>(defs: I, f: F) -> R
where
    F: FnOnce() -> R,
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    static LOCK: Mutex<()> = Mutex::new(());

    let iter = defs.into_iter();
    let mut vars_to_unset = Vec::with_capacity(iter.size_hint().0);

    let _guard = LOCK.lock().unwrap();

    for (key, value) in iter {
        unsafe {
            env::set_var(key, value);
        }
        vars_to_unset.push(key);
    }

    let r = f();

    for key in vars_to_unset {
        unsafe {
            env::remove_var(key);
        }
    }

    r
}
