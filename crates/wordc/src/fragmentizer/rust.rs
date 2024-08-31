use std::fmt;
use tree_sitter::{Parser, Query, QueryCapture, QueryCursor, QueryMatch, Tree};

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

		let tree = parser.parse(source.0.slice(..).to_string(), None).unwrap();

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
		let source = self.source.0.slice(..).to_string();
		let matches = cursor.matches(&self.query, self.tree.root_node(), source.as_bytes());
		let patterns = self.query.capture_names();

		let capture_to_fragment = |match_: &QueryMatch, capture: &QueryCapture| {
			let start = capture.node.byte_range().start as u32;
			let end = capture.node.byte_range().end as u32;
			let mut span = Span::new(BytePos(start), BytePos(end));

			// TODO: pattern index doesn't match patterns map in any circonstances
			let kind = match patterns[match_.pattern_index] {
				"ident" => FragmentKind::Ident,
				"sentence.string" | "sentence.comment" => FragmentKind::Sentence,
				"sentence.comment.raw" => {
					// Special handling for raw comments content until
					// we have a way to access comment content
					match capture.node.grammar_name() {
						"line_comment" => {
							span.low = span.low + BytePos(2);
						}
						"block_comment" => {
							span.low = span.low + BytePos(2);
							span.high = span.high - BytePos(2);
						}
						_ => {}
					};
					FragmentKind::Sentence
				}
				_ => unreachable!("this part is kept in sync with query"),
			};

			Fragment { kind, span }
		};

		matches
			.flat_map(|match_| {
				match_
					.captures
					.iter()
					.map(|capture| capture_to_fragment(&match_, capture))
					.collect::<Vec<_>>()
			})
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn query_expect_patterns() {
		let grammar = tree_sitter_rust::language();
		let query = Query::new(&grammar, SPELLCHECK_QUERY).unwrap();

		assert_eq!(
			query.capture_names(),
			[
				"ident",
				"sentence.string",
				"sentence.comment",
				"sentence.comment.raw"
			]
		);
	}
}
