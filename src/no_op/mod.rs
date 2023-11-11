use std::{convert::Infallible, sync::Arc};

use tracing::info;

use crate::{EmailAccess, MailService};
#[derive(Debug, Clone)]
pub struct NoOpAccess;
impl EmailAccess for NoOpAccess {
    type Error = Infallible;

    type Settings = ();

    type ConnectionState = ();

    fn get_settings(&self) -> std::sync::Arc<Self::Settings> {
        Arc::new(())
    }

    fn send(&self, email: impl crate::Email) -> Result<(), Self::Error> {
        info!("NoOpAccess: {:?}", email);
        Ok(())
    }

    fn get_state(&self) -> std::sync::Arc<crate::shared::Mutex<Self::ConnectionState>> {
        Arc::new(crate::shared::Mutex::new(()))
    }

    fn get_app_state(&self) -> std::sync::Arc<crate::ServiceState> {
        crate::ServiceState::new()
    }
}

pub struct NoOpService;
impl MailService for NoOpService {
    type Settings = ();

    type Access = NoOpAccess;

    type Error = Infallible;

    type ConnectionState = ();

    async fn init(_: Self::Settings) -> Result<Self::Access, Self::Error>
    where
        Self: Sized,
    {
        Ok(NoOpAccess)
    }

    fn get_state(&self) -> std::sync::Arc<crate::shared::Mutex<Self::ConnectionState>> {
        Arc::new(crate::shared::Mutex::new(()))
    }

    fn get_app_state(&self) -> std::sync::Arc<crate::ServiceState> {
        crate::ServiceState::new()
    }

    fn get_settings(&self) -> std::sync::Arc<Self::Settings> {
        Arc::new(())
    }

    async fn is_connected(&self) -> bool {
        false
    }
}
