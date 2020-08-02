use dbusify;
use rspotify;
use dbus;
use rspotify_hyper;

use rspotify::spotify::client::Spotify;
use dbus::ConnPath;
use dbus::Connection;
use std::{panic, env};
use rspotify::spotify::oauth2::SpotifyOAuth;
use rspotify::spotify::oauth2::SpotifyClientCredentials;
use std::sync::Arc;
use std::thread;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use dbus::BusType;

use dbusify::AccountType;
use rspotify_hyper::get_token_hyper;

pub fn run_test_type<T>(_type: AccountType, test: T) -> ()
    where T: FnOnce(Spotify, ConnPath<'_, &Connection>) -> () + panic::UnwindSafe
{

    let client_id = env::var("CLIENT_ID");
    let client_secret = env::var("CLIENT_SECRET");

    if client_id.is_err() || client_secret.is_err() {
        println!("CLIENT_ID or CLIENT_SECRET not provided");
        return;
    }

    const REDIRECT_URL: &'static str = "http://localhost:8888/callback";

    let oauth = SpotifyOAuth::default()
        .client_id(client_id.unwrap().as_str())
        .client_secret(client_secret.unwrap().as_str())
        .redirect_uri(REDIRECT_URL)
        .cache_path(_type.get_config_file())
        .scope("playlist-read-private playlist-read-collaborative playlist-modify-public playlist-modify-private streaming ugc-image-upload user-follow-modify user-follow-read user-library-read user-library-modify user-read-private user-read-birthdate user-read-email user-top-read user-read-playback-state user-modify-playback-state user-read-currently-playing user-read-recently-played")
        .build();

    let token_info = get_token_hyper(&mut (oauth.clone())).unwrap();

    let client_credential = SpotifyClientCredentials::default()
        .token_info(token_info)
        .build();

    let spotify = Spotify::default()
        .client_credentials_manager(client_credential)
        .build();

    let _spotify2 = spotify.clone();

    let running = Arc::new(AtomicBool::new(false));
    let running2 = running.clone();

    let job = thread::spawn( move || {

        let c = dbusify::get_connection(oauth);

        let r = running2.clone();

        loop {

            let y = r.clone();

            c.incoming(1000).next();

            if y.load(Ordering::Relaxed) {
                break;
            }
        }

        c.release_name("org.mpris.MediaPlayer2.dbusify").unwrap();

    });

    thread::sleep(Duration::from_millis(500));

    let c = Connection::get_private(BusType::Session).unwrap();
    let p = c.with_path("org.mpris.MediaPlayer2.dbusify", "/org/mpris/MediaPlayer2", 5000);

    if _type == AccountType::Main {
        spotify.volume(0, None).unwrap();

        thread::sleep(Duration::from_millis(500));
    }

    test(spotify.clone(), p);

    running.store(true, Ordering::Relaxed);

    job.join().unwrap();

    thread::sleep(Duration::from_millis(500));

    if _type == AccountType::Main {
        if spotify.current_playback(None).unwrap().unwrap().is_playing {
            spotify.pause_playback(None).unwrap();
        }
    }

}
