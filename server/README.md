# shotty-server

This is the Sinatra app powering <https://shotty.herokuapp.com/>. It provides
the OAuth flow for Dropbox.

## Overview

The server provides 3 routes:

`GET /`: A basic landing page
`GET /authorize`: Redirects user to Dropbox to authenticate `shotty`.
`GET /callback`: Dropbox redirects the user here after they finish or cancel
the authentication process.

The authorization flow is as follows:

1. The user visits `/authorize`.
2. This application redirects them to Dropbox.
3. Dropbox asks the user to accept or deny `shotty` access to their account.
4. If the user accepts access, they are redirected to this application at
   `/callback?code=CODE`. If the user denies access (or any other error
   occurs), they are redirected to `/callback?error=ERROR`
5. If the user successfully authorized `shotty`, a shell command is displayed
   to write a JSON file with their OAuth token. Otherwise an error is
   displayed.

## Environment Variables

The following environment variables must be set for the application to run:

`DROPBOX_APP_KEY`: This is your public App key from Dropbox
`DROPBOX_APP_SECRET`: This is your private App secret from Dropbox
`SHOTTY_CALLBACK_URL`: This is the URL to this application's `/callback` route

In development, these can be set in a `.env` file. In production they should
be set via shell profile (or Heroku config).

## Deploying to Heroku

Since the server application is in a subdirectory of the main `shotty`
project, `git subtree` us used push just the subdirectory.

`bin/deploy` provides this functionality. Run it from any branch and it will
be force pushed to master on Heroku.

## Development

1. Fork it (https://github.com/itspriddle/shotty/fork)
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a new Pull Request

## License

Released under the MIT License. See `LICENSE`.
