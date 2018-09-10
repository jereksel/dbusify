use mpris;
use dbus::tree::MethodErr;
use rspotify::spotify::senum::RepeatState;
use dbus::arg::Variant;
use dbus::arg::RefArg;
use std::collections::HashMap;
use rspotify::spotify::client::Spotify;
use std::thread;
use rspotify::spotify::model::offset::for_uri;
use std::sync::Arc;
use std::borrow::Borrow;
use std::time::Duration;

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

impl mpris::player::OrgMprisMediaPlayer2Player for () {
    type Err = dbus::tree::MethodErr;

    fn next(&self) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return get_spotify().next_track(None).map_err(|err| MethodErr::failed(&err.to_string()));
    }

    fn previous(&self) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return get_spotify().previous_track(None).map_err(|err| MethodErr::failed(&err.to_string()));
    }

    fn pause(&self) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return get_spotify().pause_playback(None).map_err(|err| MethodErr::failed(&err.to_string()));
    }

    fn play_pause(&self) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        match get_spotify().current_playing(None) {
            Ok(result) => {
                match result {
                    Some(playback) => {
                        if playback.is_playing {
                            self.pause()
                        } else {
                            self.play()
                        }
                    }
                    None => {
                        Err(MethodErr::failed(&"Playback is empty"))
                    }
                }
            }
            Err(error) => {
                Err(MethodErr::failed(&error.to_string()))
            }
        }
    }

    fn stop(&self) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return self.pause();
    }

    fn play(&self) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return get_spotify().start_playback(None, None, None, None).map_err(|err| MethodErr::failed(&err.to_string()));
    }

    fn seek(&self, offset: i64) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        let offset_millis = offset / 1000;
        return match get_spotify().current_playing(None) {
            Ok(result) => {
                match result {
                    Some(playback) => {
                        match playback.progress_ms {
                            Some(current_loc) => {
                                match playback.item {
                                    Some(song) => {

                                        let new_loc = (current_loc as i64) + offset_millis;
                                        let new_loc_u: u32 = if new_loc < 0 {
                                            0
                                        } else if new_loc > (std::u32::MAX as i64) {
                                            std::u32::MAX
                                        } else {
                                            new_loc as u32
                                        };

                                        if new_loc > song.duration_ms as i64 {
                                            // Spotify autoplays next track, where, according to MPRIS, when song is paused or stopped it stays that way
                                            match get_spotify().next_track(None) {
                                                Ok(_) => {
                                                    if !playback.is_playing {
                                                        thread::sleep(Duration::from_millis(500));
                                                        get_spotify().pause_playback(None).map_err(|err| MethodErr::failed(&err.to_string()))
                                                    } else {
                                                        Ok(())
                                                    }
                                                }
                                                Err(err) => {
                                                    Err(MethodErr::failed(&err.to_string()))
                                                }
                                            }
                                        } else {
                                            get_spotify().seek_track(new_loc_u, None).map_err(|err| MethodErr::failed(&err.to_string()))
                                        }
                                    }

                                    None => {
                                        Err(MethodErr::failed(&"Song is not available (propably private mode)"))
                                    }
                                }
                            }
                            None => {
                                Err(MethodErr::failed(&"Progress is empty (propably private mode)"))
                            }
                        }
                    }
                    None => {
                        Err(MethodErr::failed(&"Playback is empty"))
                    }
                }
            }
            Err(error) => {
                Err(MethodErr::failed(&error.to_string()))
            }
        }
    }

    fn set_position(&self, track_id: dbus::Path, position: i64) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {

        match get_spotify().current_playing(None) {
            Ok(result) => {
                match result {
                    Some(playback) => {
                        match playback.item {
                            Some(song) => {
                                let position_ms = position / 1000;
                                if position_ms >= 0 && position_ms <= (song.duration_ms as i64) && ("/org/mpris/MediaPlayer2/Track/".to_string() + &song.id == track_id.to_string()) {
                                    get_spotify()
                                        .seek_track(position_ms as u32, None)
                                        .map_err(|err| MethodErr::failed(&err.to_string()))

                                } else {
                                    //Invalid song - perfectly normal situation according to MPRIS
                                    Ok(())
                                }

                            }

                            None => {
                                Err(MethodErr::failed(&"Song is not available (propably private mode)"))
                            }
                        }
                    }
                    None => {
                        Err(MethodErr::failed(&"Playback is empty"))
                    }
                }
            }
            Err(error) => {
                Err(MethodErr::failed(&error.to_string()))
            }
        }

    }

    fn open_uri(&self, uri: &str) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        match get_spotify().track(uri) {
            Ok(track) => {
                let album_uri = track.album.uri;
                get_spotify().start_playback(None, Some(album_uri), None, for_uri(uri.to_string())).map_err(|err| MethodErr::failed(&err.to_string()))
            }
            Err(error) => {
                Err(MethodErr::failed(&error.to_string()))
            }
        }
    }

    fn get_playback_status(&self) -> Result<String, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        match get_spotify().current_playback(None) {
            Ok(playback_op) => {
                match playback_op {
                    Some(playback) => {
                        Ok((if playback.is_playing { "Playing" } else { "Paused" }).to_string())
                    }
                    None => {
                        Err(MethodErr::failed(&"Playback is not available"))
                    }
                }
            }
            Err(error) => {
                Err(MethodErr::failed(&error.to_string()))
            }
        }
    }

    fn get_loop_status(&self) -> Result<String, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        match get_spotify().current_playback(None) {
            Ok(playback_op) => {
                match playback_op {
                    Some(playback) => {
                        Ok(match playback.repeat_state {
                            RepeatState::Off => "None",
                            RepeatState::Track => "Track",
                            RepeatState::Context => "Playlist",
                        }.to_string())
                    }
                    None => {
                        Err(MethodErr::failed(&"Playback is not available"))
                    }
                }
            }
            Err(error) => {
                Err(MethodErr::failed(&error.to_string()))
            }
        }
    }

    fn set_loop_status(&self, value: String) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {

        let status_option = match value.as_ref() {
            "None" => Some(RepeatState::Off),
            "Track" => Some(RepeatState::Track),
            "Playlist" => Some(RepeatState::Context),
            _ => None
        };

        match status_option {
            Some(status) => {
                get_spotify().repeat(status, None).map_err(|err| MethodErr::failed(&err.to_string()))
            }
            None => {
                Err(MethodErr::failed(&"Invalid loop status value"))
            }
        }

    }

    fn get_rate(&self) -> Result<f64, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return Ok(1.0)
    }

    fn set_rate(&self, _value: f64) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return Ok(())
    }

    fn get_shuffle(&self) -> Result<bool, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        match get_spotify().current_playback(None) {
            Ok(result) => {
                match result {
                    Some(playback) => {
                        Ok(playback.shuffle_state)
                    }
                    None => {
                        Err(MethodErr::failed(&"Playback is empty"))
                    }
                }
            }
            Err(error) => {
                Err(MethodErr::failed(&error.to_string()))
            }
        }
    }

    fn set_shuffle(&self, value: bool) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return get_spotify().shuffle(value, None).map_err(|err| MethodErr::failed(&err.to_string()));
    }

    fn get_metadata(&self) -> Result<HashMap<String, Variant<Box<RefArg>>>, Self::Err> {
        match get_spotify().current_playing(None) {
            Ok(result) => {
                match result {
                    Some(playback) => {
                        match playback.item {
                            Some(song) => {
                                let name = song.name.clone();
                                let mut map = HashMap::new();
                                map.insert("mpris:trackid".to_string(), Variant(Box::new("/org/mpris/MediaPlayer2/Track/".to_string() + &song.id) as Box<RefArg>));
                                map.insert("xesam:title".to_string(), Variant(Box::new(name) as Box<RefArg>));
                                Ok(map)
                            }

                            None => {
                                Err(MethodErr::failed(&"Song is not available (propably private mode)"))
                            }
                        }
                    }
                    None => {
                        Err(MethodErr::failed(&"Playback is empty"))
                    }
                }
            }
            Err(error) => {
                Err(MethodErr::failed(&error.to_string()))
            }
        }

    }

    fn get_volume(&self) -> Result<f64, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        match get_spotify().current_playback(None) {
            Ok(result) => {
                match result {
                    Some(playback) => {
                        Ok(playback.device.volume_percent as f64)
                    }
                    None => {
                        Err(MethodErr::failed(&"Playback is empty"))
                    }
                }
            }
            Err(error) => {
                Err(MethodErr::failed(&error.to_string()))
            }
        }
    }

    fn set_volume(&self, value: f64) -> Result<(), <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        let volume: u8 = if value < 0.0 {
            0
        } else if value > 100.0 {
            100
        } else {
            value as u8
        };

        get_spotify().volume(volume, None).map_err(|err| MethodErr::failed(&err.to_string()))

    }

    fn get_position(&self) -> Result<i64, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        match get_spotify().current_playback(None) {
            Ok(result) => {
                match result {
                    Some(playback) => {
                        match playback.progress_ms {
                            Some(progress_ms) => {
                                return Ok(progress_ms as i64 * 1000)
                            }
                            None => {
                                Err(MethodErr::failed(&"Progress is not available"))
                            }
                        }
                    }
                    None => {
                        Err(MethodErr::failed(&"Playback is empty"))
                    }
                }
            }
            Err(error) => {
                Err(MethodErr::failed(&error.to_string()))
            }
        }
    }

    fn get_minimum_rate(&self) -> Result<f64, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return Ok(1.0)
    }

    fn get_maximum_rate(&self) -> Result<f64, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return Ok(1.0)
    }

    fn get_can_go_next(&self) -> Result<bool, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return std::result::Result::Ok(true);
    }

    fn get_can_go_previous(&self) -> Result<bool, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return std::result::Result::Ok(true);
    }

    fn get_can_play(&self) -> Result<bool, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return std::result::Result::Ok(true);
    }

    fn get_can_pause(&self) -> Result<bool, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return std::result::Result::Ok(true);
    }

    fn get_can_seek(&self) -> Result<bool, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return std::result::Result::Ok(true);
    }

    fn get_can_control(&self) -> Result<bool, <Self as mpris::player::OrgMprisMediaPlayer2Player>::Err> {
        return std::result::Result::Ok(true);
    }
}

pub fn get_interface(spotify: Arc<Spotify>, f: &dbus::tree::Factory<dbus::tree::MTFn<()>, ()>) -> dbus::tree::Interface<dbus::tree::MTFn, ()> {
    unsafe {
        SPOTIFY = Some(spotify);
    }

    let i1 = mpris::player::org_mpris_media_player2_player_server(f, (), |minfo| minfo.path.get_data());
    return i1;
}

