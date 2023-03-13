use std::{marker::PhantomData, sync::Arc};

use rocket::{
    data::ByteUnit,
    fairing::{self, Fairing, Info, Kind},
    http::uri::Reference,
    Build, Ignite, Orbit, Phase, Rocket,
};

use crate::handlers::{
    creation_handler, file_info_handler, info_handler, termination_handler, upload_handler,
};
use crate::{comet_vault::MeteorVault, CometFile, CometVault, MeteoritusHeaders};

/// The tus fairing itself.
///
/// # Phases
///
/// A ` Meteoritus` instance represents a tus middleware and its state. It progresses
/// through three statically-enforced phases: build, ignite, orbit.
///
/// * **Build**: _middleware configuration_
///
///   This phase enables:
///
///> * setting mount route and configuration options like: temp path and max upload size
///> * registering callbacks for events
///> * adding custom implementation for [`CometVault`]
///
///> This is the _only_ phase in which an instance can be modified. To finalize changes,
///> an instance is ignited via [` Meteoritus::build()`], progressing it into the <i>ignite</i>
///> phase, or directly launched into orbit with [`Meteoritus::launch()`] which progress
///> the instance through ignite into orbit.
///
/// * **Ignite**: _finalization of configuration_
///
///   An instance in the [`Ignite`] phase is in its final configuration.
///   Barring user-supplied interior mutation, application state is guaranteed
///   to remain unchanged beyond this point.
///   An instance in the ignite phase can be launched into orbit to serve tus
///   requests via [`Meteoritus::launch()`].
///
/// * **Orbit**: _a running tus middleware_
///
///   An instance in the [`Orbit`] phase represents a _running_ middleware,
///   actively serving requests.
///
/// # Launching
///
/// In order to launch a `Meteoritus` middleware an instance of `Meteoritus<Ignite>` _must_ be
/// attached to [`Rocket`] server using [`rocket::Rocket::attach()`]:
///
///   ```rust,no_run
///   # #[macro_use] extern crate rocket;
///   use meteoritus::{CometFile, CometVault, Meteoritus};
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
/// Since `Meteoritus<Build>` implements the _builder pattern_ it exports public methods
/// to customize the middleware behavior, like registering event callbacks and custom          
/// configuration:
///
///   ```rust,no_run
///   # #[macro_use] extern crate rocket;
///   use meteoritus::{CometFile, CometVault, Meteoritus};
///
///   #[launch]
///   fn rocket() -> _ {
///       let meteoritus: Meteoritus<Ignite> = Meteoritus::new()
///           .mount_to("/api/files")
///           .with_temp_path("./tmp/uploads")
///           .with_vault(MyCustomVault::new())
///           .with_max_size(ByteUnit::Gibibyte(1))
///           .on_creation(|rocket, file, upload_uri| {
///                 println!("On Creation: Invoked!!");
///                 println!("Given file {:?}", file);
///                 Ok(())
///            })
///           .on_complete(|rocket| {
///                println!("Upload complete!");
///            })
///           .on_termination(|rocket| {
///                println!("File deleted!");
///            })
///           .build();
///     
///       rocket::build().attach(meteoritus)
///   }
///   ```
#[derive(Clone)]
pub struct Meteoritus<P: Phase> {
    base_route: &'static str,
    max_size: ByteUnit,
    vault: Arc<dyn CometVault>,
    on_creation: Option<
        Arc<
            dyn Fn(&Rocket<Orbit>, &CometFile, &mut Reference) -> Result<(), &'static str>
                + Send
                + Sync,
        >,
    >,
    on_complete: Option<Arc<dyn Fn(&Rocket<Orbit>) + Send + Sync>>,
    on_termination: Option<Arc<dyn Fn() + Send + Sync>>,
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
        MeteoritusHeaders::Extensions(&["creation"])
    }

    pub fn get_protocol_max_size(&self) -> MeteoritusHeaders {
        MeteoritusHeaders::MaxSize(self.max_size.as_u64())
    }
}

impl Meteoritus<Build> {
    /// Returns a instance of `Meteoritus` into the _build_ phase.
    pub fn new() -> Meteoritus<Build> {
        Meteoritus::<Build> {
            base_route: "/meteoritus",
            max_size: ByteUnit::Megabyte(5),
            vault: Arc::new(MeteorVault::new("./tmp/files")),
            on_creation: Default::default(),
            on_complete: Default::default(),
            on_termination: Default::default(),
            state: PhantomData::<Build>,
        }
    }

    /// Returns a instance of `Meteoritus` into the _ignite_ phase.
    pub fn build(self) -> Meteoritus<Ignite> {
        Meteoritus::<Ignite> {
            state: std::marker::PhantomData,
            ..self
        }
    }

