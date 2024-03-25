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
}

impl<'a> DumbFragmentizer<'a> {
	pub(crate) fn new(source: &'a Source) -> Box<Self> {
		Box::new(Self {
			// char_indices is the way
			chars: source.0.chars().enumerate().peekable(),
		})
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
					BytePos(self.chars.peek().map(|(pos, _)| *pos).unwrap_or(0) as u32),
				);
				break Some(Fragment { kind, span });
			};
		}
	}
}

impl DumbFragmentizer<'_> {
	/// Eats symbols while predicate returns true or until the end of file is reached.
	pub(super) fn eat_while(&mut self, predicate: impl Fn(char) -> bool) {
		while let Some(_) = self.chars.next_if(|(_, c)| predicate(*c)) {}
	}
}

impl DumbFragmentizer<'_> {
	fn cook_word(&mut self) -> FragmentKind {
		self.eat_while(char::is_alphanumeric);
		FragmentKind::Unknown
	}
}
