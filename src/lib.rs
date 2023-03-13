#![feature(trait_alias)]
#![feature(file_create_new)]
#![feature(type_changing_struct_update)]

#[macro_use]
extern crate rocket;

use rocket::http::Header;

mod meteoritus;
pub use meteoritus::Meteoritus;

mod comet_vault;
pub use comet_vault::{CometFile, CometVault};

mod handlers;

pub trait CometFn = Fn() + Send + Sync;

pub enum MeteoritusHeaders {
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
