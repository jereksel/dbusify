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
use dbus::arg::RefArg;
use std::sync::atomic::AtomicBool;
use dbus::Connection;
use dbus::BusType;
use dbus::ConnPath;
use dbusify::mpris::player::OrgMprisMediaPlayer2Player;
use std::time::Duration;
use rspotify::spotify::model::offset::for_position;
use rspotify::spotify::senum::RepeatState::Context;
use rspotify::spotify::senum::RepeatState::Track;
use rspotify::spotify::senum::RepeatState::Off;
use rspotify::spotify::senum::RepeatState;
use integration_tests_utils::run_test_type;
use dbusify::AccountType;

#[test]
fn play() {

    run_test(|spotify, connection| {
        let album = "spotify:album:7yXJHuBXqHuNXKvHwWBirS".to_string();
        spotify.start_playback(None, Some(album), None, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.pause_playback(None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(!spotify.current_playback(None).unwrap().unwrap().is_playing);
        connection.play().unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(spotify.current_playback(None).unwrap().unwrap().is_playing);
    })

}

#[test]
fn pause() {

    run_test(|spotify, connection| {
        let album = "spotify:album:7yXJHuBXqHuNXKvHwWBirS".to_string();
        spotify.start_playback(None, Some(album), None, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(spotify.current_playback(None).unwrap().unwrap().is_playing);
        connection.pause().unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(!spotify.current_playback(None).unwrap().unwrap().is_playing);
    })

}

#[test]
fn play_pause() {

    run_test(|spotify, connection| {
        let album = "spotify:album:7yXJHuBXqHuNXKvHwWBirS".to_string();
        spotify.start_playback(None, Some(album), None, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(spotify.current_playback(None).unwrap().unwrap().is_playing);
        connection.play_pause().unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(!spotify.current_playback(None).unwrap().unwrap().is_playing);
        connection.play_pause().unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(spotify.current_playback(None).unwrap().unwrap().is_playing);
    })


}

#[test]
fn set_shuffle() {

    run_test(|spotify, connection| {
        let album = "spotify:album:7yXJHuBXqHuNXKvHwWBirS".to_string();
        spotify.start_playback(None, Some(album), None, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.shuffle(false, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(!spotify.current_playback(None).unwrap().unwrap().shuffle_state);
        connection.set_shuffle(true).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(spotify.current_playback(None).unwrap().unwrap().shuffle_state);
        connection.set_shuffle(false).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(!spotify.current_playback(None).unwrap().unwrap().shuffle_state);
    })

}

#[test]
fn get_shuffle() {

    run_test(|spotify, connection| {
        let album = "spotify:album:7yXJHuBXqHuNXKvHwWBirS".to_string();
        spotify.start_playback(None, Some(album), None, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.shuffle(false, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(!connection.get_shuffle().unwrap());
        spotify.shuffle(true, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(connection.get_shuffle().unwrap());
    });

}

#[test]
fn seek() {

    run_test(|spotify, connection| {
        let album = "spotify:album:2kyTLcEZe6nc1s6ve0zW9P".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.pause_playback(None).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.shuffle(false, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(!spotify.current_playback(None).unwrap().unwrap().is_playing);
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, "spotify:track:2DDBGX5qlbBRZsAqAWfILT");
        let current_loc = spotify.current_playback(None).unwrap().unwrap().progress_ms.unwrap();
        connection.seek(10_000_000).unwrap();
        thread::sleep(Duration::from_millis(500));
        let current_loc2 = spotify.current_playback(None).unwrap().unwrap().progress_ms.unwrap();
        let difference = current_loc2 - current_loc;
        println!("Difference: {:?}", difference);
        assert!(difference < 11_000);
        assert!(difference > 9_000);

        connection.seek(-100_000_000).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, "spotify:track:2DDBGX5qlbBRZsAqAWfILT");
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().progress_ms.unwrap(), 0);

        connection.seek(100_000_000_000).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, "spotify:track:5VL3c4UjMFR8BcPqoFVTNc");
        assert!(!spotify.current_playback(None).unwrap().unwrap().is_playing);
        assert!(spotify.current_playback(None).unwrap().unwrap().progress_ms.unwrap() < 1_000);

        spotify.start_playback(None, None, None, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(spotify.current_playback(None).unwrap().unwrap().is_playing);

        connection.seek(100_000_000_000).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, "spotify:track:4R4HHwbrMb4Fttzrfccmh7");
        assert!(spotify.current_playback(None).unwrap().unwrap().is_playing);
        assert!(spotify.current_playback(None).unwrap().unwrap().progress_ms.unwrap() < 5_000);
    });

}

#[test]
fn open_uri() {

    run_test(|spotify, connection| {
        let song_uri = "spotify:track:3cfOd4CMv2snFaKAnMdnvK";
        connection.open_uri(song_uri).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, song_uri);
        assert!(spotify.current_playback(None).unwrap().unwrap().is_playing);

        let song_uri_2 = "spotify:track:3y9l6k1sTwXgJ3L80IfrS9";
        connection.open_uri(song_uri_2).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, song_uri_2);
        assert!(spotify.current_playback(None).unwrap().unwrap().is_playing);
    });

}

#[test]
fn set_volume() {

    run_test(|spotify, connection| {
        let song_uri = "spotify:track:3cfOd4CMv2snFaKAnMdnvK";
        connection.open_uri(song_uri).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.pause_playback(None).unwrap();
        thread::sleep(Duration::from_millis(500));

        connection.set_volume(50.0).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().device.volume_percent, 50);

        connection.set_volume(-10.0).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().device.volume_percent, 0);

        connection.set_volume(110.0).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().device.volume_percent, 100);
    });

}

#[test]
fn get_volume() {

    run_test(|spotify, connection| {
        let song_uri = "spotify:track:3cfOd4CMv2snFaKAnMdnvK";
        connection.open_uri(song_uri).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.pause_playback(None).unwrap();
        thread::sleep(Duration::from_millis(500));

        spotify.volume(70, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(connection.get_volume().unwrap(), 70.0);
    })

}

#[test]
fn get_position() {

    run_test(|spotify, connection| {
        let song_uri = "spotify:track:3cfOd4CMv2snFaKAnMdnvK";
        connection.open_uri(song_uri).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.pause_playback(None).unwrap();

        let api_position = spotify.current_playback(None).unwrap().unwrap().progress_ms.unwrap() as i64;
        let dbusify_position = connection.get_position().unwrap() / 1000;

        let delta = api_position - dbusify_position;

        assert!(delta.abs() < 1000);

    })

}

#[test]
fn next() {

    run_test(|spotify, connection| {

        let album = "spotify:album:2kyTLcEZe6nc1s6ve0zW9P".to_string();
        let song1 = "spotify:track:2DDBGX5qlbBRZsAqAWfILT".to_string();
        let song2 = "spotify:track:5VL3c4UjMFR8BcPqoFVTNc".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.shuffle(false, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, song1.clone());
        connection.next().unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, song2.clone());

    })

}

#[test]
fn previous() {

    run_test(|spotify, connection| {

        let album = "spotify:album:2kyTLcEZe6nc1s6ve0zW9P".to_string();
        let song1 = "spotify:track:2DDBGX5qlbBRZsAqAWfILT".to_string();
        let song2 = "spotify:track:5VL3c4UjMFR8BcPqoFVTNc".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.shuffle(false, None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, song1.clone());
        spotify.next_track(None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, song2.clone());
        connection.previous().unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().item.unwrap().uri, song1.clone());
    })

}

