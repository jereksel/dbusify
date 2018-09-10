#!/usr/bin/env bash

set -e

export RUST_TEST_THREADS=1

cargo test --test integration_player_test
cargo test --test integration_playlist_get_data_test
cargo test --test integration_playlist_play_test
cargo test --test integration_tracklist_test
