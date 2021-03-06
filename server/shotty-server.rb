ENV["RACK_ENV"] ||= "development"

require "json"
require "net/http"
require "sinatra"
require "rack/protection"
require "uri"

if ENV["RACK_ENV"] == "development"
  require "dotenv"
  Dotenv.load
end

# Dropbox App key.
#
# Returns a String.
DROPBOX_APP_KEY = ENV.fetch("DROPBOX_APP_KEY")

# Dropbox App secret.
#
# Returns a String.
DROPBOX_APP_SECRET = ENV.fetch("DROPBOX_APP_SECRET")

# URL Dropbox redirects to after an OAuth attempt.
#
# Returns a String.
SHOTTY_CALLBACK_URL = ENV.fetch("SHOTTY_CALLBACK_URL")

# Session lifetime in seconds (15 mins).
#
# Returns an Integer.
SHOTTY_SESSION_TTL = 900

# Session secret. Generate with `SecureRandom.hex(64)`
#
# Returns a String.
SHOTTY_SESSION_SECRET = ENV.fetch("SHOTTY_SESSION_SECRET")

# CSRF token.
#
# Returns a String.
def csrf_token
  env["rack.session"].fetch(:csrf)
end

# Generates a URL to dropbox.com to ask a user to authorize Shotty.
#
# Returns a String.
def build_auth_url
  URI.parse("https://www.dropbox.com/oauth2/authorize").tap do |uri|
    uri.query = URI.encode_www_form(
      client_id:     DROPBOX_APP_KEY,
      redirect_uri:  SHOTTY_CALLBACK_URL,
      response_type: :code,
      state:         csrf_token,
    )
  end
end

# Gets an OAuth token for the user using their authorization code.
#
# code - Dropbox Authorization code, sent back to this server by Dropbox
#
# Returns a String or nil.
def get_token(code)
  uri = URI.parse("https://api.dropboxapi.com/oauth2/token")

  payload = URI.encode_www_form(
    client_id:     DROPBOX_APP_KEY,
    client_secret: DROPBOX_APP_SECRET,
    code:          code,
    grant_type:    "authorization_code",
    redirect_uri:  SHOTTY_CALLBACK_URL,
  )

  http         = Net::HTTP.new(uri.host, uri.port)
  http.use_ssl = uri.scheme == "https"

  response = http.post(uri.path, payload, {})

  body = JSON.parse(response.body)

  body["access_token"]
end

# Verifies the `state` param sent to Dropbox matches the one they sent back.
#
# state - String state
#
# Returns a boolean.
def verify_state?(state)
  return false if state.nil? || state.empty?
  Rack::Utils.secure_compare(csrf_token, state)
end

use Rack::Session::Cookie,
  expire_after: SHOTTY_SESSION_TTL,
  secret:       SHOTTY_SESSION_SECRET

use Rack::Protection::AuthenticityToken

error 404 do
  erb :"404", layout: :layout
end

get "/" do
  erb :index, layout: :layout
end

get "/robots.txt" do
  "User-agent: *\nDisallow: /\n"
end

get "/authorize" do
  redirect build_auth_url
end

get "/callback" do
  unless verify_state?(params[:state])
    halt 404
  end

  # TODO: Nicer error
  if error = params[:error]
    halt error
  end

  @token = get_token(params[:code])

  erb :callback, layout: :layout
end
