#![doc(
    html_logo_url = "https://github.com/kallebysantos/meteoritus/raw/main/assets/logo-boxed-rounded.png"
)]
#![doc(
    html_favicon_url = "https://github.com/kallebysantos/meteoritus/raw/main/assets/favicon.ico"
)]
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
//! rocket = "0.5.1"
//! meteoritus = "0.2.1"
//! ```
//!
//! Then attach [`Meteoritus`] to your [`Rocket`] server on launch:
//!
//! ```rust,no_run
//! #[macro_use] extern crate rocket;
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
//!           .on_creation(|ctx| {
//!                 println!("on_creation: {:?}", ctx);
//!                 Ok(())
//!            })
//!           .on_created(|ctx| {
//!                 println!("on_created: {:?}", ctx);
//!            })
//!           .on_completed(|ctx| {
//!                println!("on_completed: {:?}", ctx);
//!            })
//!           .on_termination(|ctx| {
//!                println!("on_termination: {:?}", ctx);
//!            })
//!         .build();
//!
//!     rocket::build()
//!         .attach(meteoritus)
//!         .mount("/", routes![hello])
//! }
//! ```
//! [`Rocket`]: https://api.rocket.rs/v0.5/rocket/index.html
//! [`Fairing`]: https://api.rocket.rs/v0.5/rocket/fairing/index.html

/// These are public dependencies! Update docs if these are changed, especially
/// figment's version number in docs.

#[macro_use]
extern crate rocket;

use rocket::http::Header;

mod meteoritus;
pub use crate::meteoritus::Meteoritus;

mod fs;
pub use crate::fs::{
    Built, Completed, Created, FileInfo, Metadata, MetadataError, Terminated,
    Vault,
};

mod handlers;
pub use crate::handlers::HandlerContext;

/// Represents the tus protocol headers.
pub enum MeteoritusHeaders {
    MaxSize(u64),
    Extensions(&'static [&'static str]),
    Version(&'static [&'static str]),
    Resumable(&'static str),
}

impl Into<Header<'_>> for MeteoritusHeaders {
    fn into(self) -> Header<'static> {
        match self {
            MeteoritusHeaders::MaxSize(size) => {
                Header::new("Tus-Max-Size", size.to_string())
            }
            MeteoritusHeaders::Extensions(ext) => {
                Header::new("Tus-Extension", ext.join(","))
            }
            MeteoritusHeaders::Version(ver) => {
                Header::new("Tus-Version", ver.join(","))
            }
            MeteoritusHeaders::Resumable(ver) => {
                Header::new("Tus-Resumable", ver)
            }
        }
    }
}
