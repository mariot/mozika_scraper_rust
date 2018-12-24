extern crate clap;
extern crate mozika_scraper_rust;
extern crate postgres;

use mozika_scraper_rust::scrap_artists;
use mozika_scraper_rust::scrap_songs;

use clap::{App, Arg};
use postgres::{Connection, TlsMode};

fn main() {
    let matches = App::new("Mozika Scraper")
        .version("0.1.1")
        .author("Mariot Tsitoara <mariot.tsitoara@gmail.com>")
        .about("Scrap Lyrics with Rust")
        .arg(
            Arg::with_name("type")
                .short("t")
                .long("type")
                .takes_value(true)
                .required(true)
                .help("Type of data to be parsed. Must be 'artist' or 'song'"),
        )
        .arg(
            Arg::with_name("page")
                .short("p")
                .long("page")
                .takes_value(true)
                .required(true)
                .help("Page number when scraping artists. Artist ID when scraping songs"),
        )
        .get_matches();

    let page_type = matches.value_of("type").unwrap_or("song");
    let page_num: i32 = matches
        .value_of("page")
        .unwrap_or("0")
        .parse()
        .expect("Page number invalid");

    let conn = Connection::connect(
        "postgresql://mozikauser:zelda@localhost:5432/mozikarust",
        TlsMode::None,
    )
    .unwrap();

    let insert_artist_stmt = conn.prepare("INSERT INTO scraper_artist(hits, name, number_of_songs, songs_id, url) VALUES ($1, $2, $3, $4, $5) RETURNING id").unwrap();
    let insert_song_stmt = conn.prepare("INSERT INTO scraper_song(title, lyrics, artist_id, hits, url) VALUES ($1, $2, $3, $4, $5);").unwrap();

    match page_type {
        "artist" => {
            println!("Searching for artists in page {}", page_num);
            let artists = scrap_artists(page_num * 20);

            for artist in &artists {
                let rows = &insert_artist_stmt
                    .query(&[
                        &0i32,
                        &artist.name,
                        &artist.number_of_songs,
                        &artist.songs_id,
                        &artist.url,
                    ])
                    .unwrap();
                let id: i32 = rows.get(0).get("id");

                let songs = scrap_songs(artist.songs_id);
                for song in songs {
                    insert_song_stmt
                        .execute(&[&song.title, &song.lyrics, &id, &0i32, &song.url])
                        .unwrap();
                }
            }
        }
        "song" => {
            println!("Searching for songs with artist ID {}", page_num);
        }
        _ => {
            println!("Type must be 'artist' or 'song'");
        }
    }
}
