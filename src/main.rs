use std::{env::var, path::Path};

use crate::download::DownloadClient;
use args::Args;
use clap::Parser;
use dotenv::dotenv;
use download::DownloadProgress;
use error::DownOnSpotError;
use futures::{pin_mut, Stream, StreamExt};
use librespot::{
	core::{cache::Cache, config::SessionConfig, session::Session},
	discovery::Credentials,
};
use parse::DownloadableAudio;
use simple_logger::SimpleLogger;

mod args;
mod audio_format;
mod convert;
mod download;
mod error;
mod parse;

#[tokio::main]
async fn main() {
	if let Err(error) = run().await {
		log::error!("{}", error);
	}
}

async fn run() -> Result<(), DownOnSpotError> {
	setup_logging()?;
	setup_env()?;

	let args = Args::parse();

	// Librespot session.
	let session = &get_session().await?;

	// Initialize client to download tracks.
	let download_client = DownloadClient::new(session);

	let downloadable_audio = DownloadableAudio::from_id_or_url(session, &args.input).await?;
	let download = download_client
		.download_audio(
			&downloadable_audio,
			&args.strategy,
			&args.output_directory,
			args.mp3,
		)
		.await;

	print_progress(download).await
}

async fn print_progress(
	download: impl Stream<Item = Result<DownloadProgress, DownOnSpotError>>,
) -> Result<(), DownOnSpotError> {
	pin_mut!(download);

	while let Some(progress) = download.next().await {
		match progress? {
			download::DownloadProgress::Started => {
				log::info!("Started download");
			}
			download::DownloadProgress::Finished => {
				log::info!("Finished download");
			}
			download::DownloadProgress::Progress { current, total } => {
				log::info!(
					"Download progress: {:.2}%",
					(current as f64 / total as f64) * 100.0
				);
			}
		}
	}

	Ok(())
}

async fn get_session() -> Result<Session, DownOnSpotError> {
	let config = SessionConfig::default();
	let credentials_cache = Path::new("credentials_cache");
	let cache = Cache::new(credentials_cache.into(), None, None, None).unwrap();
	let (session, _) = Session::connect(
		config,
		Credentials::with_password(
			var("SPOTIFY_USERNAME").expect("SPOTIFY_USERNAME must be set."),
			var("SPOTIFY_PASSWORD").expect("SPOTIFY_PASSWORD must be set."),
		),
		cache.into(),
		true,
	)
	.await?;

	log::info!("Connected to Spotify");

	Ok(session)
}

fn setup_logging() -> Result<(), DownOnSpotError> {
	SimpleLogger::new()
		.with_level(log::LevelFilter::Off)
		.with_module_level("down_on_spot", log::LevelFilter::Debug)
		.init()
		.map_err(|e| DownOnSpotError::Error(e.to_string()))
}

fn setup_env() -> Result<(), DownOnSpotError> {
	dotenv().map_err(|e| DownOnSpotError::Error(e.to_string()))?;
	Ok(())
}
