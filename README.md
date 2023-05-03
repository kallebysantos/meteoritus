<p align="center">
    <img src="assets/logo-boxed-rounded.png" width="150" />
</p>

<h1 align="center">Meteoritus</h1>

<div align="center">
<a href="https://github.com/kallebysantos/meteoritus/actions/workflows/build-and-test.yml" align="center">
    <img alt="CI Build and test" src="https://github.com/kallebysantos/meteoritus/actions/workflows/build-and-test.yml/badge.svg">
</a>

<a href="https://docs.rs/meteoritus" align="center">
    <img alt="docs.rs" src="https://img.shields.io/docsrs/meteoritus">
</a>

<a href="https://crates.io/crates/meteoritus" align="center">
    <img alt="Crates.io" src="https://img.shields.io/crates/v/meteoritus">
</a>

<a href="https://rocket.rs" align="center">
    <img alt="Rocket Homepage" src="https://img.shields.io/badge/web-rocket.rs-red?style=flat&label=https&colorB=d33847">
</a>

[![Crates.io](https://img.shields.io/crates/d/meteoritus)](https://crates.io/crates/meteoritus)
[![Crates.io](https://img.shields.io/crates/l/meteoritus)](https://crates.io/crates/meteoritus)
</div>

#### A tus server integration for [Rocket framework](https://rocket.rs/).

### Getting started:
Meteoritus is a `Fairing` that implements tus protocol on top of [`Rocket`](https://rocket.rs) framework, so in order to use it you'll need the following dependencies in `Cargo.toml`:

#### Current version v0.2.0 [See changelog](https://github.com/kallebysantos/meteoritus/blob/main/CHANGELOG.md)
```toml
[dependencies]
rocket = "0.5.0-rc.2"
meteoritus = "0.2.0"
```

Then attach `Meteoritus` to your `Rocket` server on launch:

```rust
#[macro_use] extern crate rocket;
use rocket::data::ByteUnit;
use meteoritus::Meteoritus;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
let meteoritus = Meteoritus::new()
        .mount_to("/api/files")
        .with_temp_path("./tmp/uploads")
        .with_max_size(ByteUnit::Gibibyte(1))
          .on_creation(|ctx| {
                println!("on_creation: {:?}", ctx);
                Ok(())
           })
          .on_created(|ctx| {
                println!("on_created: {:?}", ctx);
           })
          .on_completed(|ctx| {
               println!("on_completed: {:?}", ctx);
           })
          .on_termination(|ctx| {
               println!("on_termination: {:?}", ctx);
           })
        .build();
    
    rocket::build()
        .attach(meteoritus)
        .mount("/", routes![hello])
}
```
For more detailed information check out the complete [Api documentation](https://docs.rs/meteoritus/).
