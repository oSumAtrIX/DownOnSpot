<div align="center">

# DownOnSpot

A Spotify downloader written in Rust

<img src="assets/icon.svg" alt="drawing" width="500"/>

<br>

[![Build project](https://github.com/oSumAtrIX/DownOnSpot/actions/workflows/rust.yml/badge.svg)](https://github.com/oSumAtrIX/DownOnSpot/actions/workflows/rust.yml)
[![GitHub license](https://img.shields.io/github/license/oSumAtrIX/DownOnSpot)](https://github.com/oSumAtrIX/DownOnSpot/blob/main/LICENSE)
[![GitHub issues](https://img.shields.io/github/issues/oSumAtrIX/DownOnSpot)](https://github.com/oSumAtrIX/DownOnSpot/issues)
[![GitHub forks](https://img.shields.io/github/forks/oSumAtrIX/DownOnSpot)](https://github.com/oSumAtrIX/DownOnSpot/network)
[![GitHub stars](https://img.shields.io/github/stars/oSumAtrIX/DownOnSpot)](https://github.com/oSumAtrIX/DownOnSpot/stargazers)
[![Stability: Experimental](https://masterminds.github.io/stability/experimental.svg)](https://masterminds.github.io/stability/experimental.html)

</div>

## Disclaimer

```text
DownOnSpot was not developed for piracy.
It is meant to be used in compliance with DMCA, Section 1201, for educational, private and fair use.
I am not responsible in any way for the usage of the source code.
```

## Features

- Works with free Spotify accounts
- Download 96, 160kbit/s audio with a free, 256 and 320 kbit/s audio with a premium account from Spotify, directly
- Multi-threaded
- Search for tracks
- Download tracks, playlists, albums and artists
- Convert to mp3
- Metadata tagging
- Simple usage over CLI

## Building

1. Clone the repository using git and change to the local repository directory:

   ```bash
   git clone https://github.com/oSumAtrIX/DownOnSpot.git
   cd DownOnSpot
   ```

2. Build

   ```bash
   cargo build --release
   ```

> [!WARNING]
> You need [this private SSH key](assets/free_librespot_private_key) to clone a dependency of DownOnSpot in order to use it with a free Spotify account.
> Follow [this answer by DopeGhoti on stackexchange.com](https://unix.stackexchange.com/a/494485) to set up SSH with the private key.
> A sample `~/.ssh/config` file could look like this:
>
> ```text
> Host github.com
>   IdentityFile ~/.ssh/free_librespot_private_key
> ```

> [!NOTE]
> If you do not want to use `free-librespot` (i.e. if you are using a paid Spotify account), then remove the git dependency of `free-librespot`.
> For that, replace `git = "ssh://git@github.com/oSumAtrIX/free-librespot.git"` with `librespot = "0.4.2"` inside the `Cargo.toml` file.

> [!WARNING]
> If you get a linker error, you might need to download the [standard libmp3lame](https://www.rarewares.org/mp3-lame-libraries.php#libmp3lame) library.
> On Mac OS, it should be enough to just run `brew install lame`, provided you have [Homebrew](https://brew.sh/) installed.

## Usage/ Examples

Running DownOnSpot once will create the default configuration file in the same directory as your shell.

```bash
$ down_on_spot.exe
Settings could not be loaded, because of the following error: IO: NotFound No such file or directory. (os error 2)...
..but default settings have been created successfully. Edit them and run the program again.

$ down_on_spot.exe
Usage:
down_on_spot.exe (search_term | track_url | album_url | playlist_url | artist_url)
```
> [!NOTE]
> On Mac OS, the `settings.json` file is created globally for the logged in user and is located in `~/.config/down_on_spot/settings.json`.  
> On Windows, the `settings.json` file is located at `C:\Users\<user>\AppData\Roaming\down_on_spot\settings.json`.

Apart from your Spotify username and password, you will need to login in to the Spotify developer dashboard and [create a new private application](https://developer.spotify.com/dashboard/applications). Fill in the `client_id` and `client_secret` in your `settings.json` from your newly created app.
All the other settings should be self-explanatory, conversion from Ogg to MP3 is disabled by default.

> [!NOTE]
> If you're on a free Spotify account, remember to set down the quality to 160kbit/s at maximum ("Q320" -> "Q160"). Otherwise, you'll get an "Audio Key Error" since the free account can't exceed 160kbit/s.

### Template variables

Following variables are available for `path` and `filename_template` in the `settings.json`:

- %0disc%
- %0track%
- %album%
- %albumArtist%
- %albumArtists%
- %artist%
- %disc%
- %id%
- %title%
- %track%

## Additional scripts

- [Userscript to download titles from YouTube](https://gist.github.com/oSumAtrIX/6abf46e2ea25d32f4e6608c3c3cf837e)

## Known issues

- Mp3 downloads slow due to libmp3lame
- Downloads fail sometimes due to `channel error`

## Authors

- [@oSumAtrIX](https://osumatrix.me/#github)
- [@exttex](https://git.freezer.life/exttex)
- [@breuerfelix](https://github.com/breuerfelix)
- [@thatpix3l](https://github.com/thatpix3l)

## License

[GPLv3](https://choosealicense.com/licenses/gpl-3.0/)
