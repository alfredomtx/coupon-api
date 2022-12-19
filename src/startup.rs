use crate::{
    configuration::{DatabaseSettings, Settings},
    authentication::{
        hello, authenticate, User
    },
    coupon::{
        health_check, get_coupon_by_code, get_coupon_by_id, get_all_coupons, add_coupon, update_coupon, delete_coupon_by_code, delete_coupon_by_id,
    },
};
use actix_web::{
    App, HttpServer,
    dev::Server, 
    web::{Data, scope},
};
use sqlx::{
    MySqlPool,
    mysql::MySqlPoolOptions,
};
use tracing_actix_web::TracingLogger;
use std::net::TcpListener;
use actix_jwt_auth_middleware::{Authority, CookieSigner, AuthenticationService};
use exonum_crypto::KeyPair;
use jwt_compact::alg::Ed25519;


pub fn run(listener: TcpListener, db_pool: MySqlPool, base_url: String, api_key: String) -> Result<Server, std::io::Error> {
    let key_pair = KeyPair::random();

    let cookie_signer = CookieSigner::new()
        .signing_key(key_pair.secret_key().clone())
        .algorithm(Ed25519)
        .build()
        .unwrap();

    let authority = Authority::<User, _, _, _>::new()
        .refresh_authorizer(|| async move { Ok(()) })
        .cookie_signer(Some(cookie_signer.clone()))
        .verifying_key(key_pair.public_key().clone())
        .build()
        .unwrap();
    
    let db_pool = Data::new(db_pool);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let api_key = Data::new(ApplicationApiKey(api_key));

    let server = HttpServer::new(move || {
        App::new()
            // TracingLogger instead of default actix_web logger to return with request_id (and other information aswell)
            .wrap(TracingLogger::default())

            .app_data(Data::new(authority.clone()))
            .app_data(db_pool.clone())
            .app_data(base_url.clone())
            .app_data(api_key.clone())
            .app_data(Data::new(cookie_signer.clone()))

            /*
                all access routes (not authenticated)
            */ 
            .service(authenticate)
            .service(health_check)

            /*
                authenticated routes
            */ 
            // in order to wrap the entire app scope excluding the login handlers we have add a new service
            // with an empty scope first
            .service(
                // we need this scope so we can exclude the login service
                // from being wrapped by the jwt middleware
                scope("")
                    .service(hello)
                    .service(get_all_coupons)
                    .service(get_coupon_by_id)
                    .service(get_coupon_by_code)
                    .service(add_coupon)
                    .service(update_coupon)
                    .service(delete_coupon_by_id)
                    .service(delete_coupon_by_code)
                    .wrap(AuthenticationService::new(authority.clone()))
                )

    })
    .listen(listener)?
    .run();

    return Ok(server);
}

pub fn get_connection_pool(configuration: &DatabaseSettings, test_database: bool) -> MySqlPool {
    return MySqlPoolOptions::new()
    .acquire_timeout(std::time::Duration::from_secs(2))
    .connect_lazy_with(configuration.with_db(test_database));
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
pub struct ApplicationApiKey(pub String);

impl Application {

    pub async fn build(configuration: Settings, test_database: bool) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database, test_database);

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
            configuration.application.api_key,
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