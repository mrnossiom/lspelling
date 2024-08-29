use std::fmt;
use tree_sitter::{Parser, Query, QueryCursor, Tree};

use super::{Fragment, Fragmentizer};
use crate::{
	fragmentizer::FragmentKind,
	span::{BytePos, Source, Span},
};

pub(crate) struct RustFragmentizer<'a> {
	source: &'a Source,

	parser: Parser,
	query: Query,

	tree: Tree,
}

impl fmt::Debug for RustFragmentizer<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("RustFragmentizer").finish_non_exhaustive()
	}
}

pub const SPELLCHECK_QUERY: &str = include_str!("../../queries/rust.scm");

impl<'a> RustFragmentizer<'a> {
	pub(crate) fn new(source: &'a Source) -> Self {
		let grammar = tree_sitter_rust::language();

		let mut parser = Parser::new();
		parser.set_language(&grammar).expect("language is correct");

		let query = Query::new(&grammar, SPELLCHECK_QUERY).expect("spellcheck query is correct");

		let tree = parser
			.parse(source.0.slice(..).as_str().unwrap(), None)
			.unwrap();

		Self {
			source,

			parser,
			query,
			tree,
		}
	}

	pub(crate) fn boxed(self) -> Box<dyn Fragmentizer<'a> + 'a> {
		Box::new(self)
	}
}

impl<'a> Fragmentizer<'a> for RustFragmentizer<'a> {
	fn lang_code(&self) -> &'static str {
		"rust"
	}

	fn fragmentize(&self) -> Vec<Fragment> {
		let mut cursor = QueryCursor::new();
		let matches = cursor.matches(
			&self.query,
			self.tree.root_node(),
			self.source.0.slice(..).as_str().unwrap().as_bytes(),
		);

		matches
			.flat_map(|m| {
				m.captures
					.iter()
					.map(|c| {
						let start = c.node.byte_range().start as u32;
						let end = c.node.byte_range().end as u32;

						// `rust_spellcheck_query` test ensures that these invariants are correct
						match m.pattern_index {
							// ident
							0 => Fragment {
								kind: FragmentKind::Ident,
								span: Span::new(BytePos(start), BytePos(end)),
							},
							// sentence.string
							1 => Fragment {
								kind: FragmentKind::Sentence,
								// TODO: fix for complex strings
								span: Span::new(BytePos(start + 1), BytePos(end - 1)),
							},
							// sentence.comment
							2 => Fragment {
								kind: FragmentKind::Sentence,
								// TODO: fix for all types of comments
								span: Span::new(BytePos(start + 3), BytePos(end)),
							},
							_ => unreachable!(""),
						}
					})
					.collect::<Vec<_>>()
			})
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rust_spellcheck_query() {
		let grammar = tree_sitter_rust::language();
		let query = Query::new(&grammar, SPELLCHECK_QUERY).unwrap();

		assert_eq!(
			query.capture_names(),
			["ident", "sentence.string", "sentence.comment"]
		);
	}
}
