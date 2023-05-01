use std::{error::Error, marker::PhantomData, sync::Arc};

use rocket::{
    data::ByteUnit,
    fairing::{self, Fairing, Info, Kind},
    Build, Ignite, Orbit, Phase, Rocket,
};

use crate::{
    fs::Terminated,
    handlers::{
        creation_handler, file_info_handler, info_handler, termination_handler,
        upload_handler,
    },
};

#[allow(unused_imports)]
use crate::{
    fs::{Built, Completed, Created, LocalVault, Metadata},
    handlers::HandlerContext,
    MeteoritusHeaders, Vault,
};

/// The tus fairing itself.
///
/// # Phases
///
/// A [`Meteoritus`] instance represents a tus middleware and its state. It progresses
/// through three statically-enforced phases: [`Build`], [`Ignite`], [`Orbit`].
///
/// * **Build**: _middleware configuration_
///
///   This phase enables:
///
///> * setting mount route and configuration options like: temp path and max upload size
///> * registering callbacks for events
/* ///# > * adding custom implementation for [`Vault`] */
///
///> This is the _only_ phase in which an instance can be modified. To finalize changes,
///> an instance is ignited via [` Meteoritus::build()`], progressing it into the <i>ignite</i>
///> phase, then it should be attached with  [`rocket::Rocket::attach()`] in order to be launched into orbit.
///
/// * **Ignite**: _finalization of configuration_
///
///   An instance in the [`Ignite`] phase is in its final configuration.
///   Barring user-supplied interior mutation, application state is guaranteed
///   to remain unchanged beyond this point.
///   An instance in the ignite phase can be launched into orbit to serve tus requests via [`rocket::Rocket::attach()`].
///
/// * **Orbit**: _a running tus middleware_
///
///   An instance in the [`Orbit`] phase represents a _running_ middleware,
///   actively serving requests.
///
/// # Launching
///
/// In order to launch a [`Meteoritus`] middleware an instance of [`Meteoritus<Ignite>`] _must_ be
/// attached to [`Rocket`] server using [`rocket::Rocket::attach()`]:
///
///   ```rust,no_run
///   # #[macro_use] extern crate rocket;
///   use rocket::{Build, Ignite};
///   use meteoritus::Meteoritus;
///
///   #[launch]
///   fn rocket() -> _ {
///       let meteoritus: Meteoritus<Build> = Meteoritus::new();
///     
///       let meteoritus: Meteoritus<Ignite> = meteoritus.build();
///
///       rocket::build().attach(meteoritus)
///   }
///   ```
///
/// This generates all tus endpoints needed to handle tus requests
///
/// * **Launching with custom options**
///
/// Since [`Meteoritus<Build>`]implements the _builder pattern_ it exports public methods
/// to customize the middleware behavior, like registering event callbacks and custom configuration:
///
///   ```rust,no_run
///   # #[macro_use] extern crate rocket;
///   use rocket::{Build, Ignite, data::ByteUnit};
///   use meteoritus::{Built, Completed, Created, Terminated, HandlerContext, Meteoritus};
///   # //
///   # // use std::io::Result;
///   # // use meteoritus::{Vault, CometFile};
///   # // pub struct MyCustomVault {}
///   # //
///   # // impl MyCustomVault {
///   # //     pub fn new() -> Self {
///   # //         Self {}
///   # //     }
///   # // }
///   # //
///   # // impl Vault for MyCustomVault {
///   # //     fn add(&self, file: &CometFile) -> Result<()> {
///   # //         // Save file information on some persistent storage
///   # //         todo!()
///   # //     }
///   # //
///   # //     fn take(&self, id: String) -> Result<CometFile> {
///   # //         // Get the file information from persistent storage
///   # //         todo!()
///   # //     }
///   # //
///   # //     fn remove(&self, file: &CometFile) -> Result<()> {
///   # //         // Remove file information and all data from persistent storage
///   # //         todo!()
///   # //     }
///   # //
///   # //     fn update(&self, file: &mut CometFile, buf: &mut [u8]) -> std::io::Result<()> {
///   # //         // Patch the file content based on current offset
///   # //         todo!()
///   # //     }
///   # // }
///
///   #[launch]
///   fn rocket() -> _ {
///       let meteoritus: Meteoritus<Ignite> = Meteoritus::new()
///           .mount_to("/api/files")
///           .with_temp_path("./tmp/uploads")
///   # //           .with_vault(MyCustomVault::new())
///           .with_max_size(ByteUnit::Gibibyte(1))
///           .on_creation(|ctx: HandlerContext<Built>| {
///                 println!("on_creation: {:?}", ctx);
///                 Ok(())
///            })
///           .on_created(|ctx: HandlerContext<Created>| {
///                 println!("on_created: {:?}", ctx);
///            })
///           .on_completed(|ctx: HandlerContext<Completed>| {
///                println!("on_completed: {:?}", ctx);
///            })
///           .on_termination(|ctx: HandlerContext<Terminated>|{
///                println!("on_termination: {:?}", ctx);
///             })
///           .build();
///     
///       rocket::build().attach(meteoritus)
///   }
///   ```
#[derive(Clone)]
pub struct Meteoritus<P: Phase> {
    auto_terminate: bool,
    base_route: &'static str,
    max_size: ByteUnit,
    vault: Arc<dyn Vault>,
    on_creation: Option<
        Arc<
            dyn Fn(HandlerContext<Built>) -> Result<(), Box<dyn Error>>
                + Send
                + Sync,
        >,
    >,
    on_created: Option<Arc<dyn Fn(HandlerContext<Created>) + Send + Sync>>,
    on_completed: Option<Arc<dyn Fn(HandlerContext<Completed>) + Send + Sync>>,
    on_termination:
        Option<Arc<dyn Fn(HandlerContext<Terminated>) + Send + Sync>>,
    state: std::marker::PhantomData<P>,
}

