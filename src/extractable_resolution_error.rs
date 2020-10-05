use core::fmt::{Display, Error as fmtError, Formatter};
use rhizome_crate::error::Error as rhizomeError;
use std::{borrow::Cow, error::Error as stdError};

#[derive(Debug)]
pub struct ExtractableResolutionError {
	pub component: &'static str,
	pub dependency: &'static str,
	pub source: rhizomeError,
}
impl stdError for ExtractableResolutionError {
	fn source(&self) -> Option<&(dyn stdError + 'static)> {
		Some(&self.source)
	}
}

impl Display for ExtractableResolutionError {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmtError> {
		write!(
			fmt,
			"Missing runtime dependency for component:
{}
-> {}{}",
			self.component,
			self.dependency,
			{
				// Not ideal.
				match &self.source {
					rhizomeError::NoDefault => Cow::from(" (No default provision.)"),
					rhizomeError::NoTagMatched => {
						Cow::from(" (Could not find matching tag to provide at.)")
					}
					source @ rhizomeError::Other(_) => Cow::from(format!(
						"
-> {:#}",
						source,
					)),
				}
			}
		)?;
		Ok(())
	}
}
