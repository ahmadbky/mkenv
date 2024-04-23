use crate::{make_env, Env as _, Error};

#[test]
fn basic() {
    make_env! {DbEnv:
        db_url: {
            id: DbUrl(String),
            kind: normal,
            var: "DB_URL",
            desc: "The database URL",
        }
    }

    let db_env = DbEnv::try_get();
    assert!(db_env.is_err());
    let Err(e) = db_env else { unreachable!() };
    assert_eq!(e.errors(), [Error::Missing("DB_URL")]);
}
