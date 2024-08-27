use std::fmt;

use crate::span::Span;

mod dumb;
mod rust;

pub(crate) use self::dumb::DumbFragmentizer;
pub(crate) use self::rust::RustFragmentizer;

pub(crate) trait Fragmentizer<'a>: Send + Sync + fmt::Debug {
	fn fragmentize(&self) -> Vec<Fragment>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Fragment {
	pub(crate) kind: FragmentKind,
	pub(crate) span: Span,
}

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
