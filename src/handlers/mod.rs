mod creation;
mod file_info;
mod info;
// mod termination;
mod upload;

pub use creation::creation_handler;
pub use file_info::file_info_handler;
pub use info::info_handler;
use rocket::{Orbit, Rocket};
// pub use termination::termination_handler;
pub use upload::upload_handler;

use crate::fs::FileInfo;

/// Represents the context of a file upload handler.
///
/// It contains a reference to the [`Rocket`] instance and a reference to the [`FileInfo`] struct,
/// which contains information about the uploaded file and its current state.
#[derive(Debug)]
pub struct HandlerContext<'a, S> {
    pub rocket: &'a Rocket<Orbit>,
    pub file_info: &'a FileInfo<S>,
}
