use crate::{
	fragmentizer::{Fragment, FragmentKind, Fragmentizer},
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
	frag: Box<dyn Fragmentizer<'a> + 'a>,
	source: &'a Source,
}

impl<'a> FragmentProcessor<'a> {
	pub(crate) fn new(source: &'a Source, frag: Box<dyn Fragmentizer<'a> + 'a>) -> Self {
		Self { frag, source }
	}

	pub(crate) fn process(&self) -> Vec<Token> {
		let mut tokens = Vec::new();
		for fragment in self.frag.fragmentize() {
			match fragment.kind {
				// TODO: somehow split sentence
				FragmentKind::Sentence => todo!(),

				// Unknown is parsed as indent
				FragmentKind::Ident | FragmentKind::Unknown => {
					let mut toks = self.split_generic_casing(&fragment);
					tokens.append(&mut toks);
				}
			}
		}
		tokens
	}

	// Heck is MIT licenced, let me steal code alone
	fn split_generic_casing(&self, fragment: &Fragment) -> Vec<Token> {
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

		let source = self.source.str_from(fragment.span).as_str().unwrap();

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
							span: Span::from_bounds(
								fragment.span.low + BytePos(init as u32),
								fragment.span.low + BytePos(next_i as u32),
							),
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
							span: Span::from_bounds(
								fragment.span.low + BytePos(init as u32),
								fragment.span.low + BytePos(i as u32),
							),
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
						span: Span::from_bounds(
							fragment.span.low + BytePos(init as u32),
							fragment.span.high,
						),
					});
					break;
				}
			}
		}

		parts_of_fragment
	}
}
