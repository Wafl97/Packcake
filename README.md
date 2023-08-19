# Packcake

Packcake is a simple framework for creating REST APIs in Rust.

Planned features:

- [x] Grouping endpoints with common routes
- [x] Option to add middleware to routes

## Using Packcake

````rust
use crate::packcake::*;

fn main() {
    let mut api = Packcake::new(4); // Use 4 threads
    api.port(2121);
    api._path("/api/v1", None, None, Some(Vec::from([
        group_e("/user", Vec::from([
            get("", get_user),
            post("", post_user),
            put("", put_user),
            delete("", delete_user)
        ])),
    ])));
    api.start();
}
````

The same can also be done by chaining the methods:

````rust
use packcake::Packcake;

fn main() {
    Packcake::new(4)
        .port(2048)
        .get("/api/v1/resource", get_resource)
        .post("/api/v1/resource", post_resource)
        .put("/api/v1/resource", put_resource)
        .delete("/api/v1/resource", delete_resource)
        .start();
}
````

## Creating a handler

````rust
use packcake::{Request,Response,StatusCodes};

fn post_resource(request: Request, mut response: Response) {
    let some_header: Option<&String> = request.get_header("SomeHeader");
    if some_header.is_none() {
        response.status(StatusCodes::BadRequest);
        response.send("Missing 'SomeHeader' in headers");
        return;
    }
    let body: &String = request.get_body();
    // use data in body
    response.status(StatusCodes::Created);
    response.json("{\"message\":\"resource created\"}");
}
````