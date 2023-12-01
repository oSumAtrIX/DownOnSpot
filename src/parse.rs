use futures::future;
use librespot::{
	core::{session::Session, spotify_id::SpotifyId},
	metadata::{Album, Episode, Metadata, Playlist, Show, Track},
};
use url::Url;

use crate::error::DownOnSpotError;

pub enum DownloadableAudio {
	Track(Track),
	Album(Vec<Track>),
	Playlist(Vec<Track>),
	Show(Vec<Episode>),
	Episode(Episode),
}

impl DownloadableAudio {
	pub async fn from_id_or_url(
		session: &Session,
		input: &str,
	) -> Result<DownloadableAudio, DownOnSpotError> {
		// Try parsing as URL.
		let item = DownloadableAudio::from_url(session, input).await;
		if item.is_ok() {
			return item;
		}

		// Try parsing as Spotify ID.
		let invalid_id = || DownOnSpotError::Invalid("Invalid Spotify URL or ID".to_string());

		let mut splits = input
			.strip_prefix("spotify:")
			.ok_or_else(invalid_id)?
			.split(':');

		let item_type = splits.next().ok_or_else(invalid_id)?;
		let spotify_id = splits.next().ok_or_else(invalid_id)?;

		DownloadableAudio::from_id(session, item_type, spotify_id).await
	}

	/// Get Spotify item from URL.
	async fn from_url(
		session: &Session,
		input: &str,
	) -> Result<DownloadableAudio, DownOnSpotError> {
		let url = Url::parse(input)?;

		let invalid_uri_error = || DownOnSpotError::Invalid("Invalid Spotify URL".to_owned());
		let domain = url.domain().ok_or_else(invalid_uri_error)?;

		if !domain.to_lowercase().ends_with("spotify.com") {
			return Err(invalid_uri_error());
		}

		let mut segments = url.path_segments().ok_or_else(invalid_uri_error)?;
		let item_type = segments
			.next()
			.ok_or_else(invalid_uri_error)?
			.replace('/', "");
		let spotify_id = segments.next_back().ok_or_else(invalid_uri_error)?;

		DownloadableAudio::from_id(session, &item_type, spotify_id).await
	}

	/// Get Spotify item from ID.
	async fn from_id(
		session: &Session,
		item_type: &str,
		spotify_id: &str,
	) -> Result<DownloadableAudio, DownOnSpotError> {
		let spotify_id = SpotifyId::from_uri(&format!("spotify:{}:{}", item_type, spotify_id))?;

		let spotify_item = match item_type {
			"track" => DownloadableAudio::Track(Track::get(session, spotify_id).await?),
			"album" => {
				let album_tracks = Album::get(session, spotify_id).await?.tracks;

				let futures = album_tracks
					.into_iter()
					.map(|track| Track::get(session, track))
					.collect::<Vec<_>>();

				let tracks = future::join_all(futures)
					.await
					.into_iter()
					.filter_map(|track| track.ok())
					.collect::<Vec<_>>();

				DownloadableAudio::Album(tracks)
			}
			"playlist" => {
				let playlist = Playlist::get(session, spotify_id).await?;

				let futures = playlist
					.tracks
					.iter()
					.map(|track| Track::get(session, *track))
					.collect::<Vec<_>>();

				let tracks = future::join_all(futures)
					.await
					.into_iter()
					.filter_map(|track| track.ok())
					.collect::<Vec<_>>();

				DownloadableAudio::Playlist(tracks)
			}
			"show" => {
				let show = Show::get(session, spotify_id).await?;

				let futures = show
					.episodes
					.into_iter()
					.map(|episode| Episode::get(session, episode))
					.collect::<Vec<_>>();

				let episodes = future::join_all(futures)
					.await
					.into_iter()
					.filter_map(|episode| episode.ok())
					.collect::<Vec<_>>();

				DownloadableAudio::Show(episodes)
			}
			"episode" => DownloadableAudio::Episode(Episode::get(session, spotify_id).await?),
			_ => return Err(DownOnSpotError::InvalidOrUnsupportedId),
		};

		Ok(spotify_item)
	}
}
