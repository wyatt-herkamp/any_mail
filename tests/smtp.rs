use std::path::PathBuf;

use any_mail::{smtp::SMTPServiceSettings, MailService};

fn load_smtp_config() -> anyhow::Result<SMTPServiceSettings> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("smtp.test.toml");

    if !path.exists() {
        let settings = toml::to_string_pretty(&SMTPServiceSettings::default())?;
        std::fs::write(&path, settings)?;
        anyhow::bail!("SMTP test config file not found: {:?}", path);
    }

    let settings = std::fs::read_to_string(&path)?;
    let settings: SMTPServiceSettings = toml::from_str(&settings)?;
    Ok(settings)
}
#[tokio::test]
async fn basic_tests() -> anyhow::Result<()> {
    let settings = load_smtp_config()?;

    let access = any_mail::smtp::SMTPService::init(settings.clone()).await?;
    println!("{:?}", access);
    Ok(())
}
