use actix_web::web::Data;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::collection::owned_disc::*;



pub fn retrieve_owned_discs(db_connection_pool: Data<Pool<SqliteConnectionManager>>) -> Vec<OwnedDisc> {
    let connection = db_connection_pool
        .get()
        .expect("Erreur à lxa récupération d'une connection dans le pool");
    let mut statement = connection
        .prepare("SELECT D.ID, D.TITLE, D.COVER, A.ID, A.NAME, A.PICTURE FROM DISC D INNER JOIN ARTIST A ON D.ARTIST_ID = A.ID")
        .expect("Impossible de préparer la requête de consultation des disques.");

    statement
        .query_map([], |row| {
            dbg!(&row);
            Ok(OwnedDisc {
                id: row.get(0).expect("Erreur lors de la récupération de l'id"),
                title: row.get(1).expect("Erreur lors de la récupération du titre"),
                cover: row
                    .get(2)
                    .expect("Erreur lors de la récupération de la couverture"),
                artist: OwnedArtist {
                    id: row.get(3).expect("Erreur lors de la récupération de l'id"),
                    name: row.get(4).expect("Erreur lors de la récupération du nom"),
                    picture: row
                        .get(5)
                        .expect("Erreut lors de la récupération de l'image de l'artiste"),
                },
            })
        })
        .and_then(Iterator::collect)
        .expect("Erreur lors de la consultation des disques")
}
