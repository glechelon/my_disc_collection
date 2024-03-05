use actix_web::web::{self, Data};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::collection::owned_disc::*;

pub fn retrieve_owned_discs(
    db_connection_pool: Data<Pool<SqliteConnectionManager>>,
) -> Vec<OwnedDisc> {
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

pub fn is_disc_owned(
    db_connection_pool: &Data<Pool<SqliteConnectionManager>>,
    title: &String,
) -> bool {
    let connection = db_connection_pool
        .get()
        .expect("Erreur à la récupération d'une connexion dans le pool");
    connection
        .prepare("SELECT COUNT(*) FROM DISC WHERE TITLE =?1")
        .and_then(|mut statement| {
            statement.query_row([&title], |row| {
                row.get(0).and_then(|count: i32| Ok(count == 1))
            })
        })
        .expect("Erreur lors de la vérification de présence du disque")
}

pub fn is_artist_owned(
    db_connection_pool: &Data<Pool<SqliteConnectionManager>>,
    name: &String,
) -> bool {
    let connection = db_connection_pool
        .get()
        .expect("Erreur à la récupération d'une connexion dans le pool");
    connection
        .prepare("SELECT COUNT(*) FROM ARTIST WHERE NAME =?1")
        .and_then(|mut statement| {
            statement.query_row([&name], |row| {
                row.get(0).and_then(|count: i32| Ok(count == 1))
            })
        })
        .expect("Erreur lors de la vérification de présence de l'artiste")
}

pub fn create_artist(db_connection_pool: &Data<Pool<SqliteConnectionManager>>, artist: &OwnedArtist) {
    let connection = db_connection_pool
        .get()
        .expect("Erreur à la récupération d'une connexion dans le pool");
    connection
        .prepare("INSERT INTO ARTIST(NAME, PICTURE) VALUES(?1, ?2)")
        .and_then(|mut statement| statement.execute([&artist.name, &artist.picture]))
        .expect("Erreur lors de la création de l'artiste");
}

pub fn find_artist_id_by_name(
    db_connection_pool: &Data<Pool<SqliteConnectionManager>>,
    name: &String,
) -> i32 {
    let connection = db_connection_pool
        .get()
        .expect("Erreur à la récupération d'une connexion dans le pool");
    connection
        .prepare("SELECT ID FROM ARTIST WHERE NAME =?1")
        .and_then(|mut statement| statement.query_row([&name], |row| row.get(0)))
        .expect("Erreur lors de la récupératon de l'ID de l'artiste")
}

pub fn create_disc(db_connection_pool: &Data<Pool<SqliteConnectionManager>>, disc: &OwnedDisc, artist_id : &i32) {
    let connection = db_connection_pool
        .get()
        .expect("Erreur à la récupération d'une connexion dans le pool");
    connection
        .prepare("INSERT INTO DISC(TITLE, COVER, ARTIST_ID) VALUES(?1, ?2, ?3)")
        .and_then(|mut statement| statement.execute((&disc.title, &disc.cover, &artist_id)))
        .expect("Erreur lors de la création du disque");
}