impl<P: Phase> Meteoritus<P> {
    pub fn get_protocol_version(&self) -> MeteoritusHeaders {
        MeteoritusHeaders::Version(&["1.0.0"])
    }

    pub fn get_protocol_resumable_version(&self) -> MeteoritusHeaders {
        MeteoritusHeaders::Resumable("1.0.0")
    }

    pub fn get_protocol_extensions(&self) -> MeteoritusHeaders {
        MeteoritusHeaders::Extensions(&["creation", "termination"])
    }

    pub fn get_protocol_max_size(&self) -> MeteoritusHeaders {
        MeteoritusHeaders::MaxSize(self.max_size.as_u64())
    }
}

impl Meteoritus<Build> {
    /// Returns a instance of [`Meteoritus`] into the _[`Build`]_ phase.
    pub fn new() -> Meteoritus<Build> {
        Meteoritus::<Build> {
            auto_terminate: true,
            base_route: "/meteoritus",
            max_size: ByteUnit::Megabyte(5),
            vault: Arc::new(LocalVault::new("./tmp/files")),
            on_creation: Default::default(),
            on_created: Default::default(),
            on_completed: Default::default(),
            on_termination: Default::default(),
            state: PhantomData::<Build>,
        }
    }

    /// Returns a instance of [`Meteoritus`] into the _[`Ignite`]_ phase.
    pub fn build(self) -> Meteoritus<Ignite> {
        Meteoritus::<Ignite> {
            state: std::marker::PhantomData,
            ..self
        }
    }

    /// Optional configuration that specifies if completed uploads should be keep on disk or deleted.
    ///
    /// By default Meteoritus will assumes that an `on_completed` callback has assigned to move uploads to a permanent location and will auto-terminate all completed uploads.
    pub fn keep_on_disk(mut self) -> Self {
        self.auto_terminate = false;
        self
    }

