use crate::{
	fragmentizer::{DumbFragmentizer, FragmentKind, Fragmentizer, RustFragmentizer},
	span::{BytePos, Source, Span},
};

// TODO: rename, make doc, refers to a processed fragment ready to be checked
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
	pub(crate) kind: TokenKind,
	pub(crate) span: Span,
}

impl Token {
	const fn new(kind: TokenKind, span: Span) -> Self {
		Self { kind, span }
	}

	const fn new_word(span: Span) -> Self {
		Self {
			kind: TokenKind::Word,
			span,
		}
	}
}

// TODO: we only support hunspell which is a single word checker
// we could extend to more complex strategies to lint sentences as a whole
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
	Word,
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
			lang => {
				tracing::warn!("language `{lang}` is not listed, defaulting to dumb fragmentizer");
				DumbFragmentizer::new(source).boxed()
			}
		};

		Self::new(fragmentizer, source)
	}

	// TODO: op for keeping sentence as is? useful for other strategies than ruspell
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

	/// Split code idents on casing boundaires to retrieve individual words
	#[must_use]
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

		for (index, word) in str_split_indices(&source, |c: char| !c.is_alphanumeric()) {
			let local_span = span.relative(BytePos::from(index), BytePos::from(index + word.len()));

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
							kind: TokenKind::Word,
							span: local_span.relative(BytePos::from(init), BytePos::from(next_i)),
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
							kind: TokenKind::Word,
							span: local_span.relative(BytePos::from(init), BytePos::from(i)),
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
						kind: TokenKind::Word,
						span: Span::new(local_span.low + BytePos::from(init), local_span.high),
					});
					break;
				}
			}
		}

		parts_of_fragment
	}

	/// Splits content by whitespace and trim individual words from non-alphabetical characters
	#[must_use]
	fn split_sentence(&self, span: Span) -> Vec<Token> {
		let source = self.source.str_from(span).to_string();

		str_split_indices(&source, char::is_whitespace)
			.map(|(index, string)| {
				let offset_before_trim = addr_of(string);
				let trimmed = string.trim_matches(|c: char| !c.is_alphabetic());
				let offset = addr_of(trimmed) - offset_before_trim;

				Token {
					kind: TokenKind::Word,
					span: span.relative(
						BytePos::from(index + offset),
						BytePos::from(index + trimmed.len()),
					),
				}
			})
			.collect()
	}
}

fn addr_of(s: &str) -> usize {
	s.as_ptr() as usize
}

fn str_split_indices(
	slice: &str,
	pattern: impl FnMut(char) -> bool,
) -> impl Iterator<Item = (usize, &str)> {
	slice
		.split(pattern)
		.filter(|slice| !slice.is_empty())
		.map(move |sub| (addr_of(sub) - addr_of(slice), sub))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn split_snake_case_ident() {
		let source = "
			let bye_jello = true;
		";
		let src = Source::new(source);
		let proc = FragmentProcessor::from_lang("rust", &src);

		let ident_span = Span::new(BytePos(8), BytePos(8 + 9));
		assert_eq!(
			proc.split_generic_casing(ident_span),
			[
				// bye
				Token::new_word(ident_span.relative(BytePos(0), BytePos(3))),
				// jello
				Token::new_word(ident_span.relative(BytePos(4), BytePos(9)))
			]
		);
	}
}
