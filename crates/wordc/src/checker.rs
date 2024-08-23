use std::path::Path;

use crate::{
	fragmentizer::{DumbFragmentizer, FragmentKind, Fragmentizer, SplitFragmentizer},
	span::{Source, Span},
};
use ruspell::Dictionary;

pub struct WordDiagnostic {
	pub word: String,
	pub span: Span,
}

pub struct Checker<'a> {
	pub(crate) source: &'a Source,
	// TODO: dedup with lsp, wa for no send bound
	dictionary: Dictionary,
	fragmentizer: Box<dyn Fragmentizer + 'a>,
}

impl<'a> Checker<'a> {
	#[must_use]
	pub fn new(source: &'a Source) -> Self {
		let fragmentizer = DumbFragmentizer::new(source);
		Self {
			source,
			dictionary: Dictionary::from_pair(Path::new(env!("HUNSPELL_DICT"))).unwrap(),
			fragmentizer: SplitFragmentizer::new(source, fragmentizer).boxed(),
		}
	}

	fn diagnostic(&self, word: String, span: Span) -> Option<WordDiagnostic> {
		if self.dictionary.lookup(&word).unwrap() {
			None
		} else {
			Some(WordDiagnostic { word, span })
		}
	}
}

impl<'a> Iterator for Checker<'a> {
	type Item = WordDiagnostic;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let fragment = self.fragmentizer.next()?;

			match fragment.kind {
				FragmentKind::Sentence => todo!(),
				FragmentKind::Ident | FragmentKind::Unknown => {
					let source = self.source.str_from(fragment.span).as_str().unwrap();
					match self.diagnostic(source.to_owned(), fragment.span) {
						Some(diag) => return Some(diag),
						None => continue,
					};
				}
			};
		}
	}
}
