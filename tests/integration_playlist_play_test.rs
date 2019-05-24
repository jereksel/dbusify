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
use dbusify::mpris::playlists::OrgMprisMediaPlayer2Playlists;
use std::time::Duration;
use rspotify::spotify::model::offset::for_position;
use integration_tests_utils::run_test_type;
use dbusify::AccountType;

#[test]
fn get_active_playlist() {

    run_test(|spotify, connection| {

        let album = "spotify:album:2kyTLcEZe6nc1s6ve0zW9P".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(500));

        assert_eq!(connection.get_active_playlist().unwrap().0, false);

        let playlist = "spotify:user:wizzler:playlist:00wHcTN0zQiun4xri9pmvX".to_string();
        spotify.start_playback(None, Some(playlist), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(500));

        let active = connection.get_active_playlist().unwrap();
        assert_eq!(active.0, true);
        assert_eq!((active.1).1, "Movie Soundtrack Masterpieces");

        //TODO
        assert_eq!((active.1).2, "");

    });

}

#[test]
fn activate_playlist() {

    run_test(|_spotify, connection| {

        let playlist = &connection.get_playlists(0, 1, "Alphabetical", false).unwrap()[0];

        let playlist_path = &playlist.0;

        connection.activate_playlist(playlist_path.clone()).unwrap();
        thread::sleep(Duration::from_millis(500));

        let active_playlist = connection.get_active_playlist().unwrap();

        assert_eq!(true, active_playlist.0);
        assert_eq!(playlist_path.to_string(), (active_playlist.1).0.to_string());

    });

}

fn run_test<T>(test: T) -> ()
    where T: FnOnce(Spotify, ConnPath<&Connection>) -> () + panic::UnwindSafe
{
    run_test_type(AccountType::Main, test);
}