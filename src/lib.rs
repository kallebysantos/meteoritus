#[macro_use]
extern crate rocket;

use rocket::{
    fairing::{self, Fairing, Info, Kind},
    Build, Rocket,
};

#[derive(Default, Clone)]
pub struct Meteoritus {}

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
            .mount("/meteor", routes![hello_meteor]);

        Ok(meteor_build)
    }
}
