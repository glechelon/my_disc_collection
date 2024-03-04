use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::{get, web, HttpRequest};
use handlebars::Handlebars;
use qstring::QString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

#[derive(Deserialize, Serialize, Debug)]
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
async fn discs(req: HttpRequest, template_registry: Data<Handlebars<'static>>) -> impl Responder {
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
  //  dbg!(&search_results);
    let json: SearchResponse =
        serde_json::from_str(&search_results).expect("Erreur lors de la déserialisation");

    let mut data: HashMap<String, Vec<SearchData>> = HashMap::new();
    data.insert("disc_search_results".to_string(), json.data);

    let render = template_registry
        .render("disc_search", &data)
        .expect("Erreur");

    HttpResponse::Ok().body(render)
}
