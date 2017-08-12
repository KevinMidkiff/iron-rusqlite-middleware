extern crate iron;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use std::error::Error;
use std::sync::Arc;

use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};
use std::path::{Path};

/// Pool of `SqliteConnectionManager` kept by the Iron middleware
pub type RusqlitePool = Arc<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>>;

/// Iron rusqlite middleware
pub struct RusqliteMiddleware {
    /// Pool of connections to SQLite through the rusqlite library
    pub pool: RusqlitePool
}

pub struct Value(RusqlitePool);

impl typemap::Key for RusqliteMiddleware { type Value = Value; }

impl RusqliteMiddleware {

    pub fn new<P: AsRef<Path>>(path: P) -> Result<RusqliteMiddleware, Box<Error>> {
        let config = r2d2::Config::default();
        let manager = r2d2_sqlite::SqliteConnectionManager::new(path);
        let pool = r2d2::Pool::new(config, manager)?;

        Ok(RusqliteMiddleware{ pool: Arc::new(pool) })
    }

    pub fn new_with_flags<P: AsRef<Path>>(path: P, flags: rusqlite::OpenFlags) -> Result<RusqliteMiddleware, Box<Error>> {
        let config = r2d2::Config::default();
        let manager = r2d2_sqlite::SqliteConnectionManager::new_with_flags(path, flags);
        let pool = r2d2::Pool::new(config, manager)?;

        Ok(RusqliteMiddleware{ pool: Arc::new(pool) })
    }

    pub fn get_connection(&self) -> r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager> {
        let poll = self.pool.clone();
        poll.get().unwrap()
    }
}

impl BeforeMiddleware for RusqliteMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<RusqliteMiddleware>(Value(self.pool.clone()));
        Ok(())
    }
}

pub trait RusqliteRequestExtension {
    fn database_connection(&self) -> r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;
}

impl<'a, 'b> RusqliteRequestExtension for Request<'a, 'b> {
    fn database_connection(&self) -> r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager> {
        let pv = self.extensions.get::<RusqliteMiddleware>().unwrap();  // Is this safe?
        let &Value(ref poll) = pv;
        poll.get().unwrap()
    }
}
