use std::collections::HashMap;

use actix_web::{
    get,
    web::{self, Data},
    HttpResponse, Responder,
};
use handlebars::Handlebars;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::retrieve_owned_discs;

pub(crate) fn configure_collection_views(config: &mut web::ServiceConfig) {
    config.service(owned_discs);
}

#[get("views/collection")]
async fn owned_discs(
    template_registry: Data<Handlebars<'static>>,
    db_connection_pool: Data<Pool<SqliteConnectionManager>>,
) -> impl Responder {
    dbg!("test");
    let owned_discs = retrieve_owned_discs(db_connection_pool);
    let mut data = HashMap::new();
    data.insert("owned_discs", owned_discs);
    let render = template_registry
        .render("owned_discs", &data)
        .expect("Erreur lors du rendu du template");
    HttpResponse::Ok().body(render)
}
