#![doc(
    html_logo_url = "https://github.com/kallebysantos/meteoritus/raw/main/assets/logo-boxed-rounded.png"
)]
#![doc(
    html_favicon_url = "https://github.com/kallebysantos/meteoritus/raw/main/assets/favicon.ico"
)]
#![feature(trait_alias)]
#![feature(file_create_new)]
#![feature(type_changing_struct_update)]

//!  # Meteoritus - API Documentation
//!
//! Hello, and welcome to the Meteoritus API documentation!
//!
//! ## Usage
//!
//! Meteoritus is a [`Fairing`] that implements tus protocol on top of [`Rocket`] framework, so in
//! order to use it you'll need the following dependencies in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rocket = "0.5.0-rc.2"
//! meteoritus = "0.1.0"
//! ```
//!
//! Then attach [`Meteoritus`] to your [`Rocket`] server on launch:
//!
//! ```rust,no_run
//! #[macro_use]
//! extern crate rocket;
//! use rocket::data::ByteUnit;
//! use meteoritus::Meteoritus;
//!
//! #[get("/")]
//! fn hello() -> &'static str {
//!     "Hello, world!"
//! }
//!
//! #[launch]
//! fn rocket() -> _ {
//!     let meteoritus = Meteoritus::new()
//!         .mount_to("/api/files")
//!         .with_temp_path("./tmp/uploads")
//!         .with_max_size(ByteUnit::Gibibyte(1))
//!         .on_creation(|rocket, file, upload_uri| {
//!              println!("File created: {:?}", file);
//!              Ok(())
//!          })
//!         .on_complete(|rocket| {
//!              println!("Upload complete!");
//!          })
//!         .build();
//!
//!     rocket::build()
//!         .attach(meteoritus)
//!         .mount("/", routes![hello])
//! }
//! ```
//! [`Rocket`]: https://api.rocket.rs/v0.5-rc/rocket/index.html
//! [`Fairing`]: https://api.rocket.rs/v0.5-rc/rocket/fairing/index.html

/// These are public dependencies! Update docs if these are changed, especially
/// figment's version number in docs.

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
