use std::sync::Arc;

use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    response::{self, Responder},
    Orbit, Request, Rocket, State,
};

use crate::{HandlerContext, Meteoritus, Vault};

#[delete("/<id>")]
pub fn termination_handler(
    id: &str,
    req: TerminationRequest,
    vault: &State<Arc<dyn Vault>>,
    meteoritus: &State<Meteoritus<Orbit>>,
) -> TerminationResponder {
    match vault.terminate_file(id) {
        Err(_) => TerminationResponder::Failure,
        Ok(file) => {
            if let Some(callback) = &meteoritus.on_termination() {
                callback(HandlerContext {
                    rocket: req.rocket,
                    file_info: &file,
                });
            }

            TerminationResponder::Success
        }
    }
}

#[derive(Debug)]
pub struct TerminationRequest<'r> {
    rocket: &'r Rocket<Orbit>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TerminationRequest<'r> {
    type Error = &'static str;

    async fn from_request(
        req: &'r Request<'_>,
    ) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(TerminationRequest {
            rocket: req.rocket(),
        })
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
