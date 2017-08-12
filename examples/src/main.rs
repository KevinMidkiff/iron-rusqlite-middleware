extern crate iron;
extern crate iron_rusqlite_middleware;

use iron::prelude::*;
use iron::status;
use iron_rusqlite_middleware::{RusqliteMiddleware, RusqliteRequestExtension};

static CREATE_TODOS_TBL: &'static str = "
CREATE TABLE IF NOT EXISTS todos(
    id    INT PRIMARY KEY NOT NULL,
    task  TEXT            NOT NULL
);
";

static SELECT_TODOS: &'static str = "
SELECT * FROM todos;
";

struct ToDo {
    id: i32,
    task: String
}

fn list_todos(req: &mut Request) -> IronResult<Response> {
    // Get the rusqlite connection
    let conn = req.database_connection();
    // Get all of the users
    let mut stmt = match conn.prepare(SELECT_TODOS) {
        Ok(s) => s,
        Err(_) => return Ok(Response::with((status::InternalServerError))),
    };

    // Map the query results to strings
    let query = match stmt.query_map(&[], |row| { ToDo { id: row.get(0), task: row.get(1) } }) {
        Ok(q) => q,
        Err(_) => return Ok(Response::with((status::InternalServerError))),
    };

    // Process the query results
    let mut todos = String::new();
    for todo in query {
        match todo {
            Ok(t) => todos.push_str(&format!("{}: {}\n", t.id, t.task)),
            Err(_) => return Ok(Response::with((status::InternalServerError))),
        }
    }

    Ok(Response::with((status::Ok, todos)))
}

pub fn main() {
    let rusqlite_middleware = RusqliteMiddleware::new("example.db").unwrap();
    let conn = rusqlite_middleware.get_connection();
    conn.execute(CREATE_TODOS_TBL, &[]).unwrap();

    let mut chain = Chain::new(list_todos);
    chain.link_before(rusqlite_middleware);
    let addr = "127.0.0.1:3000";
    println!("-- Running webserver on {}", addr);
    Iron::new(chain).http(addr).unwrap();
}
