use lspelling_wordc::{checker::Checker, span::Source};
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, TextDocumentItem};

// TODO: wtf is this module

#[derive(Debug)]
pub(crate) struct CheckedDocument {
	pub(crate) item: TextDocumentItem,

	pub(crate) source: Source,

	pub(crate) checker: Checker<'static>,
}

impl CheckedDocument {
	pub(crate) fn update(&mut self, changes: &[TextDocumentContentChangeEvent]) {
		if let [TextDocumentContentChangeEvent {
			range: None, text, ..
		}] = changes
		{
			// TODO: change to incremental changes
			self.source = Source::new(text);

			#[allow(unsafe_code)]
			self.checker
				.replace_src(unsafe { std::mem::transmute::<&Source, &Source>(&self.source) });

			return;
		}

		todo!("incremental changes")

		// for TextDocumentContentChangeEvent { range, text, .. } in changes {
		// 	let range = range.unwrap();
		// 	self.source.0.remove(range.start..=range.end);
		// 	todo!()
		// }
	}
}

pub(crate) trait ToLspType: Sized {
	type Target;

	fn to_lsp_type(self) -> Self::Target;
}

impl ToLspType for lspelling_wordc::span::Position {
	type Target = Position;

	fn to_lsp_type(self) -> Self::Target {
		Position::new(self.0, self.1)
	}
}

impl ToLspType for lspelling_wordc::span::Range {
	type Target = Range;

	fn to_lsp_type(self) -> Self::Target {
		Range::new(self.0.to_lsp_type(), self.1.to_lsp_type())
	}
}
