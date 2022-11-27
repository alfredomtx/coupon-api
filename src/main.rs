use actix_mysql::{
    configuration::{
        get_configuration,
        DatabaseSettings,
        Settings,
    },
    cupom::{
        get_cupom_by_code,
        get_cupom_by_id,
        get_all_cupoms,
        add_cupom,
    },
    routes::{
        health_check,
    },
    telemetry::{get_subscriber, init_subscriber},
};
use actix_web::{
    App, HttpServer,
    dev::Server, 
    web::Data,
};
use sqlx::{
    MySqlPool,
    mysql::MySqlPoolOptions,
};
use tracing_actix_web::TracingLogger;
use std::net::TcpListener;

pub fn run(
    listener: TcpListener,
    db_pool: MySqlPool,
    base_url: String,
) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            // TracingLogger instead of default actix_web logger to return with request_id (and other information aswell)
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(get_cupom_by_id)
            .service(get_cupom_by_code)
            .service(get_all_cupoms)
            .service(add_cupom )
            .app_data(db_pool.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    return Ok(server);
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // initializing subscriber for tracing & telemetry stuff
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    
    Ok(())
}


pub struct Application {
    port: u16, 
    server: Server,
}

// We need to define a wrapper type in order to retrieve the URL
// in the `subscribe` handler.
// Retrieval from the context, in actix-web, is type-based: using
// a raw `String` would expose us to conflicts.
pub struct ApplicationBaseUrl(pub String);

impl Application {
    // We have converted the `build` function into a constructor for `Application`
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {

        let connection_pool = get_connection_pool(&configuration.database);

        let address = format!("{}:{}"
            , configuration.application.host, configuration.application.port
        );
        // Bubble up the io::Error if we failed to bind the address
        // Otherwise call .await on our Server
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        print!("Running on {:?}:{:?}", configuration.application.host, configuration.application.port);
        let server = run(
            listener,
            connection_pool,
            configuration.application.base_url,
        )?;



        // We "save" the bound port in one of `Application`'s fields
        return Ok(Self { port, server });
    }

    pub fn port(&self) -> u16 {
        return self.port;
    }

    // A more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        return self.server.await;
    }
    
}

fn get_connection_pool(configuration: &DatabaseSettings) -> MySqlPool {
    return MySqlPoolOptions::new()
    .acquire_timeout(std::time::Duration::from_secs(2))
    .connect_lazy_with(configuration.with_db());
}

