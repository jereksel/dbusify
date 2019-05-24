use mpris;
use dbus::tree::MethodErr;
use dbus::arg::Variant;
use dbus::arg::RefArg;
use std::collections::HashMap;
use rspotify::spotify::client::Spotify;
use rspotify::spotify::model::offset::for_uri;
use dbus::Path;
use dbus::arg;
use rspotify::spotify::senum::Type;
use rspotify::spotify::model::playlist::FullPlaylist;
use rspotify::spotify::model::track::FullTrack;
use rspotify::spotify::model::album::FullAlbum;
use rspotify::spotify::model::track::SimplifiedTrack;
use std::time::Instant;

use super::super::spotify_holder::SpotifyHolder;

extern crate std;
extern crate dbus;

static mut SPOTIFY: Option<SpotifyHolder> = None;

fn get_spotify() -> Spotify {
    unsafe {
        match SPOTIFY {
            Some(ref x) => x.get_spotify(),
            None => panic!(),
        }
    }
}

impl mpris::tracklist::OrgMprisMediaPlayer2TrackList for () {
    type Err = dbus::tree::MethodErr;

    fn get_tracks_metadata(&self, track_ids: Vec<Path>) -> Result<Vec<::std::collections::HashMap<String, arg::Variant<Box<arg::RefArg>>>>, Self::Err> {
        get_current_tracks()
            .map(|tracks| {
                tracks.iter()
                    .filter(|track| {
                        let dbus_path = track_to_dbus_path(track);
                        track_ids.contains(&dbus_path)
                    })
                    .map(|track| {
                        let mut map = HashMap::new();
                        let name = track.name.clone();
                        map.insert("mpris:trackid".to_string(), Variant(Box::new("/org/mpris/MediaPlayer2/Track/".to_string() + &track.id) as Box<RefArg>));
                        map.insert("xesam:title".to_string(), Variant(Box::new(name) as Box<RefArg>));
                        map
                    })
                    .collect()
            })
            .map_err(|err| MethodErr::failed(&err.clone()))
    }

    fn add_track(&self, _uri: &str, _after_track: Path, _set_as_current: bool) -> Result<(), Self::Err> {
        Err(MethodErr::failed(&"adding track is not supported"))
    }

    fn remove_track(&self, _track_id: Path) -> Result<(), Self::Err> {
        Err(MethodErr::failed(&"removing track is not supported"))
    }

    fn go_to(&self, track_id: Path) -> Result<(), Self::Err> {

        let start = "/org/mpris/MediaPlayer2/Track/";

        let song = track_id.to_string()[start.len()..].to_string();
        let uri = "spotify:track:".to_string() + &song;

        get_spotify().current_playback(None)
            .map_err(|e| e.to_string())
            .and_then(|r| r.ok_or("Playback is not available".to_string()))
            .and_then(|r| r.context.ok_or("Playback is not available".to_string()))
            .and_then(|context| {
                get_spotify().start_playback(None, Some(context.uri), None, for_uri(uri))
                    .map_err(|e| e.to_string())
            })
            .map_err(|err| MethodErr::failed(&err.clone()))

    }

    fn get_tracks(&self) -> Result<Vec<Path<'static>>, Self::Err> {

        get_current_tracks()
            .map(|songs| {
                songs.iter()
                    .map(|song| track_to_dbus_path(song))
                    .collect()
            })
            .map_err(|err| MethodErr::failed(&err.clone()))

    }

    fn get_can_edit_tracks(&self) -> Result<bool, Self::Err> {
        Ok(false)
    }
}

fn get_current_tracks() -> Result<Vec<SimplifiedTrack>, String> {

    let context = get_spotify().current_playback(None)
        .map_err(|e| e.to_string())
        .and_then(|r| r.ok_or("Playback is not available".to_string()))
        .and_then(|r| r.context.ok_or("Playback is not available".to_string()));

    match context {
        Ok(context) => {
            match context._type {
                Type::Playlist => {

                    let d: Vec<&str> = context.uri.split(':').collect();

                    // https://developer.spotify.com/community/news/2018/06/12/changes-to-playlist-uris/
                    let id = (d[4]).to_string();

                    get_spotify().playlist(&id, None, None)
                        .map_err(|err| err.to_string())
                        .and_then(|playlist| get_all_tracks_for_playlist(playlist))
                },
                Type::Album => {

                    let d: Vec<&str> = context.uri.split(':').collect();

                    let id = (d[2]).to_string();

                    get_spotify().album(&context.uri)
                        .map_err(|err| err.to_string())
                        .and_then(|album| get_all_tracks_for_album(album))

                },
                _ => {
                    Err("Not implemented".to_string())
                }
            }
        },
        Err(err) => {
            Err(err.to_string())
        },
    }

}

fn track_to_dbus_path(song: &SimplifiedTrack) -> dbus::Path<'static> {
    dbus::Path::from("/org/mpris/MediaPlayer2/Track/".to_string() + &song.id)
}

fn get_all_tracks_for_album(album: FullAlbum) -> Result<Vec<SimplifiedTrack>, String> {
    let mut vec = Vec::new();
    let mut current_num = 0;

    let spotify = get_spotify();
    let step = 2;

    loop {

        match spotify.album_track(album.id.as_str(), None, current_num) {
            Ok(songs) => {

                for song in songs.items {
                    vec.push(song);
                }

                current_num += songs.limit;

                if songs.next.is_none() {
                    break
                }


            },
            Err(err) => {
                return Err(err.to_string());
            },
        }

    }

    return Ok(vec);
}

fn get_all_tracks_for_playlist(playlist: FullPlaylist) -> Result<Vec<SimplifiedTrack>, String> {
    let mut vec = Vec::new();
    let mut current_num = 0;

    let spotify = get_spotify();
    let step = 2;

    let start = Instant::now();

    loop {

        match spotify.user_playlist_tracks(&playlist.owner.id, &playlist.id, None, Some(100), current_num, None) {
            Ok(songs) => {

                for song in songs.items {
                    vec.push(full_to_simplified(song.track));
                }

                current_num += songs.limit;

                if songs.next.is_none() {
                    break
                }


            },
            Err(err) => {
                return Err(err.to_string());
            },
        }

    }

    let elapsed = start.elapsed();

    println!("All tracks: {:?}", elapsed);

    return Ok(vec);
}

fn full_to_simplified(track: FullTrack) -> SimplifiedTrack {
    SimplifiedTrack {
        artists: track.artists,
        available_markets: Some(track.available_markets),
        disc_number: track.disc_number,
        duration_ms: track.duration_ms,
        //Not available in FullTrack
        explicit: false,
        external_urls: track.external_urls,
        href: track.href,
        id: track.id,
        name: track.name,
        preview_url: track.preview_url,
        track_number: track.track_number,
        _type: track._type,
        uri: track.uri
    }
}

pub fn get_interface(spotify: SpotifyHolder, f: &dbus::tree::Factory<dbus::tree::MTFn<()>, ()>) -> dbus::tree::Interface<dbus::tree::MTFn, ()> {
    unsafe {
        SPOTIFY = Some(spotify);
    }

    let i1 = mpris::tracklist::org_mpris_media_player2_track_list_server(f, (), |minfo| minfo.path.get_data());
    return i1;
}

