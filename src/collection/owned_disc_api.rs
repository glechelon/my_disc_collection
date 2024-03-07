use actix_web::{
    delete, post,
    web::{self, Data},
    HttpResponse, Responder,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::{
    create_artist, create_disc, find_artist_id_by_name, is_artist_owned, is_disc_owned, OwnedDisc,
};

pub(crate) fn configure_collecrtion_api(config: &mut web::ServiceConfig) {
    config.service(add_owned_disc).service(remove_owned_disc);
}

#[post("api/owned-discs")]
async fn add_owned_disc(
    disc: web::Json<OwnedDisc>,
    db_connection_pool: Data<Pool<SqliteConnectionManager>>,
) -> HttpResponse {
    dbg!(&disc);
    let is_disc_already_present = is_disc_owned(&db_connection_pool, &disc.title);
    let is_artist_already_present = is_artist_owned(&db_connection_pool, &disc.artist.name);

    if is_disc_already_present {
        return HttpResponse::Conflict().finish();
    }

    if !is_artist_already_present {
        create_artist(&db_connection_pool, &disc.artist);
    }

    let artist_id: i32 = find_artist_id_by_name(&db_connection_pool, &disc.artist.name);
    create_disc(&db_connection_pool, &disc, &artist_id);

    HttpResponse::Created()
        .insert_header(("HX-Trigger", "refreshSearch"))
        .finish()
}

#[delete("api/owned-discs/{title}")]
async fn remove_owned_disc(
    title: web::Path<String>,
    db_connection_pool: Data<Pool<SqliteConnectionManager>>,
) -> impl Responder {
    dbg!(&title);
    let connection = db_connection_pool
        .get()
        .expect("Erreur à la récupération d'une connexion dans le pool");
    connection
        .prepare("DELETE FROM DISC WHERE TITLE = ?1")
        .and_then(|mut statement| statement.execute([&title.to_string().replace("%20", " ")])) //TODO: La suppression par ID est préféreable
        .expect("Erreur lors de la création du disque");
    HttpResponse::Created()
        .insert_header(("HX-Trigger", "refreshCollection"))
        .finish()
}
