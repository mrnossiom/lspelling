use crate::span::{BytePos, Source, Span};
use ropey::iter::Chars;
use std::iter::{Enumerate, Peekable};

pub(crate) trait Fragmentizer: Iterator<Item = Fragment> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Fragment {
	pub(crate) kind: FragmentKind,
	pub(crate) span: Span,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FragmentKind {
	// An ident
	// Inner string contains a composed? word with a code case (snake, upper, camel, etc.)
	Ident,

	// Could be either a string litteral or a comment
	// The inner string contains text with spaces
	Sentence,

	// String that should be alone with no parsing semantics
	Unknown,
}

// TODO: make a context-aware parser with tree-sitter, change checking mode in function of context
// TODO: make a nom parser to compare speed
#[derive(Debug)]
pub(crate) struct DumbFragmentizer<'a> {
	chars: Peekable<Enumerate<Chars<'a>>>,
	max_chars: usize,
}

impl<'a> DumbFragmentizer<'a> {
	pub(crate) fn new(source: &'a Source) -> Self {
		Self {
			// char_indices is the way
			chars: source.0.chars().enumerate().peekable(),
			max_chars: source.0.len_bytes(),
		}
	}

	pub(crate) fn boxed(self) -> Box<Self> {
		Box::new(self)
	}
}

impl<'a> Fragmentizer for DumbFragmentizer<'a> {}

impl Iterator for DumbFragmentizer<'_> {
	type Item = Fragment;
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let (start, char_) = self.chars.next()?;

			let kind = match char_ {
				c if c.is_alphanumeric() => Some(self.cook_word()),
				_ => {
					// Ignore special characters
					self.eat_while(|c| !char::is_alphanumeric(c));
					None
				}
			};

			if let Some(kind) = kind {
				// TODO: ugly
				let span = Span::from_bounds(
					BytePos(start as u32),
					BytePos(self.chars.peek().map_or(self.max_chars, |(pos, _)| *pos) as u32),
				);
				break Some(Fragment { kind, span });
			};
		}
	}
}

impl DumbFragmentizer<'_> {
	/// Eats symbols while predicate returns true or until the end of file is reached.
	pub(super) fn eat_while(&mut self, predicate: impl Fn(char) -> bool) {
		while self.chars.next_if(|(_, c)| predicate(*c)).is_some() {}
	}
}

impl DumbFragmentizer<'_> {
	fn cook_word(&mut self) -> FragmentKind {
		self.eat_while(char::is_alphanumeric);
		FragmentKind::Unknown
	}
}

pub(crate) struct SplitFragmentizer<'a, F: Fragmentizer> {
	frag: F,
	buffer: Vec<Fragment>,
	source: &'a Source,
}

impl<'a, F: Fragmentizer> SplitFragmentizer<'a, F> {
	pub(crate) fn new(source: &'a Source, frag: F) -> Self {
		Self {
			frag,
			buffer: Vec::new(),
			source,
		}
	}

	pub(crate) fn boxed(self) -> Box<Self> {
		Box::new(self)
	}

	/// Heck is MIT licenced, let me steal code alone
	fn split_generic_casing(&self, fragment: &Fragment) -> Vec<Fragment> {
		let source = self.source.str_from(fragment.span).as_str().unwrap();

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
						parts_of_fragment.push(Fragment {
							kind: fragment.kind.clone(),
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
						parts_of_fragment.push(Fragment {
							kind: fragment.kind.clone(),
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
					parts_of_fragment.push(Fragment {
						kind: fragment.kind.clone(),
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

impl<F: Fragmentizer> Fragmentizer for SplitFragmentizer<'_, F> {}

impl<F: Fragmentizer> Iterator for SplitFragmentizer<'_, F> {
	type Item = Fragment;
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if let Some(item) = self.buffer.pop() {
				return Some(item);
			} else if let Some(frag) = self.frag.next() {
				match frag.kind {
					FragmentKind::Sentence => return Some(frag),
					// Unknown is parsed as indent
					FragmentKind::Ident | FragmentKind::Unknown => {
						let mut frags = self.split_generic_casing(&frag);
						self.buffer.append(&mut frags);
					}
				}
			} else {
				return None;
			}
		}
	}
}
