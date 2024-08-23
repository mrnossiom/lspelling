use tower_lsp::lsp_types::Command;

pub const ADD_TO_DICT: &str = "add-to-dict";
pub struct AddToDict;
impl AddToDict {
	pub fn command(word: String) -> Command {
		Command {
			title: "Add a word to the workspace dictionary".into(),
			command: ADD_TO_DICT.into(),
			arguments: Some(vec![word.into()]),
		}
	}
}
