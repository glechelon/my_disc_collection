use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::{get, web, HttpRequest};
use handlebars::Handlebars;
use qstring::QString;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::retrieve_owned_discs;
use crate::OwnedDisc;

#[derive(Deserialize, Debug)]
struct SearchResponse {
    data: Vec<SearchData>,
}

#[derive(Deserialize, Serialize, Debug)]
struct SearchData {
    id: i32,
    title: String,
    r#type: String,
    cover_medium: String,
    artist: SearchArtist,
    isOwned: Option<bool>,
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
async fn search(template_registry: Data<Handlebars<'static>>) -> impl Responder {
    let data: HashMap<String, String> = HashMap::new();
    let render = template_registry
        .render("recherche", &data)
        .expect("Erreur lors du rendu du template");
    HttpResponse::Ok().body(render)
}

#[get("parts/disc-search-results")]
async fn discs(
    req: HttpRequest,
    template_registry: Data<Handlebars<'static>>,
    db_connection_pool: Data<Pool<SqliteConnectionManager>>,
) -> impl Responder {
    dbg!(req.query_string());
    let nom_album_recherche = QString::from(req.query_string())
        .get("name")
        .expect("Erreur lors de la récupération de la requête")
        .replace(" ", "%20");
    dbg!(&nom_album_recherche);
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

    let search_results = json.data
        .iter()
        .map(|response| SearchData {
            id: response.id,
            title: response.title.clone(),
            r#type: response.r#type.clone(),
            cover_medium: response.cover_medium.clone(),
            artist: response.artist.clone(),
            isOwned: (|| {
                if my_collection.iter().any(|owned_disc| {
                    owned_disc.title.eq(&response.title)
                        && owned_disc.artist.name.eq(&response.artist.name)
                }) {
                    Some(true)
                } else {
                    Some(false)
                }
            })()
        })
        .collect::<Vec<SearchData>>();

    let mut data: HashMap<String, Vec<SearchData>> = HashMap::new();
    data.insert("disc_search_results".to_string(), search_results);

    let render = template_registry
        .render("disc_search", &data)
        .expect("Erreur");

    HttpResponse::Ok().body(render)
}
