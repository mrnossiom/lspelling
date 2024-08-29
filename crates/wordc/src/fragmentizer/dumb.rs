use std::iter::{Enumerate, Peekable};

use ropey::iter::Chars;

use super::{Fragment, FragmentKind, Fragmentizer};
use crate::span::{BytePos, Source, Span};

// TODO: make a context-aware parser with tree-sitter, change checking mode in function of context
// TODO: make a nom parser to compare speed
#[derive(Debug)]
pub(crate) struct DumbFragmentizer<'a> {
	source: &'a Source,
}

impl<'a> DumbFragmentizer<'a> {
	pub(crate) const fn new(source: &'a Source) -> Self {
		Self { source }
	}

	pub(crate) fn boxed(self) -> Box<dyn Fragmentizer<'a> + 'a> {
		Box::new(self)
	}
}

impl<'a> Fragmentizer<'a> for DumbFragmentizer<'a> {
	fn lang_code(&self) -> &'static str {
		"plaintext"
	}

	fn fragmentize(&self) -> Vec<Fragment> {
		let mut chars = self.source.0.chars().enumerate().peekable();
		let max_chars = self.source.0.len_chars();

		let mut buffer = Vec::new();
		while let Some((start, char_)) = chars.next() {
			let kind = match char_ {
				c if c.is_alphanumeric() => Some(Self::cook_word(&mut chars)),
				_ => {
					// Ignore special characters
					Self::eat_while(&mut chars, |c| !char::is_alphanumeric(c));
					None
				}
			};

			if let Some(kind) = kind {
				// TODO: ugly
				let span = Span::new(
					BytePos(start as u32),
					BytePos(chars.peek().map_or(max_chars, |(pos, _)| *pos) as u32),
				);
				buffer.push(Fragment { kind, span });
			};
		}
		buffer
	}
}

impl DumbFragmentizer<'_> {
	/// Eats symbols while predicate returns true or until the end of file is reached.
	pub(super) fn eat_while(
		chars: &mut Peekable<Enumerate<Chars>>,
		predicate: impl Fn(char) -> bool,
	) {
		while chars.next_if(|(_, c)| predicate(*c)).is_some() {}
	}
}

impl DumbFragmentizer<'_> {
	fn cook_word(chars: &mut Peekable<Enumerate<Chars>>) -> FragmentKind {
		Self::eat_while(chars, char::is_alphanumeric);
		FragmentKind::Unknown
	}
}
