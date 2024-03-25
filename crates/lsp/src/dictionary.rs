use hunspell_rs::{CheckResult, Hunspell};

pub(crate) struct Dictionary {
	inner: Hunspell,
}

impl Dictionary {
	pub(crate) fn new() -> Self {
		Self {
			inner: Hunspell::new(
				concat!(env!("HUNSPELL_DICT"), ".aff"),
				concat!(env!("HUNSPELL_DICT"), ".dic"),
			),
		}
	}

	pub(crate) fn is_correct(&self, word: &str) -> bool {
		match self.inner.check(word) {
			CheckResult::FoundInDictionary => true,
			CheckResult::MissingInDictionary => false,
		}
	}

	pub(crate) fn suggest(&self, word: &str) -> Vec<String> {
		self.inner.suggest(word)
	}
}
