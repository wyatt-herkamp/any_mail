use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct MailWhaleSettings {
    pub api_url: String,
    pub client_id: String,
    pub client_secret: String,
}
impl Default for MailWhaleSettings {
    fn default() -> Self {
        Self {
            api_url: "https://mailwhale.dev/api/mail".to_string(),
            client_id: String::default(),
            client_secret: String::default(),
        }
    }
}
