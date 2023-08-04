# Packcake

Packcake is a simple framework for creating REST APIs in Rust.

Planned features:

- Grouping endpoints with common routes
- Option to add middleware to routes

## Using Packcake

````rust
use packcake::Packcake;

fn main() {
    let api = Packcake::new();
    api.port(2048); // Default port is 2468
    
    // Add endpoints and handlers for the endpoints
    api.get("/api/v1/resource", get_resource);
    api.post("/api/v1/resource", post_resource);
    api.put("/api/v1/resource", put_resource); // .patch is also available
    api.delete("/api/v1/resource", delete_resource);    
    
    // Run the API
    api.start();
}
````

The same can also be done by chaining the methods:

````rust
use packcake::Packcake;

fn main() {
    Packcake::new()
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