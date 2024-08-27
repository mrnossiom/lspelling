use std::path::Path;

use crate::{
	fragmentizer::{DumbFragmentizer, Fragmentizer, RustFragmentizer},
	processor::{FragmentProcessor, TokenKind},
	span::{Source, Span},
};
use ruspell::Dictionary;

#[derive(Debug)]
pub struct WordDiagnostic {
	pub word: String,
	pub span: Span,
}

#[derive(Debug)]
pub struct Checker<'a> {
	pub(crate) source: &'a Source,
	// TODO: dedup with lsp, wa for no send bound
	dictionary: Dictionary,
	fragmentizer: FragmentProcessor<'a>,
}

impl<'a> Checker<'a> {
	#[must_use]
	pub fn new(language: &str, source: &'a Source) -> Self {
		let fragmentizer: Box<dyn Fragmentizer<'a> + 'a> = match language {
			"rust" |
			// TODO
			// "rust" => RustFragmentizer::new(source).boxed(),

			"plaintext" => DumbFragmentizer::new(source).boxed(),
			_ => todo!(),
		};

		Self {
			source,
			dictionary: Dictionary::from_pair(Path::new(env!("HUNSPELL_DICT"))).unwrap(),
			fragmentizer: FragmentProcessor::new(source, fragmentizer),
		}
	}

	pub fn check(&self) -> Vec<WordDiagnostic> {
		let fragments = self.fragmentizer.process();
		let mut diags = Vec::new();

		for token in fragments {
			match token.kind {
				TokenKind::Ident | TokenKind::Unknown => {
					let source = self.source.str_from(token.span).as_str().unwrap();
					match self.diagnostic(source.to_owned(), token.span) {
						Some(diag) => diags.push(diag),
						None => continue,
					};
				}
			};
		}

		diags
	}

	fn diagnostic(&self, word: String, span: Span) -> Option<WordDiagnostic> {
		if self.dictionary.lookup(&word).unwrap() {
			None
		} else {
			Some(WordDiagnostic { word, span })
		}
	}
}
