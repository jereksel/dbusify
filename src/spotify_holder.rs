use rspotify;

use rspotify::spotify::oauth2::SpotifyOAuth;
use rspotify::spotify::client::Spotify;
use rspotify::spotify::oauth2::SpotifyClientCredentials;
use rspotify_hyper::get_token_hyper;
use std::sync::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SpotifyHolder {
    oauth: Arc<RwLock<SpotifyOAuth>>
}

impl SpotifyHolder {

    pub fn new(spotify: SpotifyOAuth) -> SpotifyHolder {
        return SpotifyHolder {
            oauth: Arc::new(RwLock::new(spotify))
        }
    }

    pub fn get_spotify(&self) -> Spotify {

        let mut a = self.oauth.write().unwrap();

        let oauth = &mut *a;

        let spotify = match get_token_hyper(oauth) {
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

        return spotify;

    }

}