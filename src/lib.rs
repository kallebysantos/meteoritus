#![feature(trait_alias)]
#![feature(file_create_new)]

#[macro_use]
extern crate rocket;

pub use comet_vault::CometFile;
pub use comet_vault::CometVault;

use comet_vault::MeteorVault;
use handlers::file_info_handler;
use rocket::http::uri::Reference;
use rocket::Orbit;
use rocket::{
    data::ByteUnit,
    fairing::{self, Fairing, Info, Kind},
    http::Header,
    Build, Rocket,
};
use std::sync::Arc;

mod comet_vault;
mod handlers;
use handlers::{creation_handler, info_handler, termination_handler, upload_handler};

pub trait CometFn = Fn() + Send + Sync;

#[derive(Clone)]
pub struct Meteoritus {
    base_route: &'static str,
    max_size: ByteUnit,
    vault: Arc<dyn CometVault>,
    on_creation: Option<
        Arc<
            dyn Fn(&Rocket<Orbit>, &CometFile, &mut Reference) -> Result<(), &'static str>
                + Send
                + Sync,
        >,
    >,
    on_complete: Option<Arc<dyn Fn(&Rocket<Orbit>) + Send + Sync>>,
    on_termination: Option<Arc<dyn CometFn>>,
}

#[derive(Clone)]
pub struct MeteoritusBuilder {
    meteoritus: Meteoritus,
}

impl Default for Meteoritus {
    fn default() -> Self {
        Self {
            base_route: "/meteoritus",
            max_size: ByteUnit::Megabyte(5),
            vault: Arc::new(MeteorVault::new("./tmp/files")),
            on_creation: None,
            on_complete: None,
            on_termination: None,
        }
    }
}

impl Meteoritus {
    pub fn new() -> MeteoritusBuilder {
        MeteoritusBuilder {
            meteoritus: Self::default(),
        }
    }

    fn get_protocol_version() -> MeteoritusHeaders {
        MeteoritusHeaders::Version(&["1.0.0"])
    }

    fn get_protocol_resumable_version() -> MeteoritusHeaders {
        MeteoritusHeaders::Resumable("1.0.0")
    }

    fn get_protocol_extensions() -> MeteoritusHeaders {
        MeteoritusHeaders::Extensions(&["creation", "expiration", "termination"])
    }

    fn get_protocol_max_size(&self) -> MeteoritusHeaders {
        MeteoritusHeaders::MaxSize(self.max_size.as_u64())
    }
}

impl MeteoritusBuilder {
    pub fn build(&self) -> Meteoritus {
        self.meteoritus.to_owned()
    }

    pub fn mount_to(&mut self, base_route: &'static str) -> Self {
        self.meteoritus.base_route = base_route;
        self.to_owned()
    }

    pub fn with_temp_path(&mut self, temp_path: &'static str) -> Self {
        self.with_vault(MeteorVault::new(temp_path))
    }

    pub fn with_vault<V: CometVault + 'static>(&mut self, vault: V) -> Self {
        self.meteoritus.vault = Arc::new(vault);
        self.to_owned()
    }

    pub fn with_max_size(&mut self, size: ByteUnit) -> Self {
        self.meteoritus.max_size = size;
        self.to_owned()
    }

    pub fn on_creation<F>(&mut self, callback: F) -> Self
    where
        F: Fn(&Rocket<Orbit>, &CometFile, &mut Reference) -> Result<(), &'static str>
            + Send
            + Sync
            + 'static,
    {
        self.meteoritus.on_creation = Some(Arc::new(callback));
        self.to_owned()
    }

    pub fn on_complete<F>(&mut self, callback: F) -> Self
    where
        F: Fn(&Rocket<Orbit>) + Send + Sync + 'static,
    {
        self.meteoritus.on_complete = Some(Arc::new(callback));
        self.to_owned()
    }

    pub fn on_termination<F: CometFn + 'static>(&mut self, callback: F) -> Self {
        self.meteoritus.on_termination = Some(Arc::new(callback));
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
            file_info_handler,
            termination_handler,
            upload_handler,
        ];

        Ok(rocket
            .manage(self.to_owned())
            .mount(self.base_route, routes))
    }
}

enum MeteoritusHeaders {
    MaxSize(u64),
    Extensions(&'static [&'static str]),
    Version(&'static [&'static str]),
    Resumable(&'static str),
}

impl Into<Header<'_>> for MeteoritusHeaders {
    fn into(self) -> Header<'static> {
        match self {
            MeteoritusHeaders::MaxSize(size) => Header::new("Tus-Max-Size", size.to_string()),
            MeteoritusHeaders::Extensions(ext) => Header::new("Tus-Extension", ext.join(",")),
            MeteoritusHeaders::Version(ver) => Header::new("Tus-Version", ver.join(",")),
            MeteoritusHeaders::Resumable(ver) => Header::new("Tus-Resumable", ver),
        }
    }
}
