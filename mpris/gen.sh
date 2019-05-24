#!/bin/bash

path_to_executable=$(which dbus-codegen-rust)
if [ ! -x "$path_to_executable" ]; then
    cargo install dbus-codegen
fi

set -e

MPRIS_SPEC="mpris-spec/spec"
SOURCE_LOCATION="../src/mpris"

dbus-codegen-rust < ${MPRIS_SPEC}/org.mpris.MediaPlayer2.Player.xml > ${SOURCE_LOCATION}/player.rs
dbus-codegen-rust < ${MPRIS_SPEC}/org.mpris.MediaPlayer2.Playlists.xml > ${SOURCE_LOCATION}/playlists.rs
dbus-codegen-rust < ${MPRIS_SPEC}/org.mpris.MediaPlayer2.TrackList.xml > ${SOURCE_LOCATION}/tracklist.rs
dbus-codegen-rust < ${MPRIS_SPEC}/org.mpris.MediaPlayer2.xml > ${SOURCE_LOCATION}/mpris.rs
