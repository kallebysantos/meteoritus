#![feature(trait_alias)]

#[macro_use]
extern crate rocket;

use std::sync::Arc;

use rocket::{
    fairing::{self, Fairing, Info, Kind},
    http::{ContentType, Status},
    request::{self, FromRequest, Outcome},
    response::Responder,
    Build, Request, Response, Rocket, State,
};

pub trait CometFn = Fn() + Send + Sync;

#[derive(Clone)]
pub struct Meteoritus {
    base: &'static str,
    on_creation: Option<Arc<dyn CometFn>>,
    on_complete: Option<Arc<dyn CometFn>>,
    on_termination: Option<Arc<dyn CometFn>>,
}

impl Meteoritus {
    pub fn new() -> Self {
        Meteoritus::default()
    }

    pub fn with_base(&mut self, base: &'static str) -> Self {
        self.base = base;
        self.to_owned()
    }

    pub fn on_creation<F: CometFn + 'static>(&mut self, callback: F) -> Self {
        self.on_creation = Some(Arc::new(callback));
        self.to_owned()
    }

    pub fn on_complete<F: CometFn + 'static>(&mut self, callback: F) -> Self {
        self.on_complete = Some(Arc::new(callback));
        self.to_owned()
    }

    pub fn on_termination<F: CometFn + 'static>(&mut self, callback: F) -> Self {
        self.on_termination = Some(Arc::new(callback));
        self.to_owned()
    }
}

impl Default for Meteoritus {
    fn default() -> Self {
        Self {
            base: "/meteoritus",
            on_creation: None,
            on_complete: None,
            on_termination: None,
        }
    }
}

#[rocket::async_trait]
impl Fairing for Meteoritus {
    fn info(&self) -> Info {
        Info {
            name: "Meteoritus",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let routes = routes![
            info_handler,
            creation_handler,
            upload_handler,
            termination_handler
        ];

        Ok(rocket.manage(self.clone()).mount(self.base, routes))
    }
}

#[derive(Debug)]
pub struct InfoRequest<'a> {
    value: &'a str,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for InfoRequest<'_> {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let info = InfoRequest {
            value: "Hello From InfoRequest Guard",
        };

        println!("{:?}", req);
        Outcome::Success(info)
    }
}

pub struct InfoResponder {}

impl<'r> Responder<'r, 'static> for InfoResponder {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        Response::build()
            .raw_header("Tus-Version", "1.0.0")
            .raw_header("Tus-Max-Size", "5120")
            .raw_header("Tus-Extension", "creation,expiration,termination")
            .status(Status::NoContent)
            .ok()
    }
}

#[options("/")]
fn info_handler(_req: InfoRequest) -> InfoResponder {
    InfoResponder {}
}

#[post("/")]
fn creation_handler(meteoritus: &State<Meteoritus>) {
    // do resource creation

    if let Some(callback) = &meteoritus.on_creation {
        callback();
    }
}

#[patch("/<id>")]
fn upload_handler(id: &str, meteoritus: &State<Meteoritus>) {
    let is_upload_complete = true;

    if is_upload_complete {
        if let Some(callback) = &meteoritus.on_complete {
            callback();
        };
        return;
    }

    // do patch update
    println!("Patching {}", id);
}

#[delete("/")]
fn termination_handler(meteoritus: &State<Meteoritus>) {
    // do resources termination

    if let Some(callback) = &meteoritus.on_termination {
        callback();
    }
}
