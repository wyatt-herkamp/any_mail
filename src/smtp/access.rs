use std::sync::Arc;

use lettre::{message::MessageBuilder, Message};

use super::{ConnectionState, SMTPError, SMTPServiceSettings, SharedConnectionState};
use crate::{shared::Sender, Email, EmailAccess, ServiceState};

#[derive(Debug, Clone)]
pub struct SMTPEmailAccess {
    pub(super) settings: Arc<SMTPServiceSettings>,
    pub(super) queue: Sender<Message>,
    pub(super) message_builder: MessageBuilder,
    pub(super) state: SharedConnectionState,
    pub(super) service_state: Arc<ServiceState>,
}
impl EmailAccess for SMTPEmailAccess {
    type Error = SMTPError;

    type Settings = SMTPServiceSettings;
    type ConnectionState = ConnectionState;

    fn send(&self, email: impl Email) -> Result<(), Self::Error> {
        let message =
            super::email_to_message(self.message_builder.clone(), self.settings.as_ref(), email)?;
        self.queue.send(message)?;
        Ok(())
    }

    fn get_settings(&self) -> Arc<Self::Settings> {
        self.settings.clone()
    }

    fn get_state(&self) -> Arc<crate::shared::Mutex<Self::ConnectionState>> {
        self.state.clone()
    }

    fn get_app_state(&self) -> Arc<crate::ServiceState> {
        self.service_state.clone()
    }
}
