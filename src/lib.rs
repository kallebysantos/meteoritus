#![feature(trait_alias)]

#[macro_use]
extern crate rocket;

use rocket::{
    fairing::{self, Fairing, Info, Kind},
    Build, Rocket,
};
use std::sync::Arc;

mod handlers;
use handlers::{creation_handler, info_handler, termination_handler, upload_handler};

pub trait CometFn = Fn() + Send + Sync;

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
            creation_handler,
            info_handler,
            termination_handler,
            upload_handler,
        ];

        Ok(rocket.manage(self.clone()).mount(self.base, routes))
    }
}
