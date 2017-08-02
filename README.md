# shotty

This script makes it easier to share screenshots and files via Dropbox on OS
X. If you hate the public landing page you get with these Dropbox features,
read on.

## Dropbox already has that!

Dropbox already provides [Shared Links][] and [Shared Screenshots][]. They
point to an HTML landing page that allows the user to download or browse files
(or bugs them to sign up for Dropbox). For some use cases this is fine, but
most of the time I'd rather have a direct link to paste into Slack or GitHub
Issues.

Shared Links _can_ be edited manually to point directly to a file. But I hate
doing things manually so I wrote a thing to save me seconds a week.

## Gimme

`shotty` is written in Ruby. If you are a Rubyist, you may care to know the
script uses OS X's system Ruby with standard library dependencies only. If
you're not a Rubyist, that jargon means you don't need to install anything
special to run `shotty`.

The script can be downloaded directly from GitHub. Run the following commands
in Terminal.app.

Download the script:

```
curl -L https://github.com/itspriddle/shotty/raw/master/bin/shotty > /usr/local/bin/shotty
```

Make it executable:

```
chmod +x /usr/local/bin/shotty
```

Check that it works:

```
shotty -v
```

You should see output like "Shotty v0.0.1"

## Authorization

To use `shotty`, you need a Dropbox OAuth2 key. To generate one, run `shotty
authorize` and follow the instructions in your browser.

Note that `shotty` uses full Dropbox access so it can access all of your
files. The [`shotty-server`][] handles the OAuth flow.

## Usage

You can now use `shotty` to work with your Dropbox account. If you only care
about automatically copying Dropbox URLs when you take a screenshot, skip the
next section.

### The `shotty` script

The `shotty` script contains several subcommands for working with shared
links. See `shotty --help` for a full overview. The most useful are:

* `shotty create-url <file>`: Creates a Dropbox URL for the specifed file. This
  should be used with files that have never had a shared link generated.
  Otherwise it will fail.
* `shotty get-url <file>`: Gets an existing Dropbox shared link for the
  specified file.
* `shotty url <file>`: Gets or creates a Dropbox shared link for the given
  file. This is useful when it is not known ahead of time whether a file
  already has a shared link.

### Automatically copy direct Dropbox URLs for screenshots

Do you want a direct Dropbox URL copied to your clipboard when you create a
screenshot? Me too, here's how.

Since the screenshot sharing feature built into Dropbox doesn't support a
direct link, turn it off. Option+Click on the Dropbox icon in the OS X menubar
and click Preferences. Click the Import tab, then uncheck "Share screenshots
using Dropbox".

A [launchd.plist][] will be used to copy screenshots from `~/Desktop` to
`~/Dropbox/Shotty` and copy a direct shared link to the clipboard.

To enable this behavior:

```
shotty plist > $(shotty plist-file)
launchctl load $(shotty plist-file)
```

To disable:

```
launchctl unload $(shotty plist-file)
rm $(shotty plist-file)
```

See the `screenshot_directory` configuration option below to customize the
directory screenshots are copied to.

**NOTE**: If you used [dropbox-screenshots-plist][], make sure to disable that
first:

```
launchctl unload ~/Library/LaunchAgents/net.nevercraft.dropbox-screenshots.plist
rm ~/Library/LaunchAgents/net.nevercraft.dropbox-screenshots.plist
```

## Configuration

`shotty` configuration is stored in a JSON file at `~/.config/shotty.json`.
The following options are available:

* `token` **REQUIRED**: Dropbox API OAuth2 access token.
* `dropbox_root` _OPTIONAL_: Root Dropbox directory, by default `~/Dropbox`.
  Useful if you have personal and business Dropbox folders on the same
  machine.
* `screenshot_directory` _OPTIONAL_: Directory to store screenshots in, by
  default `~/Dropbox/Shotty`. This must be a subdirectory of `dropbox_root`.

Full example configuration file:

```
{
  "token":                "MY-REDACTED-TOKEN",
  "dropbox_root":         "/Users/priddle/Dropbox (Personal)",
  "screenshot_directory": "/Users/priddle/Dropbox/Shotty"
}
```

## Ugh, I hate you, make this all go away.

Visit [Dropbox Account Security][], and remove "shotty-cli" at the bottom of
the page.

To remove `shotty` from your system:

```
launchctl unload $(shotty plist-file)
rm $(shotty plist-file)
rm $(shotty config-file)
rm /usr/local/bin/shotty
```

## Development

1. Fork it (https://github.com/itspriddle/shotty/fork)
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a new Pull Request

## License

Released under the MIT License. See `LICENSE`.

[Dropbox Account Security]: https://www.dropbox.com/account/security
[Shared Links]: https://www.dropbox.com/help/167
[Shared Screenshots]: https://www.dropbox.com/help/1964
[launchd.plist]: https://developer.apple.com/library/mac/documentation/Darwin/Reference/ManPages/man5/launchd.plist.5.html#//apple_ref/doc/man/5/launchd.plist
[`shotty-server`]: https://github.com/itspriddle/shotty/tree/master/server
[dropbox-screenshots-plist]: https://github.com/itspriddle/dropbox-screenshots-plist
