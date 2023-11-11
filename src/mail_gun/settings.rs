use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct MailGunSettings {
    pub api_url: String,
    pub client_id: String,
    pub client_secret: String,
}
impl Default for MailGunSettings {
    fn default() -> Self {
        Self {
            api_url: "https://api.mailgun.net/v3".to_string(),
            client_id: String::default(),
            client_secret: String::default(),
        }
    }
}
