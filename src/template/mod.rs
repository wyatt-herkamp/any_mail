use std::error::Error;

use serde::Serialize;
pub trait EmailTemplate {
    fn template_txt() -> &'static str;
    fn template_html() -> &'static str;
}
pub trait TemplateSet<'a> {
    type Error: Error + Send + Sync + 'static;

    fn build_email<T: EmailTemplate>(
        &self,
        data: &impl Serialize,
    ) -> Result<EmailBody, Self::Error>
    where
        Self: Sized;
}
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct EmailBody {
    pub html_body: Option<String>,
    pub text_body: Option<String>,
}