#[test]
fn get_playback_status() {

    run_test(|spotify, connection| {

        let song_uri = "spotify:track:3cfOd4CMv2snFaKAnMdnvK";
        connection.open_uri(song_uri).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(spotify.current_playback(None).unwrap().unwrap().is_playing);
        assert_eq!(connection.get_playback_status().unwrap(), "Playing");
        spotify.pause_playback(None).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(!spotify.current_playback(None).unwrap().unwrap().is_playing);
        assert_eq!(connection.get_playback_status().unwrap(), "Paused");

    })

}

//FLAKY
#[test]
#[ignore]
fn get_loop_status() {

    run_test(|spotify, connection| {

        let album = "spotify:album:2kyTLcEZe6nc1s6ve0zW9P".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(1000));

        spotify.repeat(Context, None).unwrap();
        thread::sleep(Duration::from_millis(1500));
        assert_eq!(connection.get_loop_status().unwrap(), "Playlist");

        spotify.repeat(Track, None).unwrap();
        thread::sleep(Duration::from_millis(1500));
        assert_eq!(connection.get_loop_status().unwrap(), "Track");

        spotify.repeat(Off, None).unwrap();
        thread::sleep(Duration::from_millis(1500));
        assert_eq!(connection.get_loop_status().unwrap(), "None");

    })


}

