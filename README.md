# Packcake

## Setup

You need to use the `packcake` crate and create a new `Packcake` object. In the constructor you can specify the number of threads the API will use. To start the API you need to run the `.start()` function on the API.

```rust
use packcake::Packcake;

fn main() {
	let api = Packcake::new(4); // use 4 threads for the API
	api.start();
}
```

## Add endpoints

Adding endpoints is done by calling functions named after the HTTP verbs (*get*, *post*, *put*, *patch*, and *delete*). This function take the path for the endpoint and the function which will handle the request.

```rust
use packcake::{Packcake,Request,Response};

fn main() {
	let mut api = Packcake::new(4); // use 4 threads for the API
	api.get("/user", my_get_func);
	api.start();
}

fn my_get_func(request: &Request, response: &mut Response) {
	// your logic here
}
```

## Adding groups of endpoints

For adding endpoints with similar paths, it is possible to add them in bulk. This is done with the `.path()` function on the API. is take 4 parameters.
1. The path that is common for the endpoints.
2. Any middleware that will be run before the endpoints.
3. Any inner groupings. Here you can nest additional groups of endpoints.
4. All the endpoints that you want in this groupsing.

```rust
use packcake::{Packcake,Request,Response,get};

fn main() {
	let mut api = Packcake::new(4); // Use 4 threads for the API
	api.path("/api/v1", 
			None, // No middleware 
			None, // No nested groups
			Some(Vec::from([
				get("", my_get_func),
				// post(...), put(...), patch(...), delete(...)
			])));
	api.start();
}

fn my_get_func(request: &Request, response: &mut Response) {
	// your endpoint logic
}
```

## Adding middleware

We can add middleware that will be run before the handler. This is done on the groups of endpoints.

```rust
use packcake::{Packcake,Request,Response,get,Middleware};

fn main() {
	let mut api = Packcake::new(4); // Use 4 threads for the API
	api.path("/api/v1", 
			Some(Vec::from([
				Middleware::new(my_middleware),
				// Additional middleware
			])), 
			None, // No nested groups
			Some(Vec::from([
				get("", my_get_func),
				// post(...), put(...), patch(...), delete(...)
			])));
	api.start();
}

fn my_get_func(request: &Request, response: &mut Response) {
	// your endpoint logic
}

fn my_middleware(request: &Request, response: &mut Response) -> bool {
	// return true if request passes
	true
}
```

## Nesting groups

We can add nested groups to other groups. Here we can use different functions to specify what we want from the group:

1. ``group()`` -> middleware, inner groups, endpoints
2. ``group_m()`` -> middleware
3. ``group_me()`` -> middleware, endpoints
4. ``group_mg()`` -> middleware, inner groups
5. ``group_g()`` -> inner groups
6. ``group_ge()`` -> inner groups, endpoints
7. ``group_e()`` -> endpoints

```rust
use packcake::{Packcake,Request,Response,get,Middleware,group_e};

fn main() {
	let mut api = Packcake::new(4); // Use 4 threads for the API
	api.path("/api", 
			Some(Vec::from([
				Middleware::new(my_middleware),
				// Additional middleware
			])), 
			Some(Vec::from([
				group_e("/v1", Vec::from([
					get("", my_get_func),
					// post(...), put(...), patch(...), delete(...)
				])),
				// Additional groups
			])),
			None // No endpoints
		);
	api.start();
}

fn my_get_func(request: &Request, response: &mut Response) {
	// your endpoint logic
}

fn my_middleware(request: &Request, response: &mut Response) -> bool {
	// return true if request passes
	true
}
```
## Writing the endpoint handlers

The handlers take a `&Request` and a `&mut Response` as input. These provide tools for getting information from the request and responding accordingly. The same applies to writing middleware functions.

```rust
use packcake::{Requet,Response,StatusCode};

fn my_get_func(request: &Request, response: &mut Response) {
	let token = request.get_header("Token"); // Optional<&String>
	let number_query = request.get_param("number"); // Optional<&String>
	let body = request.get_body(); // &String
	response.header("MyHeader", "packcake");
	response.status(StatusCode::Ok);
	response.send("Response message"); // This should be the last thing
}
```
