use librespot::metadata::FileFormat;

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum DownloadOrderStrategy {
	MP3,
	OGG,
	QUALITY,
}

impl DownloadOrderStrategy {
	/// Get librespot AudioFormat for this quality.
	pub fn formats(&self) -> Vec<FileFormat> {
		// Abuse order of enum variants.
		match self {
			DownloadOrderStrategy::MP3 => vec![
				FileFormat::MP3_320,
				FileFormat::MP3_256,
				FileFormat::MP3_160,
				FileFormat::MP3_160_ENC,
				FileFormat::MP3_96,
			],
			DownloadOrderStrategy::OGG => vec![
				FileFormat::OGG_VORBIS_320,
				FileFormat::OGG_VORBIS_160,
				FileFormat::OGG_VORBIS_96,
			],
			DownloadOrderStrategy::QUALITY => vec![
				FileFormat::MP3_320,
				FileFormat::OGG_VORBIS_320,
				FileFormat::MP3_256,
				FileFormat::MP3_160,
				FileFormat::MP3_160_ENC,
				FileFormat::OGG_VORBIS_160,
				FileFormat::MP3_96,
				FileFormat::OGG_VORBIS_96,
			],
		}
	}
}

/// Check if format is OGG.
pub fn is_ogg(format: FileFormat) -> bool {
	matches!(
		format,
		FileFormat::OGG_VORBIS_320 | FileFormat::OGG_VORBIS_160 | FileFormat::OGG_VORBIS_96
	)
}
