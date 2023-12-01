use std::io::{Error, ErrorKind, Read, Seek};

use lame::Lame;
use lewton::inside_ogg::OggStreamReader;
use librespot::metadata::FileFormat;

use crate::error::DownOnSpotError;

pub struct AudioConverter<T: Read + Seek> {
	decoder: OggStreamReader<T>,
	encoder: Lame,
}

#[derive(Clone)]
pub enum AudioBitrate {
	Q320,
	Q160,
	Q96,
}

impl AudioBitrate {
	/// Set bitrate for given encoder.
	pub fn set_for_encoder(&self, encoder: &mut Lame) -> Result<(), DownOnSpotError> {
		encoder.set_kilobitrate(self.into())?;
		encoder.set_quality(self.into())?;

		Ok(())
	}
}

// TODO: Fix this.
impl<T: Read + Seek> AudioConverter<T> {
	pub fn new(inner: T, bitrate: AudioBitrate) -> Result<Self, DownOnSpotError> {
		let decoder = OggStreamReader::new(inner)?;
		let mut encoder = Lame::new()
			.ok_or_else(|| DownOnSpotError::EncoderError("Failed to create encoder".to_owned()))?;

		bitrate.set_for_encoder(&mut encoder)?;
		encoder.set_sample_rate(decoder.ident_hdr.audio_sample_rate)?;
		encoder.init_params()?;

		Ok(Self { decoder, encoder })
	}
}

impl<T: Read + Seek> AudioConverter<T> {
	/// Read data from decoder and encode it with encoder.
	fn read_encoded(&mut self, buf: &mut [u8]) -> Result<usize, DownOnSpotError> {
		let Some(data) = self.decoder.read_dec_packet()? else {
			return Ok(0);
		};

		let pcm_left = &data[0];
		let pcm_right = &data[1];

		// Empty packets don't have to be encoded, continue reading.
		if pcm_left.is_empty() {
			return self.read_encoded(buf);
		}

		let size = self.encoder.encode(pcm_left, pcm_right, buf)?;

		if size == 0 {
			return self.read_encoded(buf);
		}

		Ok(size)
	}
}

impl<T: Read + Seek> Read for AudioConverter<T> {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
		self.read_encoded(buf)
			.map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
	}
}

impl From<FileFormat> for AudioBitrate {
	fn from(val: FileFormat) -> Self {
		match val {
			FileFormat::OGG_VORBIS_320 => AudioBitrate::Q320,
			FileFormat::OGG_VORBIS_160 => AudioBitrate::Q160,
			FileFormat::OGG_VORBIS_96 => AudioBitrate::Q96,
			_ => AudioBitrate::Q320,
		}
	}
}

impl From<&AudioBitrate> for i32 {
	fn from(val: &AudioBitrate) -> Self {
		match val {
			AudioBitrate::Q320 => 320,
			AudioBitrate::Q160 => 160,
			AudioBitrate::Q96 => 96,
		}
	}
}

impl From<&AudioBitrate> for u8 {
	fn from(val: &AudioBitrate) -> Self {
		match val {
			AudioBitrate::Q320 => 0,
			AudioBitrate::Q160 => 2,
			AudioBitrate::Q96 => 5,
		}
	}
}
