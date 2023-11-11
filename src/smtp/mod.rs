mod access;
mod settings;
use std::{mem, sync::Arc};

#[doc(inline)]
pub use access::*;
use lettre::{
    message::{header, Mailbox as SMTPMailBox, MessageBuilder, MultiPart, SinglePart},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    AsyncSmtpTransport, Message,
};
#[doc(inline)]
pub use settings::*;
use thiserror::Error;
use tracing::{debug, error, instrument, warn};

use crate::{
    email_types::{Address, Mailbox},
    shared::{bounded_channel, unbdounded_channel, Mutex, Receiver},
    template::EmailBody,
    Email, MailService, ServiceState,
};

#[derive(Debug, Error)]
pub enum SMTPError {
    #[error("The email queue is full")]
    QueueError(#[from] flume::SendError<Message>),
    #[error(transparent)]
    InvalidEmailAddress(#[from] lettre::address::AddressError),
    #[error(transparent)]
    InvalidEmail(#[from] lettre::error::Error),
    #[error("No Body was Provided")]
    NoBodyProvided,
    #[error("No To Address was Provided")]
    NoToAddressProvided,
    #[error(transparent)]
    SendError(#[from] lettre::transport::smtp::Error),
}
#[cfg(any(feature = "tokio"))]
type SelectedExecutor = lettre::Tokio1Executor;
#[cfg(not(any(feature = "tokio")))]
compile_error!(
    "No executor selected, please select one of the following features: tokio, actix-rt"
);
#[derive(Debug)]
pub struct ConnectionState {
    pub connected: bool,
}

pub type SharedConnectionState = Arc<Mutex<ConnectionState>>;

type Transport = AsyncSmtpTransport<SelectedExecutor>;
#[derive(Debug)]
pub struct SMTPService {
    settings: Arc<SMTPServiceSettings>,
    queue: Receiver<Message>,
    transport: Option<Transport>,
    state: SharedConnectionState,
    service_state: Arc<ServiceState>,
}
impl SMTPService {
    /// Internal Send that doesn't return an error
    #[instrument]
    async fn send(&self, email: Message) {
        use lettre::AsyncTransport;
        if let Some(transport) = &self.transport {
            match transport.send(email).await {
                Ok(_ok) => {
                    // TODO: Handle OK
                }
                Err(err) => {
                    error!("Error Sending Email: {}", err);
                }
            }
        } else {
            debug!("Email Transport not initialized");
        }
    }
    #[instrument]
    async fn build_connection(
        settings: &SMTPServiceSettings,
    ) -> Result<Option<lettre::AsyncSmtpTransport<SelectedExecutor>>, SMTPError> {
        let SMTPServiceSettings {
            username,
            password,
            host,
            port,
            encryption,
            client_id,
            ..
        } = settings.clone();
        let credentials = Credentials::new(username, password);

        let port = port.unwrap_or(encryption.default_port());

        debug!(
            "Connecting to {}:{}. Encrption: {}. Client ID: {}",
            host, port, encryption, client_id
        );

        let tls = match settings.encryption {
            SMTPServiceEncryption::StartTLS => Tls::Required(TlsParameters::new(host.clone())?),
            SMTPServiceEncryption::NONE => Tls::None,
            SMTPServiceEncryption::TLS => Tls::Wrapper(TlsParameters::new(host.clone())?),
        };

        let transport = Transport::builder_dangerous(host)
            .port(port)
            .tls(tls)
            .credentials(credentials)
            .timeout(settings.get_timeout())
            .hello_name(client_id.into())
            .build();

        // TODO Support Client Name

        if !transport.test_connection().await? {
            warn!("Email Transport Test Connection Failed");
            return Ok(None);
        }
        Ok(Some(transport))
    }
    #[cfg(feature = "tokio")]
    async fn start_inner(self) {
        use tokio::select;
        loop {
            let receiver = self.queue.recv_async();
            let notified = self.service_state.notify.notified();
            let message = select! {
            _ = notified => {
                debug!("Notified to shutdown");
                break
            }
            v = receiver => {
                if let Ok(value) = v{
                    value
                } else {
                    debug!("All Senders Dropped. Closing SMTP Service");
                    break;
                }
              }
            };
            // Maybe we should start a new task for this?
            self.send(message).await;
        }
    }
    #[cfg(not(feature = "tokio"))]
    async fn start_inner(self) {
        compile_error!("Currently only tokio is supported")
    }
    fn start(self) {
        let service_state = self.service_state.clone();
        ServiceState::watch_for_shutdown(service_state);
        #[cfg(feature = "tokio")]
        tokio::spawn(self.start_inner());
        #[cfg(not(any(feature = "tokio")))]
        compile_error!(
            "No executor selected, please select one of the following features: tokio, actix-rt"
        );
    }

    pub fn settings(&self) -> &SMTPServiceSettings {
        self.settings.as_ref()
    }
}
impl MailService for SMTPService {
    type Settings = SMTPServiceSettings;
    type Access = SMTPEmailAccess;
    type Error = SMTPError;
    type ConnectionState = ConnectionState;

    async fn init(settings: Self::Settings) -> Result<SMTPEmailAccess, Self::Error>
    where
        Self: Sized,
    {
        let transport = match Self::build_connection(&settings).await {
            Ok(ok) => ok,
            Err(value) => {
                warn!("Error building email transport: {}", value);
                None
            }
        };
        let (queue, receiver) = if settings.channel_size == 0 {
            unbdounded_channel()
        } else {
            bounded_channel(settings.channel_size)
        };
        let state = ConnectionState {
            connected: transport.is_some(),
        };

        let shared = Arc::new(Mutex::new(state));
        let settings = Arc::new(settings);
        let service_state = ServiceState::new();
        SMTPService {
            settings: settings.clone(),
            queue: receiver,
            transport,
            state: shared.clone(),
            service_state: service_state.clone(),
        }
        .start();

        Ok(SMTPEmailAccess {
            settings: settings,
            queue: queue,
            message_builder: MessageBuilder::new(),
            state: shared,
            service_state,
        })
    }

    fn get_state(&self) -> Arc<crate::shared::Mutex<Self::ConnectionState>> {
        self.state.clone()
    }

    fn get_app_state(&self) -> Arc<ServiceState> {
        self.service_state.clone()
    }

    fn get_settings(&self) -> Arc<Self::Settings> {
        self.settings.clone()
    }

    async fn is_connected(&self) -> bool {
        if let Some(transport) = &self.transport {
            let connected = transport.test_connection().await.unwrap_or(false);
            self.state.lock().connected = connected;
            connected
        } else {
            false
        }
    }
}

pub(crate) fn email_to_message(
    mut message: MessageBuilder,
    _settings: &SMTPServiceSettings,
    mut email: impl Email,
) -> Result<Message, SMTPError> {
    let Some(body) = email.body() else {
        return Err(SMTPError::NoBodyProvided);
    };
    let mut to_set = email.to();
    if to_set.len() == 0 {
        return Err(SMTPError::NoToAddressProvided);
    }
    let body = email_body_to_multipart(body);
    if to_set.len() != 1 {
        todo!("Implement sending to multiple addresses")
    } else {
        message = message.to(to_set.next().unwrap().clone().into());
    }

    if let Some(from) = email.from() {
        message = message.from(from.clone().into());
    };

    if let Some(reply_to) = email.reply_to() {
        message = message.reply_to(reply_to.clone().into());
    };

    let message = message.multipart(body)?;

    Ok(message)
}

fn email_body_to_multipart(body: EmailBody) -> MultiPart {
    let multipart = MultiPart::alternative();
    let multipart = if let Some(html) = body.html_body {
        multipart.singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(html),
        )
    } else {
        multipart.build()
    };
    let multipart = if let Some(text) = body.text_body {
        multipart.singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_PLAIN)
                .body(text),
        )
    } else {
        multipart
    };
    multipart
}

impl From<Mailbox> for SMTPMailBox {
    fn from(value: Mailbox) -> Self {
        let Mailbox { name, email } = value;
        SMTPMailBox::new(name, email.into())
    }
}
#[cfg(debug_assertions)]
impl From<Address> for lettre::Address {
    fn from(value: Address) -> Self {
        // As long as lettre deosn't change the layout of this struct, this is safe
        let result: lettre::Address = unsafe { mem::transmute_copy(&value) };
        // Checks for layout changes
        assert_eq!(
            &value, &result,
            "lettre::Address did not have the same layout as any_mail::email_types::Address"
        );
        result
    }
}
#[cfg(not(debug_assertions))]
impl From<Address> for lettre::Address {
    fn from(value: Address) -> Self {
        let result: lettre::Address = unsafe { mem::transmute(&value) };
        result
    }
}

impl PartialEq<lettre::Address> for Address {
    fn eq(&self, other: &lettre::Address) -> bool {
        self.serialized == other.as_ref()
    }
}
