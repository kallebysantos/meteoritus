use std::sync::Arc;

use rocket::{
    http::Status,
    response::{self, Responder},
    Orbit, Request, State,
};

use crate::{Meteoritus, Vault};

#[delete("/<id>")]
pub fn termination_handler(
    id: &str,
    vault: &State<Arc<dyn Vault>>,
    meteoritus: &State<Meteoritus<Orbit>>,
) -> TerminationResponder {
    match vault.terminate_file(id) {
        Err(_) => TerminationResponder::Failure,
        Ok(_) => {
            if let Some(callback) = &meteoritus.on_termination() {
                callback();
            }

            TerminationResponder::Success
        }
    }
}

pub enum TerminationResponder {
    Success,
    Failure,
}

impl<'r> Responder<'r, 'static> for TerminationResponder {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let meteoritus = req.rocket().state::<Meteoritus<Orbit>>().unwrap();

        let mut res = rocket::Response::build();

        res.header(meteoritus.get_protocol_resumable_version());

        match self {
            Self::Success => res.status(Status::NoContent),
            Self::Failure => res.status(Status::Gone),
        };

        res.ok()
    }
}
