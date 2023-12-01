use async_stream::try_stream;

use futures::StreamExt;
use futures::{stream::FuturesUnordered, Stream};
use librespot::metadata::Artist;
use librespot::{
	audio::{AudioDecrypt, AudioFile},
	core::{session::Session, spotify_id::FileId},
	metadata::{FileFormat, Metadata, Track},
};

use std::io::ErrorKind;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::pin::Pin;
use std::{fs::File, future};

use crate::parse::DownloadableAudio;
use crate::{audio_format::is_ogg, error::DownOnSpotError};
use crate::{audio_format::DownloadOrderStrategy, convert::AudioConverter};

pub struct DownloadClient {
	session: Session,
	download_progress_queue:
		Vec<Pin<Box<dyn Stream<Item = Result<DownloadProgress, DownOnSpotError>>>>>,
}

pub struct DecryptedAudioFile {
	pub is_ogg: bool,
	pub audio_decrypt: AudioDecrypt<AudioFile>,
	pub size: usize,
	pub format: FileFormat,
}

pub enum DownloadProgress {
	Started,
	Progress { current: usize, total: usize },
	Finished,
}

pub const SPOTIFY_OGG_HEADER_END: u64 = 0xA7;

impl DownloadClient {
	pub fn new(session: &Session) -> Self {
		Self {
			session: session.clone(),
			download_progress_queue: vec![],
		}
	}

	/// Remove finished downloads from the download progress queue.
	pub async fn remove_finished(&mut self) {
		let mut new_download_progress_queue = Vec::new();

		// Filter out every download that is finished.
		while let Some(mut download) = self.download_progress_queue.pop() {
			if let Some(Ok(progress)) = download.next().await {
				if let DownloadProgress::Finished = progress {
					continue;
				}

				new_download_progress_queue.push(download);
			}
		}

		self.download_progress_queue = new_download_progress_queue;
	}

	/// Get file id for given track and strategy.
	async fn file_id(
		&self,
		strategy: &DownloadOrderStrategy,
		track: Track,
	) -> Result<(FileId, FileFormat), DownOnSpotError> {
		let formats = strategy.formats();

		formats
			.iter() // Ordered by format.
			.find_map(|format| {
				let file_id = track.files.get(format)?;

				Some((*file_id, *format))
			})
			.ok_or(DownOnSpotError::Unavailable)
	}

	async fn decrypt_stream(
		&self,
		strategy: &DownloadOrderStrategy,
		track: Track,
	) -> Result<DecryptedAudioFile, DownOnSpotError> {
		let id = track.id;
		let (file_id, format) = self.file_id(strategy, track).await?;

		let audio_file = AudioFile::open(&self.session, file_id, 1024 * 1024 * 1024, true).await?;
		let size = audio_file.get_stream_loader_controller().len();
		let key = self.session.audio_key().request(id, file_id).await?;

		// Decrypt audio file.
		let mut audio_decrypt = AudioDecrypt::new(key, audio_file);

		// OGG files have a header that needs to be skipped.
		let is_ogg = is_ogg(format);
		let offset = if is_ogg {
			audio_decrypt.seek(SeekFrom::Start(SPOTIFY_OGG_HEADER_END))?; // The header is irrelevant.

			SPOTIFY_OGG_HEADER_END
		} else {
			0
		} as usize;

		Ok(DecryptedAudioFile {
			is_ogg,
			audio_decrypt,
			size: size - offset,
			format,
		})
	}

	/// Get reader for given track and strategy.
	/// If mp3 is true, convert OGG to MP3.
	async fn reader(
		&self,
		track: &Track,
		strategy: &DownloadOrderStrategy,
		mp3: bool,
	) -> Result<(usize, Box<dyn Read>), DownOnSpotError> {
		let track = self
			.available_track(track)
			.await
			.ok_or(DownOnSpotError::Unavailable)?;

		let decrypted = self.decrypt_stream(strategy, track).await?;

		let reader: Box<dyn Read> = if decrypted.is_ogg && mp3 {
			let converter = AudioConverter::new(decrypted.audio_decrypt, decrypted.format.into())?;

			Box::new(converter)
		} else {
			Box::new(decrypted.audio_decrypt)
		};

		Ok((decrypted.size, reader))
	}

	pub async fn download_audio<'a>(
		&'a self,
		downloadable_audio: &'a DownloadableAudio,
		strategy: &'a DownloadOrderStrategy,
		output_directory: &'a str,
		mp3: bool,
	) -> impl Stream<Item = Result<DownloadProgress, DownOnSpotError>> + 'a {
		match downloadable_audio {
			DownloadableAudio::Track(track) => {
				self.download(track, strategy, output_directory, mp3)
			}
			DownloadableAudio::Album(_) | DownloadableAudio::Playlist(_) => todo!(),
			DownloadableAudio::Show(_) => todo!(), // List of episodes.
			DownloadableAudio::Episode(_episode) => todo!(), // Annoyingly, episodes are not tracks.
		}
	}

	fn download<'a>(
		&'a self,
		track: &'a Track,
		strategy: &'a DownloadOrderStrategy,
		output_directory: &'a str,
		mp3: bool,
	) -> impl Stream<Item = Result<DownloadProgress, DownOnSpotError>> + 'a {
		try_stream! {
			yield DownloadProgress::Started;

			// TODO: Move this to somewhere else.
			let track_name = &track.name;
			let track_artist = Artist::get(&self.session, *track.artists.first().unwrap()).await?.name;

			// Actual downloader logic.

			let (size, mut reader) = self.reader(track, strategy, mp3).await?;

			let mut file: Vec<u8> = vec![];

			let mut current = 0;
			loop {
				let mut buffer = [0; 1024 * 64];

				match reader.read(&mut buffer) {
					Ok(0) => {
						yield DownloadProgress::Finished;
					break;
					}
					Ok(bytes_read) => {
						file.extend_from_slice(&buffer[..bytes_read]);

						current += bytes_read;
						yield DownloadProgress::Progress{current, total:size};
					}
					Err(e) => {
						if e.kind() == ErrorKind::Interrupted {
							continue;
						}

						return;
					}
				}
			}

			let mut path = PathBuf::from(output_directory);

			// TODO: Move this to somewhere else.
			let file_name = if mp3 {
				format!("{} - {}.mp3", track_artist, track_name)
			} else {
				format!("{} - {}.ogg", track_artist, track_name)
			};

			path.push(file_name);

			// Write audio file.
			File::create(path)?.write_all(&file)?;
		}
	}

	/// Find available track.
	/// If not found, fallback to alternative tracks.
	async fn available_track(&self, track: &Track) -> Option<Track> {
		if !track.files.is_empty() {
			return Some(track.to_owned());
		}

		

		track
			.alternatives
			.iter()
			.map(|alt_id| Track::get(&self.session, *alt_id))
			.collect::<FuturesUnordered<_>>()
			.filter_map(|x| future::ready(x.ok()))
			.filter(|x| future::ready(x.available))
			.next()
			.await
	}
}