    /// Mounts all of the routes of tus middleware in the supplied given `base`
    /// path.
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
    /// `Meteoritus` middleware.
    ///
    ///   ```rust,no_run
    ///   # #[macro_use] extern crate rocket;
    ///   use meteoritus::{CometFile, CometVault, Meteoritus};
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
    /// **Note:** `Meteoritus` will mount many tus protocol routes based on the specified path.
    pub fn mount_to(mut self, base_route: &'static str) -> Self {
        self.base_route = base_route;
        self
    }

    /// Directory to store temporary files.
    ///
    /// **Note:** If a custom [`CometVault`] has provided then the `Meteoritus` will ignore
    /// the supplied `temp_path`.
    ///
    /// # Examples
    ///
    /// Assign relative `tmp/uploads` to store uploaded files into.
    ///
    ///   ```rust,no_run
    ///   # #[macro_use] extern crate rocket;
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
        self.with_vault(MeteorVault::new(temp_path))
    }

    /// Overrides the default instance of [`CometVault`].
    ///
    /// If a custom vault has provided then the `Meteoritus` will ignore the `with_temp_path()`
    /// configuration. Since it assumes that all file system operations will be responsibility of
    /// the custom vault implementation.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[macro_use] extern crate rocket;
    /// use meteoritus::{CometFile, CometVault, Meteoritus};
    ///
    /// pub struct MyCustomVault {}
    ///
    /// impl MyCustomVault {
    ///     pub fn new() -> Self {
    ///         Self {}
    ///     }
    /// }
    ///
    /// impl CometVault for MyCustomVault {
    ///     fn add(&self, file: &CometFile) -> Result<()> {
    ///         // Save file information on some persistent storage
    ///         todo!()
    ///     }
    ///
    ///     fn take(&self, id: String) -> Result<CometFile> {
    ///         // Get the file information from persistent storage
    ///         todo!()
    ///     }
    ///
    ///     fn remove(&self, file: &CometFile) -> Result<()> {
    ///         // Remove file information and all data from persistent storage
    ///         todo!()
    ///     }
    ///
    ///     fn update(&self, file: &mut CometFile, buf: &mut [u8]) -> std::io::Result<()> {
    ///         // Patch the file content based on current offset
    ///         todo!()
    ///     }
    /// }
    ///
    ///   #[launch]
    ///   fn rocket() -> _ {
    ///       let meteoritus: Meteoritus<Ignite> = Meteoritus::new()
    ///           .with_temp_path("./tmp/uploads") // This will be ignored by Meteoritus
    ///           .with_vault(MyCustomVault::new())
    ///           .build();
    ///     
    ///       rocket::build().attach(meteoritus)
    ///   }
    ///   ```
    pub fn with_vault<V: CometVault + 'static>(mut self, vault: V) -> Self {
        self.vault = Arc::new(vault);
        self
    }

    /// Maximum upload size in a single `PATCH` request.
    ///
    /// # Examples
    ///
    ///   ```rust,no_run
    ///   # #[macro_use] extern crate rocket;
    ///   use rocket::data::ByteUnit
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

    pub fn on_creation<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Rocket<Orbit>, &CometFile, &mut Reference) -> Result<(), &'static str>
            + Send
            + Sync
            + 'static,
    {
        self.on_creation = Some(Arc::new(callback));
        self
    }

    pub fn on_complete<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Rocket<Orbit>) + Send + Sync + 'static,
    {
        self.on_complete = Some(Arc::new(callback));
        self
    }

    pub fn on_termination<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_termination = Some(Arc::new(callback));
        self
    }
}

impl Meteoritus<Ignite> {
    /// Returns a instance of `Meteoritus` into the _orbit_ phase.
    pub fn launch(&self) -> Meteoritus<Orbit> {
        Meteoritus::<Orbit> {
            state: std::marker::PhantomData,
            vault: self.vault.to_owned(),
            on_creation: self.on_creation.to_owned(),
            on_complete: self.on_complete.to_owned(),
            on_termination: self.on_termination.to_owned(),
            ..*self
        }
    }
}

impl Meteoritus<Orbit> {
    pub fn base_route(&self) -> &str {
        self.base_route
    }

    pub fn max_size(&self) -> ByteUnit {
        self.max_size
    }

    pub fn vault(&self) -> &Arc<dyn CometVault> {
        &self.vault
    }

    pub fn on_creation(
        &self,
    ) -> &Option<
        Arc<
            dyn Fn(&Rocket<Orbit>, &CometFile, &mut Reference) -> Result<(), &'static str>
                + Send
                + Sync,
        >,
    > {
        &self.on_creation
    }

    pub fn on_complete(&self) -> &Option<Arc<dyn Fn(&Rocket<Orbit>) + Send + Sync>> {
        &self.on_complete
    }

    pub fn on_termination(&self) -> &Option<Arc<dyn Fn() + Send + Sync>> {
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

        Ok(rocket.manage(self.launch()).mount(self.base_route, routes))
    }
}
