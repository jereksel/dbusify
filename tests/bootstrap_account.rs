extern crate rspotify;
extern crate dbus;
extern crate dbusify;

mod integration_tests_utils;

use std::path::PathBuf;
use rspotify::spotify::oauth2::SpotifyOAuth;
use rspotify::spotify::oauth2::SpotifyClientCredentials;
use rspotify::spotify::client::Spotify;
use integration_tests_utils::run_test_type;
use dbus::ConnPath;
use dbusify::AccountType;
use dbus::Connection;
use std::panic;

#[test]
//DESTRUCTIVE
#[ignore]
fn bootstrap() {

    run_test(|spotify, connection| {

        let user_id = spotify.me().unwrap().id;

        //Remove all playlists
        {
            let playlists = spotify.user_playlists(user_id.as_str(), None, None).unwrap();
            for playlist in playlists.items.iter() {
                let playlist_id = &playlist.id;
                spotify.user_playlist_unfollow(user_id.as_str(), playlist_id).unwrap();
            }
            ();
        }

        //Remove all songs
        {
            let songs = spotify.current_user_saved_tracks(None, None).unwrap().items;
            let ids: Vec<String> = songs.iter().map(|s| s.track.id.clone()).collect();
            println!("OLD IDS:");
            println!("{:?}", ids.clone());
            if ids.len() > 0 {
                spotify.current_user_saved_tracks_delete(ids).unwrap();
            }
        }

        //Add playlists
        {
            let data = vec!(
                ("Playlist 1", vec!("4R4uSnvRFeVdLQcX2zj8aU",
                                    "1uGHaW5WOyQUSR1oNKAydC",
                                    "4q8PHoRsPUB52LFylX8Ulz",
                                    "1vvNmPOiUuyCbgWmtc6yfm",
                                    "0XzkemXSiXJa7VgDFPfU4S",
                                    "5cR7culxUEPLhzIC0KWAH1",
                                    "32OlwWuMpZ6b0aN2RZOeMS",
                                    "3ZFTkvIE7kyPt6Nu3PEa7V",
                                    "5CtI0qwDJkDQGwXD1H1cLb",
                                    "2DDBGX5qlbBRZsAqAWfILT",
                                    "1CNJyTUh56oj3OCZOZ5way")),
                ("Playlist 2", vec!("4R4uSnvRFeVdLQcX2zj8aU")),
                ("Playlist 3", vec!("4R4uSnvRFeVdLQcX2zj8aU")),
                ("Playlist 4", vec!("4R4uSnvRFeVdLQcX2zj8aU")),
                ("Playlist 5", vec!("4R4uSnvRFeVdLQcX2zj8aU")),
                ("Playlist 6", vec!("4R4uSnvRFeVdLQcX2zj8aU")),
                ("Playlist 7", vec!("4R4uSnvRFeVdLQcX2zj8aU")),
                ("Playlist 8", vec!("4R4uSnvRFeVdLQcX2zj8aU")),
                ("Playlist 9", vec!("4R4uSnvRFeVdLQcX2zj8aU")),
                ("Playlist 10", vec!("4R4uSnvRFeVdLQcX2zj8aU")),
                ("Playlist 11", vec!("4R4uSnvRFeVdLQcX2zj8aU")),
            );

            for (playlist_name, songs) in data.iter() {
                let playlist = spotify.user_playlist_create(&user_id, playlist_name, true, "Desc".to_string()).unwrap();

                for song in songs.iter() {
                    spotify.user_playlist_add_tracks(&user_id, &playlist.id, &[song.to_string()], Some(0)).unwrap();
                }
            }
            ();
        }

        //Add songs
        {

            let songs = vec!(
                "4R4uSnvRFeVdLQcX2zj8aU",
                "1uGHaW5WOyQUSR1oNKAydC",
                "4q8PHoRsPUB52LFylX8Ulz",
                "1vvNmPOiUuyCbgWmtc6yfm"
            );

            for song in songs.iter() {
                spotify.current_user_saved_tracks_add(&[song.to_string()]).unwrap();
            }

        }

    });


}

fn run_test<T>(test: T) -> ()
    where T: FnOnce(Spotify, ConnPath<&Connection>) -> () + panic::UnwindSafe
{
    run_test_type(AccountType::Test, test);
}

