use crate::{
    configuration::{DatabaseSettings, Settings},
    login::{
        UserClaims, hello, verify_service_request, login_handler
    },
    cupom::{
        get_cupom_by_code,
        get_cupom_by_id,
        get_all_cupoms,
        add_cupom,
    },
    routes::{health_check},
};
use actix_web::{
    App, HttpServer,
    dev::Server, 
    web::{Data, self, scope},
};
use sqlx::{
    MySqlPool,
    mysql::MySqlPoolOptions,
};
use tracing_actix_web::TracingLogger;
use std::net::TcpListener;
use actix_jwt_auth_middleware::{AuthError, AuthService, Authority, FromRequest};


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

pub fn get_connection_pool(configuration: &DatabaseSettings, test_database: bool) -> MySqlPool {
    return MySqlPoolOptions::new()
    .acquire_timeout(std::time::Duration::from_secs(2))
    .connect_lazy_with(configuration.with_db(test_database));
}

pub fn run(listener: TcpListener, db_pool: MySqlPool, base_url: String) -> Result<Server, std::io::Error> {
    // we initialize a new Authority passing the underling type the JWT token should destructure into.
    let auth_authority = Authority::<UserClaims>::default();
    
    let db_pool = Data::new(db_pool);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            // TracingLogger instead of default actix_web logger to return with request_id (and other information aswell)
            .wrap(TracingLogger::default())

            .app_data(Data::new(auth_authority.clone()))
            .app_data(db_pool.clone())
            .app_data(base_url.clone())

            /*
                all access routes 
            */ 
            // in order to wrap the entire app scope excluding the login handlers we have add a new service
            // with an empty scope first
            .service(scope("").service(hello).wrap(AuthService::new(
                auth_authority.clone(),
                // we pass the guard function to use with this auth service
                verify_service_request,
            )))
            .service(health_check)
            /*
                authenticated routes - admin
            */ 
            .service(scope("").service(login_handler).wrap(AuthService::new(
                auth_authority.clone(), verify_service_request,
            )))
            /*
                authenticated routes - user
            */ 
            .service(get_cupom_by_id)
            .service(get_cupom_by_code)
            .service(get_all_cupoms)
            .service(add_cupom )


    })
    .listen(listener)?
    .run();

    return Ok(server);
}