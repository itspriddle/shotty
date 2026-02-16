# Shotty

A CLI tool to upload screenshots (and other files) to cloud storage and get direct URLs. Supports Dropbox and Droplr as backends.

## Install

```sh
cargo install --path .
```

## Setup

### Dropbox

#### 1. Create a Dropbox App

1. Go to <https://www.dropbox.com/developers/apps>
2. Click **Create app**
3. Choose **Scoped access** and **Full Dropbox**
4. Under **Permissions**, enable `files.content.write` and `sharing.write`
5. Copy the **App key** from the Settings tab

#### 2. Configure Shotty

```sh
shotty config set backend dropbox
shotty config set dropbox.app_key YOUR_APP_KEY
```

#### 3. Authenticate

```sh
shotty auth
```

This opens a browser for Dropbox OAuth authorization using PKCE.

### Droplr

#### 1. Configure Shotty

```sh
shotty config set backend droplr
```

#### 2. Authenticate

```sh
shotty auth
```

This prompts for your Droplr email and password. Credentials are stored securely in the system keyring.

## Usage

Upload a file:

```sh
shotty upload screenshot.png
```

Upload from stdin:

```sh
cat image.png | shotty upload --name image.png
```

Upload and copy URL to clipboard:

```sh
shotty upload screenshot.png --copy
```

## Configuration

Show current config:

```sh
shotty config show
```

Print config file path:

```sh
shotty config path
```

The config file location is platform-specific. Use `shotty config path` to check.

## Development

```sh
cargo build
cargo test
cargo fmt
cargo clippy --all-targets -- -D warnings
```
