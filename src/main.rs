mod collection;
mod search;

use crate::collection::owned_disc::*;
use crate::collection::owned_disc_dam::*;

use std::collections::HashMap;

use actix_web::{
    get,
    middleware::Logger,
    web::Data,
    App, HttpResponse, HttpServer, Responder,
};
use handlebars::Handlebars;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let db_connection_manager = SqliteConnectionManager::file("my_physical_disc_collection.db");
    let db_connection_pool =
        Pool::new(db_connection_manager).expect("Erreur lors de la création du pool de connexion");
    let mut template_registry = Handlebars::new();
    template_registry
        .register_template_file("index", "templates/pages/index.hbs")
        .expect("Erreur lors de la récupération du template");
    template_registry
        .register_template_file("disc_search", "templates/parts/disc_search.hbs")
        .expect("Erreur lors de la récupération du template");
    template_registry
        .register_template_file("owned_discs", "templates/views/collection.hbs")
        .expect("Erreur lors de la récupération du template");
    template_registry
        .register_template_file("recherche", "templates/views/recherche.hbs")
        .expect("Erreur lors de la récupération du template");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(template_registry.clone()))
            .app_data(Data::new(db_connection_pool.clone()))
            .service(index)
            .configure(collection::owned_disc_api::configure_collecrtion_api)
            .configure(collection::collection_views::configure_collection_views)
            .configure(search::configure_search)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn index<'a>(template_registry: Data<Handlebars<'static>>) -> impl Responder {
    let var_name: HashMap<String, String> = HashMap::new();
    let render = template_registry
        .render("index", &var_name)
        .expect("Erreur rendu");
    HttpResponse::Ok().body(render)
}
