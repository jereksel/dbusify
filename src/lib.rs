extern crate config;
extern crate core;
extern crate dbus;
extern crate itertools;
extern crate rspotify;
extern crate rspotify_hyper;
extern crate dirs;

pub mod mpris;

pub mod mprisimpl;

pub mod spotify_holder;

use dbus::{BusType, Connection, NameFlag};

use config::*;
use rspotify::spotify::client::Spotify;
pub use rspotify::spotify::oauth2::SpotifyClientCredentials;
use rspotify::spotify::oauth2::SpotifyOAuth;
use rspotify_hyper::get_token_hyper;
use spotify_holder::SpotifyHolder;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub fn run() {

    const REDIRECT_URL: &'static str = "http://localhost:8888/callback";

    let (client_id, client_secret) = get_client();

    let oauth = SpotifyOAuth::default()
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
        }
    };

    let c = get_connection(oauth);

    // Serve other peers forever.
    loop {
        c.incoming(1000).next();
    }
}

pub fn get_client() -> (String, String) {

    let home = dirs::home_dir().unwrap();
    let settings_location = PathBuf::from(".config/dbusify/");

    let path = home.join(settings_location).join("config.toml");

    if !path.exists() {
        println!("Config file {} does not exist", path.display());
        std::process::exit(1);
    }

    let mut settings = Config::default();
    settings.merge(File::from(path)).unwrap();

    let client_id = match settings.get_str("CLIENT_ID") {
        Ok(value) => value,
        Err(value) => {
            println!("{}", value);
            std::process::exit(1);
        }
    };

    let client_secret = match settings.get_str("CLIENT_SECRET") {
        Ok(value) => value,
        Err(value) => {
            println!("{}", value);
            std::process::exit(1);
        }
    };

    return (client_id, client_secret)

}

pub fn get_connection(spotify_o_auth: SpotifyOAuth) -> Connection {
    let c = Connection::get_private(BusType::Session).unwrap();
    c.register_name(
        "org.mpris.MediaPlayer2.dbusify",
        NameFlag::ReplaceExisting as u32,
    )
    .unwrap();

    let f = dbus::tree::Factory::new_fn::<()>();

    let arc = Arc::new(spotify_o_auth.clone());

    let holder = SpotifyHolder::new(spotify_o_auth);

    let i1 = mprisimpl::player::get_interface(holder.clone(), &f);
    let i2 = mprisimpl::playlist::get_interface(holder.clone(), &f);
    let i3 = mprisimpl::tracklist::get_interface(holder.clone(), &f);

    let t = f.tree(()).add(
        f.object_path("/org/mpris/MediaPlayer2", ())
            .introspectable()
            .add(i1)
            .add(i2)
            .add(i3),
    );

    t.set_registered(&c, true).unwrap();

    c.add_handler(t);

    return c;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    Main,

    //For playlist data tests
    Test,
}

impl AccountType {
    pub fn get_config_file(&self) -> PathBuf {
        let file = match *self {
            AccountType::Main => PathBuf::from(".spotify_token_cache_main.json"),
            AccountType::Test => PathBuf::from(".spotify_token_cache_test.json"),
        };

        let home = dirs::home_dir().unwrap();
        let settings = PathBuf::from(".config/dbusify/");

        let path = home.join(settings).join(file);

        fs::create_dir_all(path.parent().unwrap()).unwrap();
        path
    }
}
