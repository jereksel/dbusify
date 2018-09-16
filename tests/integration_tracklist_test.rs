#![feature(duration_as_u128)]

extern crate dbusify;
extern crate rspotify;
extern crate dbus;

mod integration_tests_utils;

use std::sync::atomic::Ordering;
use rspotify::spotify::client::Spotify;
use std::panic;
use rspotify::spotify::oauth2::SpotifyOAuth;
use rspotify::spotify::util::get_token;
use rspotify::spotify::oauth2::SpotifyClientCredentials;
use std::sync::Arc;
use std::thread;
use std::sync::atomic::AtomicBool;
use dbus::Connection;
use dbus::BusType;
use dbus::ConnPath;
use dbusify::mpris::tracklist::OrgMprisMediaPlayer2TrackList;
use std::time::Duration;
use rspotify::spotify::model::offset::for_position;
use dbus::arg::RefArg;
use dbusify_hyper::get_token_hyper;
use dbusify::AccountType;
use integration_tests_utils::run_test_type;

#[test]
fn get_tracks_playlist() {

    run_test(|spotify, connection| {

        let album = "spotify:user:39svv1259gc91te80t8aik838:playlist:3fBmhRB02rrzntJy3dKbyz".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.pause_playback(None).unwrap();

        let tracks_path = connection.get_tracks().unwrap();
        let tracks_str: Vec<String> = tracks_path.iter().map(|p| p.to_string()).collect();

        assert_eq!(tracks_str, vec!("/org/mpris/MediaPlayer2/Track/1CNJyTUh56oj3OCZOZ5way",
                                    "/org/mpris/MediaPlayer2/Track/2DDBGX5qlbBRZsAqAWfILT",
                                    "/org/mpris/MediaPlayer2/Track/5CtI0qwDJkDQGwXD1H1cLb",
                                    "/org/mpris/MediaPlayer2/Track/3ZFTkvIE7kyPt6Nu3PEa7V",
                                    "/org/mpris/MediaPlayer2/Track/32OlwWuMpZ6b0aN2RZOeMS",
                                    "/org/mpris/MediaPlayer2/Track/5cR7culxUEPLhzIC0KWAH1",
                                    "/org/mpris/MediaPlayer2/Track/0XzkemXSiXJa7VgDFPfU4S",
                                    "/org/mpris/MediaPlayer2/Track/1vvNmPOiUuyCbgWmtc6yfm",
                                    "/org/mpris/MediaPlayer2/Track/4q8PHoRsPUB52LFylX8Ulz",
                                    "/org/mpris/MediaPlayer2/Track/1uGHaW5WOyQUSR1oNKAydC",
                                    "/org/mpris/MediaPlayer2/Track/4R4uSnvRFeVdLQcX2zj8aU")
        );

    })

}

#[test]
fn get_tracks_album() {

    run_test(|spotify, connection| {

        let album = "spotify:album:2fMClGSlcEN5YExJlAxiEs".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.pause_playback(None).unwrap();

        let tracks_path = connection.get_tracks().unwrap();
        let tracks_str: Vec<String> = tracks_path.iter().map(|p| p.to_string()).collect();

        assert_eq!(tracks_str, vec!("/org/mpris/MediaPlayer2/Track/1pVGpXbHxexaXFoXdc4V4k",
                                    "/org/mpris/MediaPlayer2/Track/15tmj1kWeM7yKHtLQ2M7Nw",
                                    "/org/mpris/MediaPlayer2/Track/0WnvyIR27ZMpmvRdbS5vdb",
                                    "/org/mpris/MediaPlayer2/Track/2lzoH8yQ9a8bNMcplLGxUm",
                                    "/org/mpris/MediaPlayer2/Track/6E8oFpY3hTObh2g68AeQz9",
                                    "/org/mpris/MediaPlayer2/Track/3rYFZkYDs7mLeMk2Azeshe",
                                    "/org/mpris/MediaPlayer2/Track/6u0Vl90yC88fRg0O6xyKar",
                                    "/org/mpris/MediaPlayer2/Track/6LtidBuVR9Nn66sPliFOJ1",
                                    "/org/mpris/MediaPlayer2/Track/59Z54YFlIpP2QcpeIkzTrn",
                                    "/org/mpris/MediaPlayer2/Track/4uhI6JckZryQWo4DmYLhPy",
                                    "/org/mpris/MediaPlayer2/Track/4mKoCSdAw2WUGR5fjvsp0B",
                                    "/org/mpris/MediaPlayer2/Track/0G0KhUEv0IB3JH0Ep34545",
                                    "/org/mpris/MediaPlayer2/Track/7w0opqbalIha87jnkvH9Eu",
                                    "/org/mpris/MediaPlayer2/Track/7svZtW9CazHbB0dDo8smFb",
                                    "/org/mpris/MediaPlayer2/Track/1C1RdX4hAa7SAvN6RNZxMM",
                                    "/org/mpris/MediaPlayer2/Track/7Ddw3aLPY3Z4hf6AJLFUkx",
                                    "/org/mpris/MediaPlayer2/Track/66Gpo12lba0gkCSQsMFgiR",
                                    "/org/mpris/MediaPlayer2/Track/4f9YBR6sv63JuQPvgcESGm",
                                    "/org/mpris/MediaPlayer2/Track/1Ly48PEd4lBO5OA6BcVR36",
                                    "/org/mpris/MediaPlayer2/Track/6T5wHmGd5Ba2W6HtowQ1PQ")
        )

    })

}

#[test]
fn get_tracks_metadata() {

    run_test(|spotify, connection| {

        let album = "spotify:album:2fMClGSlcEN5YExJlAxiEs".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.pause_playback(None).unwrap();


        let tracks_metadata = connection.get_tracks_metadata(vec!(
            dbus::Path::from("/org/mpris/MediaPlayer2/Track/1pVGpXbHxexaXFoXdc4V4k"),
            dbus::Path::from("/org/mpris/MediaPlayer2/Track/15tmj1kWeM7yKHtLQ2M7Nw")
        )).unwrap();

        let track_0 = &tracks_metadata[0];

        assert_eq!(track_0["mpris:trackid"].0.as_str().unwrap(), "/org/mpris/MediaPlayer2/Track/1pVGpXbHxexaXFoXdc4V4k");
        assert_eq!(track_0["xesam:title"].0.as_str().unwrap(), "Infernus Ad Astra");

        let track_1 = &tracks_metadata[1];

        assert_eq!(track_1["mpris:trackid"].0.as_str().unwrap(), "/org/mpris/MediaPlayer2/Track/15tmj1kWeM7yKHtLQ2M7Nw");
        assert_eq!(track_1["xesam:title"].0.as_str().unwrap(), "Rise of the Chaos Wizards");

    })

}

#[test]
fn go_to() {

    run_test(|spotify, connection| {

        let album = "spotify:album:2fMClGSlcEN5YExJlAxiEs".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.pause_playback(None).unwrap();

        let song_uri = "/org/mpris/MediaPlayer2/Track/6E8oFpY3hTObh2g68AeQz9";

        connection.go_to(dbus::Path::from(song_uri.to_string())).unwrap();
        thread::sleep(Duration::from_millis(500));

        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, "spotify:track:6E8oFpY3hTObh2g68AeQz9");

    })

}

fn run_test<T>(test: T) -> ()
    where T: FnOnce(Spotify, ConnPath<&Connection>) -> () + panic::UnwindSafe
{
    run_test_type(AccountType::Main, test);
}