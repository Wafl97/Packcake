use std::{fs, thread, time::Duration};
use crate::packcake::*;

#[path = "packcake/packcake.rs"] mod packcake;

fn main() {
    Packcake::new(16)
        .port(2121)
        .get("/api/v1", get_docs)
        .get("/api/v1/user", get_user)
        .post("/api/v1/user", post_user)
        .put("/api/v1/user", put_user)
        .delete("/api/v1/user", delete_user)
        .get("/api/v1/team", get_team)
        .post("/api/v1/team", post_team)
        .put("/api/v1/team", put_team)
        .delete("/api/v1/team", delete_team)
        .get("/sleep", sleep_for_5)
        .start();
}

fn get_docs(req: Request, mut res: Response) {
    let api_token = req.get_header("PackcakeToken");
    if api_token.is_none() {
        res.status(StatusCode::BadRequest);
        res.send("Missing API Token in headers. Please add the token under 'PackcakeToken: <api_token>'");
    }
    let body = req.get_body();
    println!("Body: {}", body);
    let docs = fs::read_to_string("static/doc.json").unwrap();
    res.status(StatusCode::Ok);
    res.json(&docs);
}

pub fn get_user(_request: Request, mut response: Response) {
    println!("get_user");
    response.header("Content-Type","Application/json");
    response.send("{\"message\":\"get_user\"}");
}

pub fn post_user(_request: Request, mut response: Response) {
    println!("post_user");
    response.json("{\"message\":\"post_user\"}");
}

pub fn put_user(_request: Request, mut response: Response) {
    println!("put_user");
    response.send("put_user");
}

pub fn delete_user(_request: Request, mut response: Response) {
    println!("delete_user");
    response.send("delete_user");
}

pub fn get_team(_request: Request, mut response: Response) {
    println!("get_team");
    response.send("get_team");
}

pub fn post_team(_request: Request, mut response: Response) {
    println!("post_team");
    response.send("post_team");
}

pub fn put_team(_request: Request, mut response: Response) {
    println!("put_team");
    response.send("put_team");
}

pub fn delete_team(request: Request, mut response: Response) {
    println!("delete_team");
    let user_id_key = "user_id";
    let user_id = request.get_param(user_id_key);
    if user_id.is_none() {
        println!("Missing user_id");
        response.status(StatusCode::BadRequest);
        response.send(format!("Missing param [{user_id_key}]").as_str());
        return;
    }
    println!("user_id: {}", user_id.unwrap());
    response.send("delete_team");
}

pub fn sleep_for_5(_request: Request, mut response: Response) {
    println!("Sleeping...");
    thread::sleep(Duration::from_secs(5));
    println!("Nap over");
    response.send("sleep_over");
}
