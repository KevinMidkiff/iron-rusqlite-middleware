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

/// Pooled conenction to the SQLite database
pub type SqliteConnection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

/// Iron rusqlite middleware
pub struct RusqliteMiddleware {
    /// Pool of connections to SQLite through the rusqlite library
    pub pool: RusqlitePool
}

pub struct Value(RusqlitePool);

impl typemap::Key for RusqliteMiddleware { type Value = Value; }

impl RusqliteMiddleware {

    /// Creates a new pooled connection to the SQLite database using the default options `rusqlite`.
    /// The `path` should be the path to the SQLite database file on your system.
    ///
    /// See `rusqlite::Connection::open` for mode details.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<RusqliteMiddleware, Box<Error>> {
        let manager = r2d2_sqlite::SqliteConnectionManager::file(path);
        let pool = r2d2::Pool::builder().build(manager)?;

        Ok(RusqliteMiddleware{ pool: Arc::new(pool) })
    }

    /// Creates a new pooled connection to the SQLite database using the given rusqlite flags
    /// (i.e. `rusqlite::OpenFlags`). The `path` should be the path to the SQLite database file
    /// on your system.
    ///
    /// See `rusqlite::Connection::open_with_flags` for mode details.
    pub fn new_with_flags<P: AsRef<Path>>(path: P, flags: rusqlite::OpenFlags) -> Result<RusqliteMiddleware, Box<Error>> {
        let manager = r2d2_sqlite::SqliteConnectionManager::file(path).with_flags(flags);
        let pool = r2d2::Pool::builder().build(manager)?;

        Ok(RusqliteMiddleware{ pool: Arc::new(pool) })
    }

    /// Get a handle to a pooled connection for the SQLite database. This can be used to execute
    /// some SQL commands prior to launching your Iron webserver. An example would be creating
    /// tables if they do not currently exist in the database.
    pub fn get_connection(&self) -> SqliteConnection {
        let poll = self.pool.clone();
        poll.get().unwrap()
    }
}

/// Implementation of the `iron::BeforeMiddleware` trait to make this actually Iron middleware.
impl BeforeMiddleware for RusqliteMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        // Insert the the value into the request extensions
        req.extensions.insert::<RusqliteMiddleware>(Value(self.pool.clone()));
        Ok(())
    }
}

/// Trait which adds a method the `iron::Request` to enable the retrieval of a database connection.
///
/// ### Example
///
/// ```ignore
/// use iron_rusqlite_middleware::RusqliteMiddlewareExtension;
///
/// fn handler(req: &mut Request) -> IronResult<Response> {
///     let conn = req.database_connection();
///
///     // Do stuff with the rusqlite::Connection object
///
///     Ok(Response::with((status::Ok, "Done.")))
/// }
/// ```
pub trait RusqliteRequestExtension {
    /// Returns a pooled connection to the SQLite database. The connection is automatically returned
    /// to the connection pool when the pool connection is dropped.
    ///
    /// **Panics** if the `RusqliteMiddleware` has not been added to the Iron app, or if retrieving
    /// the database connection times out.
    fn database_connection(&self) -> r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;
}

/// Implementation of the `RusqliteRequestExention` for the `iron::Request`.
impl<'a, 'b> RusqliteRequestExtension for Request<'a, 'b> {
    fn database_connection(&self) -> SqliteConnection {
        let pv = self.extensions.get::<RusqliteMiddleware>().unwrap();  // Is this safe?
        let &Value(ref poll) = pv;
        poll.get().unwrap()
    }
}
