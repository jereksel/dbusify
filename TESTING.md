## Testing

For testing you need second account (can be free) for destructive operations. 

### Setup

1. Login to your main account
2. Run integration_playlist_get_data_test -> get_active_playlist
3. Authorize application
4. Logout and login to TEST ACCOUNT
5. Run integration_playlist_get_data_test -> get_playlist_count (test should fail)
6. Authorize application - MAKE SURE THIS IS TEST ACCOUNT
7. Comment "#[ignore]" in tests/bootstrap_account.rs
7. Run bootstrap_account -> bootstrap
8. You can now run tests using test.sh
