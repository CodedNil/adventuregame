use crate::{server_functions, ui};
use axum::Router;
use dioxus::{
    prelude::dioxus_server::{FullstackState, ServeConfig},
    server::DioxusRouterExt,
};

pub fn launch() {
    dotenvy::dotenv().ok();
    dioxus::logger::initialize_default();
    server_functions::init_state();

    tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(async move {
            let app = Router::<FullstackState>::new()
                .serve_dioxus_application(ServeConfig::new(), ui::App);
            let addr = dioxus::cli_config::fullstack_address_or_localhost();
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .expect("bind address");
            axum::serve(listener, app).await.expect("serve app");
        });
}
