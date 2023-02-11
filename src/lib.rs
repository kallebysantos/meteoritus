#[macro_use]
extern crate rocket;

use rocket::{
    fairing::{self, Fairing, Info, Kind},
    Build, Rocket,
};

#[derive(Clone)]
pub struct Meteoritus {
    base: &'static str,
}

impl Meteoritus {
    pub fn new() -> Self {
        Meteoritus::default()
    }

    pub fn with_base(&mut self, base: &'static str) -> Self {
        self.base = base;

        self.to_owned()
    }
}

impl Default for Meteoritus {
    fn default() -> Self {
        Self {
            base: "/meteoritus",
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
        #[get("/")]
        fn hello_meteor() -> &'static str {
            "Hello from Meteoritus"
        }

        let meteor_build = rocket
            .manage(self.clone())
            .mount(self.base, routes![hello_meteor]);

        Ok(meteor_build)
    }
}
