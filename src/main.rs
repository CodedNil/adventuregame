mod cards;
mod model;
mod server_functions;
mod styles;
mod ui;

#[cfg(feature = "server")]
mod ai;
#[cfg(feature = "server")]
mod game;
#[cfg(feature = "server")]
mod server;

fn main() {
    #[cfg(feature = "server")]
    server::launch();

    #[cfg(not(feature = "server"))]
    dioxus::launch(ui::App);
}
