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

// Endpoint
pub struct Endpoint {
    method: String,
    uri: String,
    handler: fn(Request, Response),
    //middleware: Option<Vec<fn(Request,Response)>>,
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
}

impl Packcake {
    pub fn new(threads: usize) -> Packcake {
        Packcake {
            port: 2468,
            endpoints: HashMap::<String, Endpoint>::new(),
            //temp_uri: "".to_string(),
            thread_pool_size: threads,
        }
    }

    fn add_endpoint(&mut self, endpoint: Endpoint) {
        let key = format!("{} {}", endpoint.method, endpoint.uri);
        println!("Adding endpoint -> {key}");
        self.endpoints.insert(key, endpoint);
    }

    /// Set the port for the API
    pub fn port(mut self, port: usize) -> Packcake {
        self.port = port;
        self
    }

    #[allow(dead_code)]
    /// Adds a 'GET' endpoint
    ///
    /// # Arguments
    ///
    /// * `uri` -> The uri for the endpoint
    /// * `handler` -> The handler for request to this endpoint
    pub fn get(mut self, uri: &str, handler: fn(Request, Response)) -> Packcake {
        let endpoint = Endpoint{
            method: GET.to_string(),
            uri: uri.to_string(),
            handler,
        };
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
    pub fn post(mut self, uri: &str, handler: fn(Request, Response)) -> Packcake {
        let endpoint = Endpoint {
            method: POST.to_string(),
            uri: uri.to_string(),
            handler,
        };
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
    pub fn put(mut self, uri: &str, handler: fn(Request, Response)) -> Packcake {
        let endpoint = Endpoint {
            method: PUT.to_string(),
            uri: uri.to_string(),
            handler,
        };
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
    pub fn patch(mut self, uri: &str, handler: fn(Request, Response)) -> Packcake {
        let endpoint = Endpoint {
            method: PATCH.to_string(),
            uri: uri.to_string(),
            handler,
        };
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
    pub fn delete(mut self, uri: &str, handler: fn(Request, Response)) -> Packcake {
        let endpoint = Endpoint {
            method: DELETE.to_string(),
            uri: uri.to_string(),
            handler,
        };
        self.add_endpoint(endpoint);
        self
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
                    let handler = endpoint.unwrap().handler;
                    thread_pool.execute(move || {
                        handler(request,response);
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

