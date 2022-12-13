#![allow(unused_parens)]
#![allow(clippy::needless_return)]

use actix_mysql::{
    configuration::{get_configuration},
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // initializing subscriber for tracing & telemetry stuff
    let subscriber = get_subscriber("cupom_api".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration, false).await?;
    application.run_until_stopped().await?;
    
    Ok(())
}

