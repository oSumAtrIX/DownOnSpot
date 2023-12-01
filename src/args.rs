use clap::{arg, command, Parser};

use crate::audio_format::DownloadOrderStrategy;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	#[arg(
		short,
		long,
		help = "Track / Album / Playlist / Artist / Podcast / Episode / Show / User URL, ID or search term",
	)]
	pub input: String,

	#[clap(value_enum)]
	#[arg(short, long, default_value = "quality", help = "Download strategy")]
	pub strategy: DownloadOrderStrategy,

	#[arg(short, long, default_value = "false", help = "Convert to MP3")]
	pub mp3: bool,

	#[arg(
		short,
		long,
		default_value = "4",
		help = "Number of concurrent downloads"
	)]
	pub concurrent_downloads: usize,

	#[arg(
		short,
		long,
		default_value = "%artist% - %title%",
		help = "Template for file name"
	)]
	pub template: String,

	#[arg(short, long, default_value = "downloads", help = "Output directory")]
	pub output_directory: String,

	#[arg(short, long, default_value = " - ", help = "Separator between artists")]
	pub artist_separator: String,

	#[arg(long, default_value = "true", help = "Skip download if file exists")]
	pub skip_exists: bool,
}
