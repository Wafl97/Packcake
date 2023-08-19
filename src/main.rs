use std::{fs, thread, time::Duration};
use crate::packcake::*;

#[path = "packcake/packcake.rs"] mod packcake;

fn main() {
    Packcake::new(4)
        .port(2121)
        .debug()
        .path("/api",
               Some(Vec::from([
                   Middleware::new(middleware_v1),
                   Middleware::new(middleware_api)])), Some(Vec::from([
            group_ge("/v1", Vec::from([
                group_mg("/protected",
                         Vec::from([Middleware::new(middleware_auth)]),
                         Vec::from([
                             group_e("/team", Vec::from([
                                 get("", get_team),
                                 post("", post_team),
                                 put("", put_team),
                                 delete("", delete_team)
                             ])),
                             group_me("/user", Vec::from([
                                 Middleware::new(middleware_auth)
                             ]),Vec::from([
                                 get("", get_user),
                                 post("", post_user),
                                 put("", put_user),
                                 delete("", delete_user)
                             ]))
                         ]))
            ]), Vec::from([
                get("", get_docs)
            ]))
        ])), None)
        .start();
}

fn middleware_api(_request: &Request, _response: &mut Response) -> bool {
    println!("(/api)");
    /*let token = request.get_header("Token");
    if token.is_none() {
        response.status(StatusCode::BadRequest);
        response.send("Missing Token in request header");
        return false;
    }*/
    true
}

fn middleware_v1(_request: &Request, _response: &mut Response) -> bool {
    println!("(/v1)");
    true
}

fn middleware_auth(_request: &Request, _response: &mut Response) -> bool {
    println!("(/team)");
    true
}

fn get_docs(req: &Request, res: &mut Response) {
    /*let api_token = req.get_header("PackcakeToken");
    if api_token.is_none() {
        res.status(StatusCode::BadRequest);
        res.send("Missing API Token in headers. Please add the token under 'PackcakeToken: <api_token>'");
    }*/
    let body = req.get_body();
    println!("Body: {}", body);
    let docs = fs::read_to_string("static/doc.json");
    if docs.is_ok() {
        res.status(StatusCode::Ok);
        res.json(&docs.unwrap());
    }
    res.status(StatusCode::NotFound);
    res.send("helllllo");
}

pub fn get_user(_request: &Request, response: &mut Response) {
    println!("get_user");
    response.header("Content-Type","Application/json");
    response.send("{\"message\":\"get_user\"}");
}

pub fn post_user(_request: &Request, response: &mut Response) {
    println!("post_user");
    response.json("{\"message\":\"post_user\"}");
}

pub fn put_user(_request: &Request, response: &mut Response) {
    println!("put_user");
    response.send("put_user");
}

pub fn delete_user(_request: &Request, response: &mut Response) {
    println!("delete_user");
    response.send("delete_user");
}

pub fn get_team(_request: &Request, response: &mut Response) {
    println!("get_team");
    response.send("get_team");
}

pub fn post_team(_request: &Request, response: &mut Response) {
    println!("post_team");
    response.send("post_team");
}

pub fn put_team(_request: &Request, response: &mut Response) {
    println!("put_team");
    response.send("put_team");
}

pub fn delete_team(request: &Request, response: &mut Response) {
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

pub fn sleep_for_5(_request: &Request, response: &mut Response) {
    println!("Sleeping...");
    thread::sleep(Duration::from_secs(5));
    println!("Nap over");
    response.send("sleep_over");
}
