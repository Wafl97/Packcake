use std::collections::{HashMap};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use crate::packcake::tp::ThreadPool;

#[path = "./thread_pool.rs"] mod tp;

// Consts
#[allow(dead_code)]
pub const GET: &str      = "GET";
#[allow(dead_code)]
pub const POST: &str     = "POST";
#[allow(dead_code)]
pub const PUT: &str      = "PUT";
#[allow(dead_code)]
pub const PATCH: &str    = "PATCH";
#[allow(dead_code)]
pub const DELETE: &str   = "DELETE";

// Status codes
pub enum StatusCode {
    Ok,
    Created,
    NotFound,
    BadRequest,
}

impl StatusCode {
    fn to_str(&self) -> &str {
        match self {
            StatusCode::Ok => "HTTP/1.1 200 OK",
            StatusCode::Created => "HTTP/1.1 201 CREATED",
            StatusCode::NotFound => "HTTP/1.1 404 NOT FOUND",
            StatusCode::BadRequest => "HTTP/1.1 400 BAD REQUEST",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Middleware {
    action: fn(&Request, &mut Response) -> bool,
}

impl Middleware {
    pub fn new(action: fn(&Request, &mut Response) -> bool) -> Middleware {
        Middleware {
            action
        }
    }
    pub fn trigger(&self, request: &Request, response: &mut Response) -> bool {
        (self.action)(request, response)
    }
}

// Endpoint
pub struct Endpoint {
    method: String,
    uri: String,
    handler: fn(&Request, &mut Response),
    middleware: Option<Vec<Middleware>>,
}

impl Endpoint {
    fn set_middleware(&mut self, middleware: Option<Vec<Middleware>>) {
        self.middleware = middleware;
    }
}

// Endpoint Group
pub struct Group {
    uri: String,
    middleware: Option<Vec<Middleware>>,
    groups: Option<Vec<Group>>,
    endpoints: Option<Vec<Endpoint>>,
}

impl Group {
    pub fn new(uri: &str, middleware: Option<Vec<Middleware>>,
               groups: Option<Vec<Group>>, endpoints: Option<Vec<Endpoint>>) -> Group {
        Group {
            uri: String::from(uri),
            middleware,
            groups,
            endpoints
        }
    }

    fn append_middleware(&mut self, middleware: Option<Vec<Middleware>>, do_print: bool) {
        if middleware.is_none() {
            return;
        }
        if self.middleware.is_none() {
            self.middleware = middleware;
            return;
        }
        let middleware = middleware.unwrap();
        let self_middleware = self.middleware.as_mut().unwrap();
        for m in middleware {
            if do_print {
                println!("Using middleware for \"{}/**/*\"", self.uri);
            }
            self_middleware.push(m);
        }
    }

    fn unpack(self) -> (String, Option<Vec<Middleware>>, Option<Vec<Group>>, Option<Vec<Endpoint>>) {
        let uri: String = String::from(&self.uri);
        let mut middleware: Option<Vec<Middleware>> = None;
        if self.middleware.is_some() {
            middleware = Some(self.middleware.unwrap())
        }
        let mut groups: Option<Vec<Group>> = None;
        if self.groups.is_some() {
            groups = Some(self.groups.unwrap());
        }
        let mut endpoints: Option<Vec<Endpoint>> = None;
        if self.endpoints.is_some() {
            endpoints = Some(self.endpoints.unwrap());
        }
        (uri, middleware, groups, endpoints)
    }
}

// Request
pub struct Request {
    method: String,
    uri: String,
    params: HashMap<String,String>,
    headers: HashMap<String,String>,
    body: String,
}

impl Request {
    pub(crate) fn new(line: &str, headers: HashMap<String,String>, body: String) -> Request {
        //println!("{}",line);
        let split: Vec<&str> = line.split(" ").collect();
        let method = split.get(0).unwrap().to_owned();
        let path_split: Vec<&str> = split.get(1).unwrap().to_owned().split("?").collect();
        let path = path_split.get(0).unwrap().to_owned();
        let mut params: HashMap<_,_> = HashMap::new();
        if path_split.len() >= 2 {
            let queries: Vec<&str> = path_split.get(1).unwrap().to_owned().split("&").collect();
            for query in queries {
                let query_split: Vec<&str> = query.split("=").collect();
                if query_split.len() >= 2 {
                    let query_name = query_split.get(0).unwrap().to_owned().to_owned();
                    let query_value = query_split.get(1).unwrap().to_owned().to_owned();
                    //println!("?{query_name}={query_value}");
                    params.insert(query_name, query_value);
                }
            }
        }

        Request {
            method: method.to_string(),
            uri: path.to_string(),
            params,
            headers,
            body
        }
    }

    fn from_stream(stream: &TcpStream) -> Option<Request> {
        let mut header_map = HashMap::<String,String>::new();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut request = String::new();
        loop {
            let r = reader.read_line(&mut request).unwrap();
            if r < 3 { //detect empty line
                break;
            }
        }
        let mut content_length = 0;
        let mut headers = request.split("\n");
        let request_line = headers.next().unwrap();
        for header in headers {
            if header.len() > 3 {
                let mut split = header.split(":");
                let (key, value) =
                    (String::from(split.next().unwrap()), String::from(split.next().unwrap_or_default().trim()));
                header_map.insert(key,value);
            }

            if header.starts_with("Content-Length") {
                let line = header.split(":");
                for side in line {
                    if !(side.starts_with("Content-Length")) {
                        content_length = side.trim().parse::<usize>().unwrap(); //Get Content-Length
                    }
                }
            }
        }

        let mut body_buffer = vec![0; content_length]; //New Vector with size of Content
        reader.read_exact(&mut body_buffer).unwrap(); //Get the Body Content.
        let body = String::from_utf8(body_buffer).unwrap(); //Convert to body string
        if request_line.is_empty() {
            return None;
        }
        return Some(Request::new(request_line, header_map, body));
    }

    #[allow(dead_code)]
    pub(crate) fn display(&self) {
        println!("Request:\r\nMethod: {:#?},\r\nURI: {:#?},\r\nParams: {:#?},\r\nHeaders: {:#?},\r\nBody: {:#?}",
                 self.method, self.uri, self.params, self.headers, self.body);
    }

    pub fn get_header(&self, header: &str) -> Option<&String> {
        self.headers.get(header)
    }

    pub fn get_param(&self, param: &str) -> Option<&String> {
        self.params.get(param)
    }

    pub fn get_body(&self) -> &String {
        &self.body
    }
}

// Response
pub struct Response {
    stream: TcpStream,
    status: StatusCode,
    headers: HashMap<String,String>,
}

impl Response {
    fn from_stream(stream: TcpStream) -> Response {
        Response {
            stream,
            status: StatusCode::Ok,
            headers: HashMap::<String,String>::new(),
        }
    }

    pub fn header(&mut self, header: &str, value: &str) {
        self.headers.insert(header.to_string(), value.to_string());
    }

    pub fn status(&mut self, status: StatusCode) {
        self.status = status;
    }

    pub fn send(&mut self, message: &str) {
        let status = self.status.to_str();
        let length = message.len();
        self.headers.insert("Content-Length".to_string(),length.to_string());
        //self.header("Content-Length",length.to_string().as_str());
        let headers = self.headers.iter().map(|(h,v)| format!("{h}: {v}\r\n")).collect::<String>();
        //println!("Response:\r\nStatus: {:#?},\r\nHeaders: {:#?},\r\nBody: {:#?}", status, self.headers, message);
        let response = format!("{status}\r\n{headers}\r\n{message}");
        self.stream.write_all(response.as_bytes()).unwrap();
        self.stream.flush().unwrap();
    }

    pub fn json(&mut self, json: &str) {
        self.header("Content-Type","Application/json");
        self.send(json);
    }

    pub fn raw_stream(&self) -> &TcpStream {
        &self.stream
    }
}

// API (Packcake)
pub struct Packcake {
    pub port: usize,
    pub endpoints: HashMap<String, Endpoint>,
    //temp_uri: String,
    thread_pool_size: usize,
    do_print: bool,
}

impl Packcake {
    pub fn new(threads: usize) -> Packcake {
        Packcake {
            port: 2468,
            endpoints: HashMap::<String, Endpoint>::new(),
            //temp_uri: "".to_string(),
            thread_pool_size: threads,
            do_print: false,
        }
    }

    fn add_endpoint(&mut self, endpoint: Endpoint) {
        let key = format!("{} {}", endpoint.method, endpoint.uri);
        if self.do_print {
            println!("Adding endpoint -> {key}");
        }
        self.endpoints.insert(key, endpoint);
    }

    /// Set the port for the API
    pub fn port(mut self, port: usize) -> Packcake {
        self.port = port;
        self
    }

    #[allow(dead_code)]
    pub fn debug(mut self) -> Packcake {
        self.do_print = true;
        self
    }

    #[allow(dead_code)]
    /// Adds a 'GET' endpoint
    ///
    /// # Arguments
    ///
    /// * `uri` -> The uri for the endpoint
    /// * `handler` -> The handler for request to this endpoint
    pub fn get(mut self, uri: &str, handler: fn(&Request, &mut Response)) -> Packcake {
        let endpoint = get(uri, handler);
        self.add_endpoint(endpoint);
        self
    }

    #[allow(dead_code)]
    /// Adds a 'POST' endpoint
    ///
    /// # Arguments
    ///
    /// * `uri` -> The uri for the endpoint
    /// * `handler` -> The handler for request to this endpoint
    pub fn post(mut self, uri: &str, handler: fn(&Request, &mut Response)) -> Packcake {
        let endpoint = post(uri, handler);
        self.add_endpoint(endpoint);
        self
    }

    #[allow(dead_code)]
    /// Adds a 'PUT' endpoint
    ///
    /// # Arguments
    ///
    /// * `uri` -> The uri for the endpoint
    /// * `handler` -> The handler for request to this endpoint
    pub fn put(mut self, uri: &str, handler: fn(&Request, &mut Response)) -> Packcake {
        let endpoint = put(uri, handler);
        self.add_endpoint(endpoint);
        self
    }

    #[allow(dead_code)]
    /// Adds a 'PATCH' endpoint
    ///
    /// # Arguments
    ///
    /// * `uri` -> The uri for the endpoint
    /// * `handler` -> The handler for request to this endpoint
    pub fn patch(mut self, uri: &str, handler: fn(&Request, &mut Response)) -> Packcake {
        let endpoint = patch(uri, handler);
        self.add_endpoint(endpoint);
        self
    }

    #[allow(dead_code)]
    /// Adds a 'DELETE' endpoint
    ///
    /// # Arguments
    ///
    /// * `uri` -> The uri for the endpoint
    /// * `handler` -> The handler for request to this endpoint
    pub fn delete(mut self, uri: &str, handler: fn(&Request, &mut Response)) -> Packcake {
        let endpoint = delete(uri, handler);
        self.add_endpoint(endpoint);
        self
    }

    pub fn path(&mut self, uri: &str, middleware: Option<Vec<Middleware>>, groups: Option<Vec<Group>>, endpoints: Option<Vec<Endpoint>>) -> &Packcake {
        self._path(uri, middleware, groups, endpoints);
        self
    }

    fn _path(&mut self, uri: &str, middleware: Option<Vec<Middleware>>, groups: Option<Vec<Group>>, endpoints: Option<Vec<Endpoint>>) {
        if groups.is_some() {
            let groups = groups.unwrap();
            for mut g in groups {
                // Set the updated uri
                g.uri = format!("{}{}", uri, g.uri);
                // Set the middleware of the previous group
                g.append_middleware(middleware.clone(), self.do_print);
                self.__path(g);
            }
        }

        if endpoints.is_some() {
            for mut endpoint in endpoints.unwrap() {
                endpoint.set_middleware(middleware.clone());
                endpoint.uri = format!("{}{}", uri, endpoint.uri);
                self.add_endpoint(endpoint);
            }
        }
    }

    fn __path(&mut self, group: Group) {
        let (uri, middleware, groups, endpoints) = group.unpack();
        self._path(&uri, middleware, groups, endpoints)
    }

    pub fn start(&self) {
        println!("Starting server...");
        let thread_pool = ThreadPool::new(self.thread_pool_size);
        let listener = TcpListener::bind(format!("127.0.0.1:{}",self.port)).unwrap();
        //let pool = ThreadPool::new(self.pool_size);
        println!("Server listening on port {}", self.port);
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            //Handle
            let optional_request: Option<Request> = Request::from_stream(&stream);
            if optional_request.is_some() {
                let request = optional_request.unwrap();
                //request.display();
                let mut response = Response::from_stream(stream);
                let id = format!("{} {}", request.method, request.uri);
                let endpoint = self.endpoints.get(&id);
                if endpoint.is_some() {
                    let ep = endpoint.unwrap();
                    //let mut passed_middleware_check = false;
                    let middleware = ep.middleware.clone();
                    let handler = endpoint.unwrap().handler;
                    thread_pool.execute(move || {
                        let mut passed_middleware_check = true;
                        if middleware.is_some() {
                            for middleware in middleware.as_ref().unwrap() {
                                passed_middleware_check &= middleware.trigger(&request, &mut response);
                                if !passed_middleware_check {
                                    println!("Request for {} {} failed; did not pass middleware checks", request.method, request.uri);
                                    break;
                                }
                            }
                        }
                        if passed_middleware_check {
                            handler(&request, &mut response);
                        }
                    });
                } else {
                    println!("{} {} is not mapped", request.method, request.uri);
                    response.status(StatusCode::BadRequest);
                    response.send("Route is not mapped");
                }
            }
        }
        println!("Server shutting down...");
    }
}

pub fn get(uri: &str, handler: fn(&Request, &mut Response) -> ()) -> Endpoint {
    _get(uri, None, handler)
}
fn _get(uri: &str, middleware: Option<Vec<Middleware>>, handler: fn(&Request, &mut Response) -> ()) -> Endpoint {
    Endpoint {
        method: String::from(GET),
        uri: String::from(uri),
        handler,
        middleware,
    }
}

pub fn post(uri: &str, handler: fn(&Request, &mut Response) -> ()) -> Endpoint {
    _post(uri, None, handler)
}
fn _post(uri: &str, middleware: Option<Vec<Middleware>>, handler: fn(&Request, &mut Response) -> ()) -> Endpoint {
    Endpoint {
        method: String::from(POST),
        uri: String::from(uri),
        handler,
        middleware,
    }
}

pub fn put(uri: &str, handler: fn(&Request, &mut Response) -> ()) -> Endpoint {
    _put(uri, None, handler)
}
fn _put(uri: &str, middleware: Option<Vec<Middleware>>, handler: fn(&Request, &mut Response) -> ()) -> Endpoint {
    Endpoint {
        method: String::from(PUT),
        uri: String::from(uri),
        handler,
        middleware,
    }
}

pub fn patch(uri: &str, handler: fn(&Request, &mut Response) -> ()) -> Endpoint {
    _patch(uri, None, handler)
}
fn _patch(uri: &str, middleware: Option<Vec<Middleware>>, handler: fn(&Request, &mut Response) -> ()) -> Endpoint {
    Endpoint {
        method: String::from(PATCH),
        uri: String::from(uri),
        handler,
        middleware,
    }
}

pub fn delete(uri: &str, handler: fn(&Request, &mut Response) -> ()) -> Endpoint {
    _delete(uri, None, handler)
}
fn _delete(uri: &str, middleware: Option<Vec<Middleware>>, handler: fn(&Request, &mut Response) -> ()) -> Endpoint {
    Endpoint {
        method: String::from(DELETE),
        uri: String::from(uri),
        handler,
        middleware,
    }
}

#[allow(dead_code)]
pub fn group(uri: &str, middleware: Option<Vec<Middleware>>, groups: Option<Vec<Group>>, endpoints: Option<Vec<Endpoint>>) -> Group {
    Group::new(uri, middleware, groups, endpoints)
}

#[allow(dead_code)]
pub fn group_e(uri: &str, endpoints: Vec<Endpoint>) -> Group {
    Group::new(uri, None, None, Some(endpoints))
}

#[allow(dead_code)]
pub fn group_m(uri: &str, middleware: Vec<Middleware>) -> Group {
    Group::new(uri, Some(middleware), None, None)
}

#[allow(dead_code)]
pub fn group_g(uri: &str, groups: Vec<Group>) -> Group {
    Group::new(uri, None, Some(groups), None)
}

#[allow(dead_code)]
pub fn group_mg(uri: &str, middleware: Vec<Middleware>, groups: Vec<Group>) -> Group {
    Group::new(uri, Some(middleware), Some(groups), None)
}

#[allow(dead_code)]
pub fn group_me(uri: &str, middleware: Vec<Middleware>, endpoints: Vec<Endpoint>) -> Group {
    Group::new(uri, Some(middleware), None, Some(endpoints))
}

#[allow(dead_code)]
pub fn group_ge(uri: &str, groups: Vec<Group>, endpoints: Vec<Endpoint>) -> Group {
    Group::new(uri, None, Some(groups), Some(endpoints))
}