    /// Mounts all tus middleware routes in the supplied given `base` path.
    ///
    /// # Panics
    ///
    /// Panics if either:
    ///   * the `base` mount point is not a valid static path: a valid origin
    ///     URI without dynamic parameters.
    ///
    ///   * any route's URI is not a valid origin URI.
    ///
    ///     **Note:** _This kind of panic is guaranteed not to occur if the routes
    ///     were generated using Rocket's code generation._
    ///
    /// # Examples
    ///
    /// Manually create a route path mounted at base
    /// `"/api/files"`. Requests to the `/api/files` URI will be dispatched to the
    /// [`Meteoritus`] middleware.
    ///
    ///   ```rust,no_run
    ///   # #[macro_use] extern crate rocket;
    ///   use rocket::Ignite;
    ///   use meteoritus::Meteoritus;
    ///
    ///   #[launch]
    ///   fn rocket() -> _ {
    ///       let meteoritus: Meteoritus<Ignite> = Meteoritus::new()
    ///           .mount_to("/api/files")
    ///           .build();
    ///     
    ///       rocket::build().attach(meteoritus)
    /// }
    /// ```
    /// **Note:** [`Meteoritus`] will mount many tus protocol routes based on the specified path.
    pub fn mount_to(mut self, base_route: &'static str) -> Self {
        self.base_route = base_route;
        self
    }

    /// Directory to store temporary files.
    ///
    /*# **Note:** If a custom [`Vault`] has provided then the [`Meteoritus`] will ignore
    ///# the supplied `temp_path`.*/
    ///
    /// # Examples
    ///
    /// Assign relative `tmp/uploads` to store uploaded files into.
    ///
    ///   ```rust,no_run
    ///   # #[macro_use] extern crate rocket;
    ///   use rocket::Ignite;
    ///   use meteoritus::Meteoritus;
    ///
    ///   #[launch]
    ///   fn rocket() -> _ {
    ///       let meteoritus: Meteoritus<Ignite> = Meteoritus::new()
    ///           .with_temp_path("./tmp/uploads")
    ///           .build();
    ///     
    ///       rocket::build().attach(meteoritus)
    /// }
    /// ```
    pub fn with_temp_path(self, temp_path: &'static str) -> Self {
        self.with_vault(LocalVault::new(temp_path))
    }

