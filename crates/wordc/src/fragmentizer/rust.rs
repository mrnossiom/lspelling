use std::fmt;
use tree_sitter::{Parser, Tree};

use super::{Fragment, Fragmentizer};
use crate::span::Source;

pub(crate) struct RustFragmentizer {
	parser: Parser,
	tree: Tree,
}

impl fmt::Debug for RustFragmentizer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("RustFragmentizer").finish_non_exhaustive()
	}
}

impl<'a> RustFragmentizer {
	pub(crate) fn new(source: &Source) -> Self {
		let mut parser = Parser::new();
		parser.set_language(&tree_sitter_rust::language()).unwrap();

		let tree = parser
			.parse(source.0.slice(..).as_str().unwrap(), None)
			.unwrap();

		Self { parser, tree }
	}

	pub(crate) fn boxed(self) -> Box<dyn Fragmentizer<'a>> {
		Box::new(self)
	}
}

impl<'a> Fragmentizer<'a> for RustFragmentizer {
	fn fragmentize(&self) -> Vec<Fragment> {
		todo!()
	}
}
