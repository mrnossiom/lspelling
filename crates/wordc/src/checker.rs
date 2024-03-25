use crate::{
	fragmentizer::{DumbFragmentizer, FragmentKind, Fragmentizer},
	span::{Source, Span},
};
use hunspell_rs::Hunspell;

pub struct WordDiagnostic {
	pub word: String,
	pub span: Span,
}

pub struct Checker<'a> {
	pub(crate) source: &'a Source,
	// TODO: dedup with lsp, wa for no send bound
	dictionary: Hunspell,
	fragmentizer: Box<dyn Fragmentizer + 'a>,
}

impl<'a> Checker<'a> {
	pub fn new(source: &'a Source) -> Self {
		Self {
			source,
			dictionary: Hunspell::new(
				concat!(env!("HUNSPELL_DICT"), ".aff"),
				concat!(env!("HUNSPELL_DICT"), ".dic"),
			),
			fragmentizer: DumbFragmentizer::new(&source),
		}
	}
}

impl<'a> Iterator for Checker<'a> {
	type Item = WordDiagnostic;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let Some(fragment) = self.fragmentizer.next() else {
				return None;
			};

			match fragment.kind {
				FragmentKind::Ident | FragmentKind::Sentence | FragmentKind::Unknown => {
					let word = self.source.str_from(fragment.span).as_str().unwrap();

					match self.dictionary.check(word) {
						hunspell_rs::CheckResult::FoundInDictionary => continue,
						hunspell_rs::CheckResult::MissingInDictionary => {
							return Some(WordDiagnostic {
								word: word.to_string(),
								span: fragment.span,
							});
						}
					};
				}
			};
		}
	}
}
