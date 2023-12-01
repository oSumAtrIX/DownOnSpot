use core::fmt;
use std::io::ErrorKind;

use lame::EncodeError;
use lewton::VorbisError;
use librespot::{
	core::{
		audio_key::AudioKeyError, channel::ChannelError, mercury::MercuryError,
		session::SessionError, spotify_id::SpotifyIdError,
	},
	discovery::Error,
};
use rspotify::{model::IdError, ClientError};
use tokio::task::JoinError;
use url::ParseError;

#[derive(Debug, Clone)]
pub enum DownOnSpotError {
	Error(String),
	Authentication,
	IoError(ErrorKind, String),
	Unavailable,
	InvalidOrUnsupportedId,
	DecoderError(String),
	EncoderError(String),
	Invalid(String),
	DownloaderError,
}

impl fmt::Display for DownOnSpotError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			DownOnSpotError::Error(e) => write!(f, "{}", e),
			DownOnSpotError::Authentication => write!(f, "Authentication error"),
			DownOnSpotError::IoError(kind, e) => write!(f, "IO error: {:?} - {}", kind, e),
			DownOnSpotError::Unavailable => write!(f, "Unavailable"),
			DownOnSpotError::InvalidOrUnsupportedId => write!(f, "Invalid or unsupported ID"),
			DownOnSpotError::DecoderError(e) => write!(f, "Decoder error: {}", e),
			DownOnSpotError::EncoderError(e) => write!(f, "Encoder error: {}", e),
			DownOnSpotError::Invalid(e) => write!(f, "Invalid: {}", e),
			DownOnSpotError::DownloaderError => write!(f, "Error during download"),
		}
	}
}

impl From<IdError> for DownOnSpotError {
	fn from(e: IdError) -> Self {
		Self::Invalid(e.to_string())
	}
}

impl From<ParseError> for DownOnSpotError {
	fn from(e: ParseError) -> Self {
		Self::Invalid(e.to_string())
	}
}

impl From<ClientError> for DownOnSpotError {
	fn from(_e: ClientError) -> Self {
		Self::Authentication
	}
}

impl From<VorbisError> for DownOnSpotError {
	fn from(e: VorbisError) -> Self {
		Self::DecoderError(e.to_string())
	}
}

impl From<EncodeError> for DownOnSpotError {
	fn from(e: EncodeError) -> Self {
		Self::EncoderError(format!("{:?}", e))
	}
}

impl From<lame::Error> for DownOnSpotError {
	fn from(kind: lame::Error) -> Self {
		Self::EncoderError(format!("{:?}", kind))
	}
}

impl From<AudioKeyError> for DownOnSpotError {
	fn from(_e: AudioKeyError) -> Self {
		Self::Error("AudioKey Error".to_owned())
	}
}

impl From<JoinError> for DownOnSpotError {
	fn from(_e: JoinError) -> Self {
		Self::Error("Join Error".to_owned())
	}
}

impl From<SpotifyIdError> for DownOnSpotError {
	fn from(_e: SpotifyIdError) -> Self {
		Self::InvalidOrUnsupportedId
	}
}

impl From<ChannelError> for DownOnSpotError {
	fn from(_e: ChannelError) -> Self {
		Self::Error("Channel Error".to_owned())
	}
}

impl From<std::io::Error> for DownOnSpotError {
	fn from(e: std::io::Error) -> Self {
		Self::IoError(e.kind(), e.to_string())
	}
}

impl From<Error> for DownOnSpotError {
	fn from(e: Error) -> Self {
		Self::Error(e.to_string())
	}
}

impl From<SessionError> for DownOnSpotError {
	fn from(e: SessionError) -> Self {
		match e {
			SessionError::AuthenticationError(_) => Self::Authentication,
			SessionError::IoError(_) => todo!(),
		}
	}
}

impl From<Box<dyn std::error::Error>> for DownOnSpotError {
	fn from(e: Box<dyn std::error::Error>) -> Self {
		Self::Error(e.to_string())
	}
}

impl From<MercuryError> for DownOnSpotError {
	fn from(_: MercuryError) -> Self {
		Self::DownloaderError
	}
}