    #[doc(hidden)]
    /// Overrides the default instance of [`Vault`].
    ///
    /// If a custom vault has provided then the [`Meteoritus`] will ignore the [`Meteoritus::with_temp_path()`]
    /// configuration. Since it assumes that all file system operations will be responsibility of
    /// the custom vault implementation.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # // #[macro_use] extern crate rocket;
    /// # // use std::io::Result;
    /// # // use rocket::Ignite;
    /// # // use meteoritus::{Meteoritus, Vault, FileInfo};
    /// # //
    /// # // pub struct MyCustomVault {}
    /// # //
    /// # // impl MyCustomVault {
    /// # //     pub fn new() -> Self {
    /// # //         Self {}
    /// # //     }
    /// # // }
    /// # //
    /// # // impl Vault for MyCustomVault {
    /// # //     fn add(&self, file: &CometFile) -> Result<()> {
    /// # //         // Save file information on some persistent storage
    /// # //         todo!()
    /// # //     }
    /// # //
    /// # //     fn take(&self, id: String) -> Result<CometFile> {
    /// # //         // Get the file information from persistent storage
    /// # //         todo!()
    /// # //     }
    /// # //
    /// # //     fn remove(&self, file: &CometFile) -> Result<()> {
    /// # //         // Remove file information and all data from persistent storage
    /// # //         todo!()
    /// # //     }
    /// # //
    /// # //     fn update(&self, file: &mut CometFile, buf: &mut [u8]) -> std::io::Result<()> {
    /// # //         // Patch the file content based on current offset
    /// # //         todo!()
    /// # //     }
    /// # // }
    ///
    /// # //   #[launch]
    /// # //   fn rocket() -> _ {
    /// # //       let meteoritus: Meteoritus<Ignite> = Meteoritus::new()
    /// # //           .with_temp_path("./tmp/uploads") // This will be ignored by Meteoritus
    /// # //           .with_vault(MyCustomVault::new())
    /// # //           .build();
    /// # //     
    /// # //       rocket::build().attach(meteoritus)
    /// # //   }
    ///   ```
    pub(crate) fn with_vault<V: Vault + 'static>(mut self, vault: V) -> Self {
        self.vault = Arc::new(vault);
        self
    }

    /// Maximum upload size in a single `PATCH` request.
    ///
    /// # Examples
    ///
    ///   ```rust,no_run
    ///   # #[macro_use] extern crate rocket;
    ///   use rocket::{Ignite, data::ByteUnit};
    ///   use meteoritus::Meteoritus;
    ///
    ///   #[launch]
    ///   fn rocket() -> _ {
    ///       let meteoritus: Meteoritus<Ignite> = Meteoritus::new()
    ///           .with_max_size(ByteUnit::Gibibyte(1))
    ///           .build();
    ///     
    ///       rocket::build().attach(meteoritus)
    /// }
    /// ```
    pub fn with_max_size(mut self, size: ByteUnit) -> Self {
        self.max_size = size;
        self
    }

    /// Adds a custom validation callback to be executed during file creation.
    ///
    /// The callback function will be called during file creation and can be used to perform custom metadata validation
    /// or other tasks related to the creation process. The function takes a [`HandlerContext`] parameter that contains
    /// information about the file being created, including its metadata.
    ///
    /// The callback function should return a `Result<(), Box<dyn Error>>` indicating whether the validation
    /// succeeded or failed. If the validation fails, the function should return an Err containing an error message.
    /// # Examples
    ///
    ///   ```rust,no_run
    ///   # #[macro_use] extern crate rocket;
    ///   use rocket::{Ignite, data::ByteUnit};
    ///   use meteoritus::{Built, HandlerContext, Meteoritus};
    ///   # pub struct DbService {}
    ///   #
    ///   # impl DbService {
    ///   #     fn new() -> DbService {
    ///   #         Self {}
    ///   #     }
    ///   #
    ///   #     fn say_hello(&self) {
    ///   #         println!("Hello from DbService")
    ///   #     }
    ///   # }
    ///
    ///   #[launch]
    ///   fn rocket() -> _ {
    ///       let meteoritus: Meteoritus<Ignite> = Meteoritus::new()
    ///           .on_creation(|ctx: HandlerContext<Built>| {
    ///               println!("On Creation: {:?}", ctx.file_info);
    ///
    ///               // Apply metadata validation here:
    ///               let Some(metadata) = ctx.file_info.metadata() else {
    ///                   return Err("Metadata not specified!".into());
    ///               };
    ///     
    ///               if let Err(e) = metadata.get_raw("filetype") {
    ///                   return Err(e.into());
    ///               }
    ///     
    ///               // Using rocket instance to get managed services
    ///               let db_service = ctx.rocket.state::<DbService>().unwrap();
    ///               db_service.say_hello();
    ///     
    ///               Ok(())
    ///           })
    ///           .build();
    ///     
    ///       rocket::build().attach(meteoritus)
    /// }
    /// ```
    /// The above example adds a custom validation callback that checks the metadata of the file being created to
    /// ensure that it contains a `"filetype"` field. If the validation fails, an error message is returned. The
    /// callback also demonstrates the use of the rocket instance to access managed services.
    pub fn on_creation<F>(mut self, callback: F) -> Self
    where
        F: Fn(HandlerContext<Built>) -> Result<(), Box<dyn Error>>
            + Send
            + Sync
            + 'static,
    {
        self.on_creation = Some(Arc::new(callback));
        self
    }

    /// Adds a callback to be executed after a file has been successfully created and saved to disk.
    ///
    /// The callback function will be called after a file has been successfully created and saved to disk. The function
    /// takes a [`HandlerContext`] parameter that contains information about the file that was just created,
    /// including its metadata.
    ///
    /// # Examples
    ///   ```rust,no_run
    ///   # #[macro_use] extern crate rocket;
    ///   use rocket::{Ignite, data::ByteUnit};
    ///   use meteoritus::{Created, HandlerContext, Meteoritus};
    ///   # pub struct DbService {}
    ///   #
    ///   # impl DbService {
    ///   #     fn new() -> DbService {
    ///   #         Self {}
    ///   #     }
    ///   #
    ///   #     fn say_hello(&self) {
    ///   #         println!("Hello from DbService")
    ///   #     }
    ///   # }
    ///
    ///   #[launch]
    ///   fn rocket() -> _ {
    ///       let meteoritus: Meteoritus<Ignite> = Meteoritus::new()
    ///           .on_created(|ctx: HandlerContext<Created>| {
    ///               println!("File saved on disk: {:?}", ctx.file_info);
    ///
    ///               // Using rocket instance to get managed services
    ///               let db_service = ctx.rocket.state::<DbService>().unwrap();
    ///               db_service.say_hello();
    ///           })
    ///           .build();
    ///     
    ///       rocket::build().attach(meteoritus)
    /// }
    /// ```
    /// The above example adds a callback function that simply logs the file information after it has been successfully
    /// created and saved to disk also demonstrates the use of the rocket instance to access managed services.
    pub fn on_created<F>(mut self, callback: F) -> Self
    where
        F: Fn(HandlerContext<Created>) + Send + Sync + 'static,
    {
        self.on_created = Some(Arc::new(callback));
        self
    }

    /// Specifies a callback to be called when a file upload is completed.
    ///
    /// The `on_completed` callback function takes a [`HandlerContext<Completed>`] parameter and
    /// is called once a file upload is completed. This allows users of the library to perform
    /// additional actions after the file has been uploaded and processed.
    ///
    /// At this point is recommended to move uploads to a permanent/secure location, by default Meteoritus
    /// is configured to auto-terminate after `on_completed` was invoked.
    /// Consider add [` Meteoritus::keep_on_disk()`] in order to overwrite this.
    ///
    /// # Examples
    ///   ```rust,no_run
    ///   # #[macro_use] extern crate rocket;
    ///   use rocket::{Ignite, data::ByteUnit};
    ///   use std::{fs, path::Path, str::from_utf8};
    ///   use meteoritus::{Completed, HandlerContext, Meteoritus};
    ///   # pub struct DbService {}
    ///   #
    ///   # impl DbService {
    ///   #     fn new() -> DbService {
    ///   #         Self {}
    ///   #     }
    ///   #
    ///   #     fn say_hello(&self) {
    ///   #         println!("Hello from DbService")
    ///   #     }
    ///   # }
    ///
    ///   #[launch]
    ///   fn rocket() -> _ {
    ///       let meteoritus: Meteoritus<Ignite> = Meteoritus::new()
    ///           .on_completed(|ctx: HandlerContext<Completed>| {
    ///               println!("Upload completed: {:?}", ctx.file_info);
    ///       
    ///               // Retrieving mimetype from Metadata
    ///               let mime = ctx
    ///                   .file_info
    ///                   .metadata()
    ///                   .as_ref()
    ///                   .unwrap()
    ///                   .get_raw("filetype")
    ///                   .unwrap();
    ///       
    ///               let source_path = Path::new(ctx.file_info.file_name());
    ///       
    ///               let destination_dir = Path::new("./tmp/files");
    ///               fs::create_dir_all(destination_dir).unwrap();
    ///       
    ///               let file_ext =
    ///                   from_utf8(&mime).unwrap().split("/").collect::<Vec<&str>>()[1];
    ///       
    ///               let destination_path = destination_dir
    ///                   .join(ctx.file_info.id())
    ///                   .with_extension(file_ext);
    ///       
    ///               // copying file to permanent location
    ///               fs::copy(source_path, destination_path).unwrap();
    ///       
    ///               // Using rocket instance to get managed services
    ///               let db_service = ctx.rocket.state::<DbService>().unwrap();
    ///               db_service.say_hello();
    ///           })
    ///           .build();
    ///     
    ///       rocket::build().attach(meteoritus)
    /// }
    /// ```
    /// The above example adds a callback function that move the uploaded file to a permanent location using
    /// [`Metadata`] values to discover the file extension also demonstrates the use of the rocket instance to access
    /// managed services.
    ///
    pub fn on_completed<F>(mut self, callback: F) -> Self
    where
        F: Fn(HandlerContext<Completed>) + Send + Sync + 'static,
    {
        self.on_completed = Some(Arc::new(callback));
        self
    }

    /// Specifies a callback to be executed after a file has been successfully terminated deleted from disk.
    ///
    /// The callback function will be called when a client Termination request occurs. The function
    /// takes a [`HandlerContext`] parameter that contains information about the file that was deleted from disk.
    ///
    /// # Examples
    ///   ```rust,no_run
    ///   # #[macro_use] extern crate rocket;
    ///   use rocket::{Ignite, data::ByteUnit};
    ///   use meteoritus::{Terminated, HandlerContext, Meteoritus};
    ///   # pub struct DbService {}
    ///   #
    ///   # impl DbService {
    ///   #     fn new() -> DbService {
    ///   #         Self {}
    ///   #     }
    ///   #
    ///   #     fn say_hello(&self) {
    ///   #         println!("Hello from DbService")
    ///   #     }
    ///   # }
    ///
    ///   #[launch]
    ///   fn rocket() -> _ {
    ///       let meteoritus: Meteoritus<Ignite> = Meteoritus::new()
    ///           .on_termination(|ctx: HandlerContext<Terminated>| {
    ///               println!("File was terminated by client: {:?}", ctx.file_info);
    ///
    ///               // Using rocket instance to get managed services
    ///               let db_service = ctx.rocket.state::<DbService>().unwrap();
    ///               db_service.say_hello();
    ///           })
    ///           .build();
    ///     
    ///       rocket::build().attach(meteoritus)
    /// }
    /// ```
    /// The above example adds a callback function that simply logs the file information after it has been terminated by a client request. Also demonstrates the use of the rocket instance to access managed services.
    pub fn on_termination<F>(mut self, callback: F) -> Self
    where
        F: Fn(HandlerContext<Terminated>) + Send + Sync + 'static,
    {
        self.on_termination = Some(Arc::new(callback));
        self
    }
}

