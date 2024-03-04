mod search;
mod collection;

use crate::collection::owned_disc_dam::*;
use crate::collection::owned_disc::*;

use std::collections::HashMap;

use actix_web::{
    delete, get,
    middleware::Logger,
    post,
    web::{self, Data, JsonConfig},
    App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder,
};
use handlebars::Handlebars;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};



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
            .service(owned_discs)
            .service(add_owned_disc)
            .service(remove_owned_disc)
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

#[post("api/owned-discs")]
async fn add_owned_disc(
    body: web::Json<OwnedDisc>,
    db_connection_pool: Data<Pool<SqliteConnectionManager>>,
) -> impl Responder {
    dbg!(&body);
    let connection = db_connection_pool
        .get()
        .expect("Erreur à la récupération d'une connexion dans le pool");
    let is_disc_already_present = connection
        .prepare("SELECT COUNT(*) FROM DISC WHERE TITLE =?1")
        .and_then(|mut statement| {
            statement.query_row([&body.title], |row| {
                row.get(0).and_then(|count: i32| Ok(count == 1))
            })
        })
        .expect("Erreur lors de la vérification de présence du disque");
    let is_artist_already_present = connection
        .prepare("SELECT COUNT(*) FROM ARTIST WHERE NAME =?1")
        .and_then(|mut statement| {
            statement.query_row([&body.artist.name], |row| {
                row.get(0).and_then(|count: i32| Ok(count == 1))
            })
        })
        .expect("Erreur lors de la vérification de présence de l'artiste");

    if is_disc_already_present {
        return HttpResponse::Conflict();
    }

    if !is_artist_already_present {
        connection
            .prepare("INSERT INTO ARTIST(NAME, PICTURE) VALUES(?1, ?2)")
            .and_then(|mut statement| statement.execute([&body.artist.name, &body.artist.picture]))
            .expect("Erreur lors de la création de l'artiste");
    }

    let artist_id : i32 = connection
        .prepare("SELECT ID FROM ARTIST WHERE NAME =?1")
        .and_then(|mut statement| statement.query_row([&body.artist.name], |row| row.get(0)))
        .expect("Erreur lors de la récupératon de l'ID de l'artiste");

    connection
        .prepare("INSERT INTO DISC(TITLE, COVER, ARTIST_ID) VALUES(?1, ?2, ?3)")
        .and_then(|mut statement| statement.execute((&body.title, &body.cover, &artist_id)))
        .expect("Erreur lors de la création du disque");
    HttpResponse::Created()
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
}

