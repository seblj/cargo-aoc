use chrono::prelude::*;
pub enum AocError
{
    TokenError,
    ReqwestError(reqwest::Error),
    DownloadError(String),
    #[cfg(feature = "submit")]
    SanitizeHtml,
    ParseStdout,
    InvalidRunDay,
    #[cfg(feature = "submit")]
    InvalidSubmitDay,
    InvalidYear,
    InvalidMonth,
    ParseIntError,
    ArgMatches,
    Utf8Error,
    StdIoErr(std::io::Error),
    ArgError(String),
    #[cfg(feature = "tally")]
    BuildError(String),
    #[cfg(feature = "tally")]
    RunError(String),
}

macro_rules! impl_from_helper {
    ($from:ty, $to: expr) => {
        impl From<$from> for AocError
        {
            fn from(e: $from) -> Self
            {
                $to(e)
            }
        }
    };
}

macro_rules! impl_print {
    ($($from: ty),*) => {$(
        impl $from for AocError
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
            {
                match self
                {
                    AocError::TokenError => write!(f, "Could not find AOC_TOKEN to download input or submit"),
                    AocError::ReqwestError(e) => write!(f, "reqwest error: {}", e),
                    #[cfg(feature = "submit")]
                    AocError::SanitizeHtml => write!(f, "Error on sanitizing answer"),
                    AocError::ParseStdout => write!(f, "Error on getting answer from task"),
                    AocError::InvalidRunDay => write!(f, "Day must be between 1 and 25"),
                    #[cfg(feature = "submit")]
                    AocError::InvalidSubmitDay => write!(f, "Can only submit 1 or 2"),
                    AocError::InvalidYear => write!(f, "Year must be between 2015 ..= current year"),
                    AocError::InvalidMonth => write!(f, "I's {}, but its not yet december!", Utc::now().year()),
                    AocError::ParseIntError => write!(f, "Error parsing to number"),
                    AocError::ArgMatches => write!(f, "Error on getting argument"),
                    AocError::StdIoErr(e) => write!(f, "{}", e),
                    AocError::DownloadError(e) => write!(f, "{}", e),
                    AocError::Utf8Error => write!(f, "Error on parsing to utf-8"),
                    AocError::ArgError(e) => write!(f, "{}", e),
                    #[cfg(feature = "tally")]
                    AocError::BuildError(e) => write!(f, "{}", e),
                    #[cfg(feature = "tally")]
                    AocError::RunError(e) => write!(f, "{}", e),
                }
            }
        }
    )*}
}

impl_from_helper!(dotenv::Error, |_| AocError::TokenError);
impl_from_helper!(std::num::ParseIntError, |_| AocError::ParseIntError);
impl_from_helper!(std::str::Utf8Error, |_| AocError::Utf8Error);
impl_from_helper!(std::io::Error, |e| Self::StdIoErr(e));
impl_from_helper!(reqwest::Error, |e| Self::ReqwestError(e));

impl_print!(std::fmt::Display, std::fmt::Debug);
