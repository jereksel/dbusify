## Dbusify

MPRIS2 client for Spotify Web API. Implements most of the MPRIS specification including `org.mpris.MediaPlayer2.Playlists` and `org.mpris.MediaPlayer2.TrackList`

### Prerequirements

- Spotify Premium account
- Spotify application (https://developer.spotify.com/dashboard/applications) - must have `http://localhost:8888/callback`  as one of callbacks

### Usage

1. Install dbusify `cargo install --git https://github.com/jereksel/dbusify.git --tag v0.1.0`
2. Run `dbusify` with following environmental variables:
    - "CLIENT_ID" - client id of Spotify application
    - "CLIENT_SECRET" - client secret of Spotify application
3. Web browser should open - login to your account and linked your account to application
4. Done - registered player name is `dbusify`

### Limitations

- Getting tracks doesn't work when listening to "Your Music" [upstream bug report](https://github.com/spotify/web-api/issues/1022)

