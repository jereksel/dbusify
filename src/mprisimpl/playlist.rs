use mpris;
use dbus::tree::MethodErr;
use rspotify::spotify::client::Spotify;
use rspotify::spotify::senum::Type;
use rspotify::spotify::model::playlist::SimplifiedPlaylist;
use std::cmp::min;
use std::sync::Arc;
use std::borrow::Borrow;
use std::time::Instant;

extern crate std;
extern crate dbus;

static mut SPOTIFY: Option<Arc<Spotify>> = None;

fn get_spotify() -> &'static Spotify {
    unsafe {
        match SPOTIFY {
            Some(ref x) => x.borrow(),
            None => panic!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ordering {
    Alphabetical,
    UserDefined
}

impl Ordering {
    pub fn from_str(s: &str) -> Option<Ordering> {
        match s {
            "Alphabetical" => Some(Ordering::Alphabetical),
            "UserDefined" => Some(Ordering::UserDefined),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &str {
        match *self {
            Ordering::Alphabetical => "Alphabetical",
            Ordering::UserDefined => "UserDefined"
        }
    }
}

type Playlist = (dbus::Path<'static>, String, String);

impl mpris::playlists::OrgMprisMediaPlayer2Playlists for () {
    type Err = dbus::tree::MethodErr;

    fn activate_playlist(&self, playlist_id: dbus::Path) -> Result<(), Self::Err> {

        let start = "/org/mpris/MediaPlayer2/Playlist/";

        let playlist = playlist_id.to_string()[start.len()..].to_string();
        let uri = "spotify:playlist:".to_string() + &playlist;

        get_spotify().playlist(&playlist, None)
            .and_then(|playlist| get_spotify().start_playback(None, Some(playlist.uri), None, None))
            .map_err(|err| MethodErr::failed(&err.to_string()))

    }

    fn get_playlists(&self, index: u32, max_count: u32, order: &str, reverse_order: bool) -> Result<Vec<(dbus::Path<'static>, String, String)>, Self::Err> {

        match Ordering::from_str(order) {
            Some(ordering) => {

                match get_all_playlists() {
                    Ok(playlists) => {

                        let mut playlists: Vec<Playlist> = playlists.iter()
                            .map(|playlist| {
                                let dbus_path = "/org/mpris/MediaPlayer2/Playlist/".to_string() + &playlist.id.to_string();
                                (dbus::Path::from(dbus_path), playlist.name.to_string(), "".to_string())
                            }).collect();


                        let mut playlists: Vec<Playlist> = match ordering {
                            Ordering::UserDefined => playlists,
                            Ordering::Alphabetical => {
                                playlists.sort_by_key(|playlist| playlist.1.to_string());
                                playlists
                            }
                        };

                        let playlists: Vec<Playlist> = if reverse_order {
                            playlists.reverse();
                            playlists
                        } else {
                            playlists
                        };

                        let last_index = min((index + max_count) as usize, playlists.len());

                        let a = &playlists[(index as usize)..(last_index as usize)];

                        Ok(a.to_vec())
                    },
                    Err(e) => {
                        Err(MethodErr::failed(&e))
                    },
                }

            },
            None => {
                Err(MethodErr::failed(&"Order is invalid"))
            },
        }

    }

    fn get_playlist_count(&self) -> Result<u32, Self::Err> {
        get_spotify().current_user_playlists(0, 0)
            .map_err(|err| MethodErr::failed(&err.to_string()))
            .map(|playlist| playlist.total)
    }

    fn get_orderings(&self) -> Result<Vec<String>, Self::Err> {
        Ok(vec!("Alphabetical".to_string(), "UserDefined".to_string()))
    }

    fn get_active_playlist(&self) -> Result<(bool, (dbus::Path<'static>, String, String)), Self::Err>{

        match get_spotify().current_playing(None) {
            Ok(result) => {
                match result {
                    Some(playback) => {
                        match playback.context {
                            Some(context) => {
                                let _type = context._type;

                                let is_playlist = match _type {
                                    Type::Artist => { false },
                                    Type::Album => { false },
                                    Type::Track => { false },
                                    Type::Playlist => { true },
                                    Type::User => { false },
                                };

                                if is_playlist {

                                    let d: Vec<&str> = context.uri.split(':').collect();

                                    // https://developer.spotify.com/community/news/2018/06/12/changes-to-playlist-uris/
                                    let id = (d[4]).to_string();

                                    get_spotify().playlist(&id, None)
                                        .map_err(|err| MethodErr::failed(&err.to_string()))
                                        .map(|playlist| {
                                            let dbus_path = "/org/mpris/MediaPlayer2/Playlist/".to_string() + &id.to_string();
                                            (true, (dbus::Path::from(dbus_path), playlist.name, "".to_string()))
                                        })

                                } else {
                                    Ok((false, (dbus::Path::from("/org/mpris/MediaPlayer2/Playlist/Nothing".to_string()), "".to_string(), "".to_string())))
                                }

                            },
                            None => {
                                Err(MethodErr::failed(&"Playback is empty"))
                            },
                        }
                    },
                    None => {
                        Err(MethodErr::failed(&"Playback is empty"))
                    },
                }
            },
            Err(error) => {
                Err(MethodErr::failed(&error.to_string()))
            }
        }

    }

}

pub fn get_all_playlists() -> Result<Vec<SimplifiedPlaylist>, String> {
    let mut vec = Vec::new();
    let mut current_num = 0;

    let spotify = get_spotify();
    let step = 2;

    let start = Instant::now();

    match spotify.me() {
        Ok(user) => {
            let user_id = user.id;

            loop {

                match spotify.user_playlists(&user_id.clone(), None, current_num) {
                    Ok(playlists) => {
                        for playlist in playlists.items {
                            vec.push(playlist.clone())
                        }

                        current_num += playlists.limit;

                        if playlists.next.is_none() {
                            break
                        }

                    },
                    Err(err) => {
                        return Err(err.to_string());
                    },
                }

            }

        },
        Err(err) => {
            return Err(err.to_string());
        },
    }

    let elapsed = start.elapsed();

    println!("All playlists: {:?}", elapsed);

    return Ok(vec);
}

pub fn get_interface(spotify: Arc<Spotify>, f: &dbus::tree::Factory<dbus::tree::MTFn<()>, ()>) -> dbus::tree::Interface<dbus::tree::MTFn, ()> {
    unsafe {
        SPOTIFY = Some(spotify);
    }

    let i1 = mpris::playlists::org_mpris_media_player2_playlists_server(f, (), |minfo| minfo.path.get_data());
    return i1;
}
