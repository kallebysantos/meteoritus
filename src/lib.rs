use rocket::fairing::Fairing;

struct Meteoritus {}

impl Fairing for Meteoritus {
    fn info(&self) -> rocket::fairing::Info {
        todo!()
    }
}
