use rocket::{Orbit, State};

use crate::Meteoritus;

#[delete("/")]
pub fn termination_handler(meteoritus: &State<Meteoritus<Orbit>>) {
    // do resources termination

    if let Some(callback) = &meteoritus.on_termination() {
        callback();
    }
}
