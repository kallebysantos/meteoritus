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

    pub fn build(self) -> Meteoritus<Ignite> {
        Meteoritus::<Ignite> {
            state: std::marker::PhantomData,
            ..self
        }
    }

    pub fn mount_to(mut self, base_route: &'static str) -> Self {
        self.base_route = base_route;
        self
    }

    pub fn with_temp_path(self, temp_path: &'static str) -> Self {
        self.with_vault(MeteorVault::new(temp_path))
    }

    pub fn with_vault<V: CometVault + 'static>(mut self, vault: V) -> Self {
        self.vault = Arc::new(vault);
        self
    }

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
