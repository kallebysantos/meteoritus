use rocket::State;

use crate::Meteoritus;

#[post("/")]
pub fn creation_handler(meteoritus: &State<Meteoritus>) {
    // do resource creation

    if let Some(callback) = &meteoritus.on_creation {
        callback();
    }
}
