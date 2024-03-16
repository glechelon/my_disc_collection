use actix_session::Session;
use actix_web::cookie::Cookie;
use actix_web::web::Data;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::{get, web, HttpRequest};
use handlebars::Handlebars;
use qstring::QString;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::retrieve_owned_discs;
use crate::OwnedDisc;


const CURRENT_SEARCH_STR :&str = "current-search";



#[derive(Deserialize, Debug)]
struct SearchResponse {
    data: Vec<SearchData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct SearchData {
    id: i32,
    title: String,
    r#type: String,
    cover_medium: String,
    artist: SearchArtist,
    is_owned: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct SearchArtist {
    name: String,
    picture_small: String,
}

pub(crate) fn configure_search(config: &mut web::ServiceConfig) {
    config.service(search).service(discs);
}

#[get("views/search")]
async fn search(template_registry: Data<Handlebars<'static>>, session: Session) -> impl Responder {
    let mut data: HashMap<String, String> = HashMap::new();
    session
        .get(CURRENT_SEARCH_STR)
        .expect("Erreur lors de la récupération de la recherche actuelle dans la session")
        .map(|current_search| {
            data.insert(CURRENT_SEARCH_STR.to_string(), current_search)
        });
    let render = template_registry
        .render("recherche", &data)
        .expect("Erreur lors du rendu du template");
    if data.contains_key(CURRENT_SEARCH_STR) {
        return HttpResponse::Ok()
            .insert_header(("HX-Trigger-After-Settle", "refreshSearch"))
            .body(render);
    }
    HttpResponse::Ok().body(render)
}

#[get("parts/disc-search-results")]
async fn discs(
    req: HttpRequest,
    template_registry: Data<Handlebars<'static>>,
    db_connection_pool: Data<Pool<SqliteConnectionManager>>,
    session: Session,
) -> Result<HttpResponse, Error> {
    let nom_album_recherche = QString::from(req.query_string())
        .get("name")
        .expect("Erreur lors de la récupération de la requête")
        .replace(" ", "%20");
    let search_results =
        reqwest::get("https://api.deezer.com/search/album?q=".to_string() + &nom_album_recherche)
            .await
            .expect("Erreur pedant la recherche")
            .text()
            .await
            .expect("Erreur lors de la lecture du résultat");
    let json: SearchResponse =
        serde_json::from_str(&search_results).expect("Erreur lors de la déserialisation");

    let my_collection: Vec<OwnedDisc> = retrieve_owned_discs(db_connection_pool);

    let search_results = json
        .data
        .iter()
        .map(|response| SearchData {
            id: response.id,
            title: response.title.clone(),
            r#type: response.r#type.clone(),
            cover_medium: response.cover_medium.clone(),
            artist: response.artist.clone(),
            is_owned: (|| {
                if my_collection.iter().any(|owned_disc| {
                    owned_disc.title.eq(&response.title)
                        && owned_disc.artist.name.eq(&response.artist.name)
                }) {
                    Some(true)
                } else {
                    Some(false)
                }
            })(),
        })
        .collect::<Vec<SearchData>>();

    let mut data: HashMap<String, Vec<SearchData>> = HashMap::new();

    session
        .insert("current-search", nom_album_recherche)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    data.insert("disc_search_results".to_string(), search_results);
    let render = template_registry
        .render("disc_search", &data)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(render))
}
