use rocket::State;

use crate::Meteoritus;

#[patch("/<id>")]
pub fn upload_handler(id: &str, meteoritus: &State<Meteoritus>) {
    let is_upload_complete = true;

    if is_upload_complete {
        if let Some(callback) = &meteoritus.on_complete {
            callback();
        };
        return;
    }

    // do patch update
    println!("Patching {}", id);
}
