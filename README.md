# Iron Rusqlite Middleware
[![Build Status](https://travis-ci.org/KevinMidkiff/iron-rusqlite-middleware.svg?branch=master)](https://travis-ci.org/KevinMidkiff/iron-rusqlite-middleware)

Middleware for the Iron framework for rust enabling database connections using rusqlite.
This library is inspired by the [Iron Diesel Middleware](https://github.com/darayus/iron-diesel-middleware).

## Adding to Cargo Project
To add this library to your cargo project add the following to your `Cargo.toml`.

```toml
[dependencies.iron_rusqlite_middleware]
git = "https://github.com/KevinMidkiff/iron-rusqlite-middleware"
```

## Usage
The following example showcases the major components needed for using this library in your Iron project.

```rust
// Include Iron crate with any needed "use" statements

// Include the middleware's crate
extern crate iron_rusqlite_middleware;
use iron_rusqlite_middleware::{RusqliteMiddleware, RusqliteRequestExtension};

fn handler(req: &mut Request) -> IronResult<Response> {
    // The SQLite database connection is added into the Request object through the extension
    let conn = req.database_connection();
    
    // Do your request
    
    Ok(Response::with((status::Ok, "Done."))
}

fn main() {
    // Initialize the middleware
    let rusqlite_middleware = RusqliteMiddleware::new("example.db").unwrap();
    
    // Create the Iron chain of middlewares
    let mut chain = Chain::new(handler);
    
    // Add in the middleware as before
    chain.link_before(rusqlite_middleware);
    
    // Start your Iron webserver
    let addr = "127.0.0.1:3000";
    println!("-- Running webserver on {}", addr);
    Iron::new(chain).http(addr).unwrap();
}
```

## License
This library is provided under an MIT license, and is provided WITHOUT WARRENTY.
