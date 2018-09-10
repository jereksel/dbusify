#![feature(duration_as_u128)]

extern crate dbusify;
extern crate rspotify;
extern crate dbus;
extern crate dbusify_hyper;

mod integration_tests_utils;
use std::path::PathBuf;
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
use dbusify_hyper::get_token_hyper;
use integration_tests_utils::run_test_type;
use dbusify::AccountType;

#[test]
fn get_playlist_count() {

    run_test(|_spotify, connection| {
        assert_eq!(connection.get_playlist_count().unwrap(), 11);
    });

}

#[test]
fn get_all_playlists_test() {

    run_test(|_spotify, connection| {

        // Error is returned on invalid order
        {
            let e = connection.get_playlists(0, 0, "asd", true).unwrap_err();
            assert_eq!(e.to_string(), "\"Order is invalid\"");
        }

        // Get first 5 playlists by name
        {
            let playlists = connection.get_playlists(0, 5, "Alphabetical", false).unwrap();
            let names: Vec<&String> = playlists.iter().map(|p| &p.1).collect();
            assert_eq!(names, vec!("Playlist 1", "Playlist 10", "Playlist 11", "Playlist 2", "Playlist 3"));
            ();
        }

        // Count above size works fine
        {
            let playlists = connection.get_playlists(4, 500, "Alphabetical", false).unwrap();
            let names: Vec<&String> = playlists.iter().map(|p| &p.1).collect();
            assert_eq!(names, vec!("Playlist 3", "Playlist 4", "Playlist 5", "Playlist 6", "Playlist 7", "Playlist 8", "Playlist 9"));
            ();
        }

        // Get first 5 playlist by name reversed
        {
            let playlists = connection.get_playlists(0, 5, "Alphabetical", true).unwrap();
            let names: Vec<&String> = playlists.iter().map(|p| &p.1).collect();
            assert_eq!(names, vec!("Playlist 9", "Playlist 8", "Playlist 7", "Playlist 6", "Playlist 5"));
            ();
        }

        //Get first 5 playlists by default Spotify ordering
        {
            let playlists = connection.get_playlists(0, 5, "UserDefined", false).unwrap();
            let names: Vec<&String> = playlists.iter().map(|p| &p.1).collect();
            assert_eq!(names, vec!("Playlist 11", "Playlist 10", "Playlist 9", "Playlist 8", "Playlist 7"));
        }

        //Get first 5 playlists by default Spotify ordering reversed
        {
            let playlists = connection.get_playlists(0, 5, "UserDefined", true).unwrap();
            let names: Vec<&String> = playlists.iter().map(|p| &p.1).collect();
            assert_eq!(names, vec!("Playlist 1", "Playlist 2", "Playlist 3", "Playlist 4", "Playlist 5"));
        }

    })


}


fn run_test<T>(test: T) -> ()
    where T: FnOnce(Spotify, ConnPath<&Connection>) -> () + panic::UnwindSafe
{
    run_test_type(AccountType::Test, test);
}