#[test]
fn set_loop_status() {

    run_test(|spotify, connection| {

        let album = "spotify:album:2kyTLcEZe6nc1s6ve0zW9P".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(1000));

        connection.set_loop_status("Playlist".to_string()).unwrap();
        thread::sleep(Duration::from_millis(1500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().repeat_state.as_str(), RepeatState::Context.as_str());

        connection.set_loop_status("Track".to_string()).unwrap();
        thread::sleep(Duration::from_millis(1500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().repeat_state.as_str(), RepeatState::Track.as_str());

        connection.set_loop_status("None".to_string()).unwrap();
        thread::sleep(Duration::from_millis(1500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().repeat_state.as_str(), RepeatState::Off.as_str());

    })


}

#[test]
fn get_metadata() {

    run_test(|spotify, connection| {
        let album = "spotify:album:2kyTLcEZe6nc1s6ve0zW9P".to_string();
        let song1 = "2DDBGX5qlbBRZsAqAWfILT".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(500));

        let id = connection.get_metadata().unwrap().get("mpris:trackid").unwrap().as_str().unwrap().to_string();

        assert_eq!("/org/mpris/MediaPlayer2/Track/".to_string() + &song1, id);

    })

}

#[test]
fn set_position() {

    run_test(|spotify, connection| {
        let album = "spotify:album:2kyTLcEZe6nc1s6ve0zW9P".to_string();
        let song1 = "2DDBGX5qlbBRZsAqAWfILT".to_string();
        spotify.start_playback(None, Some(album), None, for_position(0)).unwrap();
        thread::sleep(Duration::from_millis(500));
        spotify.pause_playback(None).unwrap();
        thread::sleep(Duration::from_millis(500));

        let one_minute = Duration::from_secs(60);

        connection.set_position(dbus::Path::from("/org/mpris/MediaPlayer2/Track/".to_string() + &song1), one_minute.as_micros() as i64).unwrap();
        thread::sleep(Duration::from_millis(500));

        let current_progress = spotify.current_playback(None).unwrap().unwrap().progress_ms.unwrap() as i64;

        let delta = (current_progress - one_minute.as_millis() as i64).abs();

        assert!(delta < 500);

        connection.set_position(dbus::Path::from("/org/mpris/MediaPlayer2/Track/asd".to_string()), one_minute.as_micros() as i64).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().progress_ms.unwrap() as i64, current_progress);

        connection.set_position(dbus::Path::from("/org/mpris/MediaPlayer2/Track/".to_string() + &song1), -10_000_000).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().progress_ms.unwrap() as i64, current_progress);

        connection.set_position(dbus::Path::from("/org/mpris/MediaPlayer2/Track/".to_string() + &song1), Duration::from_secs(60*5).as_micros() as i64).unwrap();
        thread::sleep(Duration::from_millis(500));
        assert_eq!(spotify.current_playback(None).unwrap().unwrap().progress_ms.unwrap() as i64, current_progress);

    })

}


fn run_test<T>(test: T) -> ()
    where T: FnOnce(Spotify, ConnPath<&Connection>) -> () + panic::UnwindSafe
{
    run_test_type(AccountType::Main, test);
}