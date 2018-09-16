extern crate rspotify;
extern crate itertools;
extern crate dbus;
extern crate core;

pub mod rspotify_hyper;

pub mod mpris;

pub mod mprisimpl;

pub mod spotify_holder;

use dbus::{Connection, BusType, NameFlag};

use rspotify::spotify::oauth2::SpotifyOAuth;
pub use rspotify::spotify::oauth2::SpotifyClientCredentials;
use rspotify::spotify::client::Spotify;
use std::sync::Arc;
use std::sync::RwLock;
use rspotify_hyper::get_token_hyper;
use std::path::PathBuf;
use std::env;
use std::fs;
use std::env::home_dir;
use spotify_holder::SpotifyHolder;

pub fn run() {

    let client_id = env::var("CLIENT_ID").unwrap();
    let client_secret = env::var("CLIENT_SECRET").unwrap();
    const REDIRECT_URL: &'static str = "http://localhost:8888/callback";

    let mut oauth = SpotifyOAuth::default()
        .client_id(client_id.as_str())
        .client_secret(client_secret.as_str())
        .redirect_uri(REDIRECT_URL)
        .cache_path(AccountType::Main.get_config_file())
        .scope("playlist-read-private playlist-read-collaborative playlist-modify-public playlist-modify-private streaming ugc-image-upload user-follow-modify user-follow-read user-library-read user-library-modify user-read-private user-read-birthdate user-read-email user-top-read user-read-playback-state user-modify-playback-state user-read-currently-playing user-read-recently-played")
        .build();

    //To open website one time
    let spotify = match get_token_hyper(&mut oauth.clone()) {
        Some(token_info) => {
            let client_credential = SpotifyClientCredentials::default()
                .token_info(token_info)
                .build();

            Spotify::default()
                .client_credentials_manager(client_credential)
                .build()

        }
        None => {
            panic!("auth failed");
        },
    };

    let c = get_connection(oauth);

    // Serve other peers forever.
    loop { c.incoming(1000).next(); }

}


pub fn get_connection(spotify_o_auth: SpotifyOAuth) -> Connection {

    let c = Connection::get_private(BusType::Session).unwrap();
    c.register_name("org.mpris.MediaPlayer2.dbusify", NameFlag::ReplaceExisting as u32).unwrap();

    let f = dbus::tree::Factory::new_fn::<()>();

    let arc = Arc::new(spotify_o_auth.clone());

    let holder = SpotifyHolder::new(spotify_o_auth);

    let i1 = mprisimpl::player::get_interface(holder.clone(), &f);
    let i2 = mprisimpl::playlist::get_interface(holder.clone(), &f);
    let i3 = mprisimpl::tracklist::get_interface(holder.clone(), &f);

    let t = f.tree(()).add(f.object_path("/org/mpris/MediaPlayer2", ()).introspectable().add(i1).add(i2).add(i3));

    t.set_registered(&c, true).unwrap();

    c.add_handler(t);

    return c;

}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    Main,

    //For playlist data tests
    Test
}

impl AccountType {

    pub fn get_config_file(&self) -> PathBuf {
        let file = match *self {
            AccountType::Main => {PathBuf::from(".spotify_token_cache_main.json")},
            AccountType::Test => {PathBuf::from(".spotify_token_cache_test.json")}
        };

        let home = home_dir().unwrap();
        let settings = PathBuf::from(".config/dbusify/");

        let path = home.join(settings).join(file);

        fs::create_dir_all(path.parent().unwrap()).unwrap();
        path
    }

}
