use crate::{
	fragmentizer::{DumbFragmentizer, FragmentKind, Fragmentizer, RustFragmentizer},
	span::{BytePos, Source, Span},
};

// TODO: rename, make doc, refers to a processed fragment ready to be checked
#[derive(Debug)]
pub(crate) struct Token {
	pub(crate) kind: TokenKind,
	pub(crate) span: Span,
}

// TODO: we only support hunspell which is a single word checker
// we could extend to more complex strategies to lint sentences as a whole
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TokenKind {
	Ident,
	Unknown,
}

// ----

#[derive(Debug)]
pub(crate) struct FragmentProcessor<'a> {
	pub(crate) fragmentizer: Box<dyn Fragmentizer<'a> + 'a>,
	source: &'a Source,
}

impl<'a> FragmentProcessor<'a> {
	pub(crate) fn new(fragmentizer: Box<dyn Fragmentizer<'a> + 'a>, source: &'a Source) -> Self {
		Self {
			fragmentizer,
			source,
		}
	}

	// TODO: rename
	pub(crate) fn from_lang(language: &str, source: &'a Source) -> Self {
		let fragmentizer: Box<dyn Fragmentizer<'a> + 'a> = match language {
			"rust" => RustFragmentizer::new(source).boxed(),
			"plaintext" => DumbFragmentizer::new(source).boxed(),
			_ => todo!(),
		};

		Self::new(fragmentizer, source)
	}

	pub(crate) fn process(&self) -> Vec<Token> {
		let mut tokens = Vec::new();
		for fragment in self.fragmentizer.fragmentize() {
			match fragment.kind {
				// TODO: somehow split sentence
				FragmentKind::Sentence => {
					let mut toks = self.split_sentence(fragment.span);
					tokens.append(&mut toks);
				}

				// Unknown is parsed as indent
				FragmentKind::Ident | FragmentKind::Unknown => {
					let mut toks = self.split_generic_casing(fragment.span);
					tokens.append(&mut toks);
				}
			}
		}
		tokens
	}

	// Heck is MIT licenced, let me steal code alone
	fn split_generic_casing(&self, span: Span) -> Vec<Token> {
		#[derive(Clone, Copy, PartialEq)]
		enum WordMode {
			/// There have been no lowercase or uppercase characters in the current
			/// word.
			Boundary,
			/// The previous cased character in the current word is lowercase.
			Lowercase,
			/// The previous cased character in the current word is uppercase.
			Uppercase,
		}

		let source = self.source.str_from(span).to_string();

		let mut first_word = true;
		let mut parts_of_fragment = Vec::new();

		for word in source.split(|c: char| !c.is_alphanumeric()) {
			let mut char_indices = word.char_indices().peekable();
			let mut init = 0;
			let mut mode = WordMode::Boundary;

			while let Some((i, c)) = char_indices.next() {
				if let Some(&(next_i, next)) = char_indices.peek() {
					// The mode including the current character, assuming the
					// current character does not result in a word boundary.
					let next_mode = if c.is_lowercase() {
						WordMode::Lowercase
					} else if c.is_uppercase() {
						WordMode::Uppercase
					} else {
						mode
					};

					// Word boundary after if current is not uppercase and next
					// is uppercase
					if next_mode == WordMode::Lowercase && next.is_uppercase() {
						parts_of_fragment.push(Token {
							// TODO
							kind: TokenKind::Unknown,
							span: span.relative(BytePos(init as u32), BytePos(next_i as u32)),
						});
						first_word = false;
						init = next_i;
						mode = WordMode::Boundary;

					// Otherwise if current and previous are uppercase and next
					// is lowercase, word boundary before
					} else if mode == WordMode::Uppercase && c.is_uppercase() && next.is_lowercase()
					{
						if first_word {
							first_word = false;
						}
						parts_of_fragment.push(Token {
							// TODO
							kind: TokenKind::Unknown,
							span: span.relative(BytePos(init as u32), BytePos(i as u32)),
						});
						init = i;
						mode = WordMode::Boundary;

					// Otherwise no word boundary, just update the mode
					} else {
						mode = next_mode;
					}
				} else {
					// Collect trailing characters as a word
					if first_word {
						first_word = false;
					}
					parts_of_fragment.push(Token {
						// TODO
						kind: TokenKind::Unknown,
						span: Span::new(span.low + BytePos(init as u32), span.high),
					});
					break;
				}
			}
		}

		parts_of_fragment
	}

	fn split_sentence(&self, span: Span) -> Vec<Token> {
		let mut tokens = Vec::new();
		let source = self.source.str_from(span).to_string();

		let mut start_idx = 0;
		for (index, char_) in source.char_indices() {
			// TODO: this `matches` is fragile, decide if we keep some chars or ignore other (e.g. `,`)
			if (char_.is_alphabetic() || matches!(char_, '-' | '\'')) && index != source.len() - 1 {
				continue;
			}

			if start_idx != index {
				tokens.push(Token {
					kind: TokenKind::Ident,
					span: span.relative(BytePos(start_idx as u32), BytePos(index as u32)),
				});
			}

			start_idx = index + 1;
		}

		tokens
	}
}