impl Meteoritus<Ignite> {
    /// Returns a instance of [`Meteoritus`] into the _[`Orbit`]_ phase.
    pub(crate) fn launch(&self) -> Meteoritus<Orbit> {
        Meteoritus::<Orbit> {
            state: std::marker::PhantomData,
            vault: self.vault.to_owned(),
            on_creation: self.on_creation.to_owned(),
            on_created: self.on_created.to_owned(),
            on_completed: self.on_completed.to_owned(),
            on_termination: self.on_termination.to_owned(),
            ..*self
        }
    }
}

impl Meteoritus<Orbit> {
    /// Returns the `base` route where all tus middleware routes are mounted.
    pub fn base_route(&self) -> &str {
        self.base_route
    }

    /// Indicates if completed uploads should be auto deleted from disk.
    pub fn auto_terminate(&self) -> bool {
        self.auto_terminate
    }

    /// Returns the maximum allowed upload size.
    pub fn max_size(&self) -> ByteUnit {
        self.max_size
    }

    pub(crate) fn on_creation(
        &self,
    ) -> &Option<
        Arc<
            dyn Fn(HandlerContext<Built>) -> Result<(), Box<dyn Error>>
                + Send
                + Sync,
        >,
    > {
        &self.on_creation
    }

    pub(crate) fn on_created(
        &self,
    ) -> &Option<Arc<dyn Fn(HandlerContext<Created>) + Send + Sync>> {
        &self.on_created
    }

    pub(crate) fn on_completed(
        &self,
    ) -> &Option<Arc<dyn Fn(HandlerContext<Completed>) + Send + Sync>> {
        &self.on_completed
    }

    pub(crate) fn on_termination(
        &self,
    ) -> &Option<Arc<dyn Fn(HandlerContext<Terminated>) + Send + Sync>> {
        &self.on_termination
    }
}

#[rocket::async_trait]
impl Fairing for Meteoritus<Ignite> {
    fn info(&self) -> Info {
        Info {
            name: "Meteoritus",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let routes = routes![
            creation_handler,
            info_handler,
            file_info_handler,
            termination_handler,
            upload_handler,
        ];

        Ok(rocket
            .manage(self.launch())
            .manage(self.vault.to_owned())
            .mount(self.base_route, routes))
    }
}
