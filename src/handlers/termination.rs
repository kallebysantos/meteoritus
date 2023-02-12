use rocket::State;

use crate::Meteoritus;

#[delete("/")]
pub fn termination_handler(meteoritus: &State<Meteoritus>) {
    // do resources termination

    if let Some(callback) = &meteoritus.on_termination {
        callback();
    }
}
