use std::{collections::HashMap, sync::Mutex};

use crate::{
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
	processor: FragmentProcessor<'a>,

	// TODO: put the mutex higher?
	cache: Mutex<HashMap<String, bool>>,
}

/// Initialization
impl<'a> Checker<'a> {
	#[must_use]
	pub fn new(dictionary: Dictionary, language: &str, source: &'a Source) -> Self {
		Self {
			source,
			dictionary,
			processor: FragmentProcessor::from_lang(language, source),

			cache: Mutex::default(),
		}
	}

	// TODO: remove this from api
	pub fn replace_src(&mut self, source: &'a Source) {
		self.source = source;
		self.processor =
			FragmentProcessor::from_lang(self.processor.fragmentizer.lang_code(), source);
	}
}

/// Spellchecking
impl<'a> Checker<'a> {
	#[must_use]
	pub fn check(&self) -> Vec<WordDiagnostic> {
		let fragments = self.processor.process();
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
		if self.lookup(&word) {
			None
		} else {
			Some(WordDiagnostic { word, span })
		}
	}

	fn lookup(&self, word: &str) -> bool {
		let mut cache = self.cache.lock().unwrap();

		if let Some(lookup) = cache.get(word) {
			*lookup
		} else {
			let lookup = self.dictionary.lookup(word).unwrap();
			cache.insert(word.to_owned(), lookup);
			lookup
		}
	}
}
