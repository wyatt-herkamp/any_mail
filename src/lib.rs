use std::{borrow::Cow, error::Error, fmt::Debug, sync::Arc};
pub(crate) mod shared;
use email_types::Mailbox;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use template::EmailBody;
pub mod email_types;
#[cfg(feature = "mail-gun")]
pub mod mail_gun;
#[cfg(feature = "mail-whale")]
pub mod mail_whale;
pub mod no_op;
#[cfg(feature = "smtp")]
pub mod smtp;
pub mod template;
#[cfg(feature = "tokio")]
pub(crate) mod tokio_rt;

#[cfg(feature = "tokio")]
#[doc(inline)]
pub use tokio_rt::ServiceState;
/// The Mail Service Types
///
/// This exists for the user to able to select the mail service. Good for when you want to parse the mail service from ENV variables.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash, Default)]
pub enum MailServiceTypes {
    #[cfg(feature = "smtp")]
    SMTP,
    #[cfg(feature = "mail-gun")]
    MailGun,
    #[cfg(feature = "mail-whale")]
    MailWhale,
    #[default]
    None,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "method", content = "settings")]
pub enum MailServiceSettings {
    #[cfg(feature = "smtp")]
    SMTP(smtp::SMTPServiceSettings),
    #[cfg(feature = "mail-gun")]
    MailGun(mail_gun::MailGunSettings),
    #[cfg(feature = "mail-whale")]
    MailWhale(mail_whale::MailWhaleSettings),
    None,
}
/// The Layout for an Email Server
///
/// This should be used to create a new email service connection. Do not implement this trait. As the API is not stable.
#[allow(async_fn_in_trait)]
pub trait MailService {
    type Settings;
    type Access: EmailAccess<Settings = Self::Settings, ConnectionState = Self::ConnectionState>;
    type Error: Error + Send + Sync + 'static;
    type ConnectionState;

    async fn init(settings: Self::Settings) -> Result<Self::Access, Self::Error>
    where
        Self: Sized;

    fn get_state(&self) -> Arc<crate::shared::Mutex<Self::ConnectionState>>;

    fn get_app_state(&self) -> Arc<ServiceState>;

    fn get_settings(&self) -> Arc<Self::Settings>;

    async fn is_connected(&self) -> bool;
}
/// A Shared Access to the Mail Service. This is clonable and can be sent across threads.
pub trait EmailAccess: Clone + Send + Sync {
    type Error: Error + Send + Sync + 'static;
    type Settings: Clone + Serialize + DeserializeOwned;
    type ConnectionState;

    fn get_settings(&self) -> Arc<Self::Settings>;

    /// Pushes an email to the queue.
    fn send(&self, email: impl Email) -> Result<(), Self::Error>;

    fn get_state(&self) -> Arc<crate::shared::Mutex<Self::ConnectionState>>;

    fn get_app_state(&self) -> Arc<ServiceState>;
}

pub trait Email: Debug {
    fn subject(&self) -> Cow<'static, str>;

    fn body(&mut self) -> Option<EmailBody>;

    fn to(&self) -> impl Iterator<Item = &Mailbox> + ExactSizeIterator + '_;

    fn from(&self) -> Option<&Mailbox>;

    fn reply_to(&self) -> Option<&Mailbox> {
        None
    }
}
pub trait EmailSettingsType: Clone + Serialize + DeserializeOwned {
    fn from(&self) -> &Mailbox;

    fn reply_to(&self) -> Option<&Mailbox>;
}
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct SimpleEmail {
    pub subject: &'static str,
    pub body: Option<EmailBody>,
    pub to: Mailbox,
    pub from: Option<Mailbox>,
}

impl Email for SimpleEmail {
    fn subject(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.subject)
    }

    fn body(&mut self) -> Option<EmailBody> {
        self.body.take()
    }

    fn to(&self) -> impl Iterator<Item = &Mailbox> + ExactSizeIterator + '_ {
        std::iter::once(&self.to)
    }

    fn from(&self) -> Option<&Mailbox> {
        self.from.as_ref()
    }
}
