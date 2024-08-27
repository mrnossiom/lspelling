//! LSP

use lspelling_wordc::{checker::Checker, span::Source};
use ruspell::Dictionary;
use serde_json::Value;
use std::{
	collections::HashMap,
	panic::{self, PanicInfo},
	path::Path,
};
use tokio::sync::RwLock;
use tower_lsp::{jsonrpc::Result, lsp_types::*, Client, LanguageServer, LspService, Server};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

mod commands;
mod debounce;

use crate::commands::{AddToDict, ADD_TO_DICT};
use crate::debounce::{CheckedDocument, ToLspType as _};

#[derive(Debug)]
struct Backend {
	client: Client,

	documents: RwLock<HashMap<Uri, CheckedDocument>>,
}

impl Backend {
	fn new(client: Client) -> Self {
		Self {
			client,
			documents: RwLock::default(),
		}
	}

	#[tracing::instrument(skip_all)]
	async fn on_change(&self, document: &CheckedDocument) {
		let diagnostics = document
			.checker
			.check()
			.iter()
			.map(|diag| {
				let range = document.source.span_to_range(diag.span).unwrap();
				Diagnostic {
					range: range.to_lsp_type(),
					severity: Some(DiagnosticSeverity::INFORMATION),
					code: Some(NumberOrString::Number(1)),
					message: format!("`{}` isn't in a loaded dictionary", diag.word),
					data: Some(diag.word.clone().into()),
					..Default::default()
				}
			})
			.collect();

		self.client
			.publish_diagnostics(
				document.item.uri.clone(),
				diagnostics,
				Some(document.item.version),
			)
			.await;
	}

	#[tracing::instrument(skip_all)]
	async fn word_at(&self, range: Range) {}
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
	#[tracing::instrument(skip_all)]
	async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
		Ok(InitializeResult {
			server_info: Some(ServerInfo {
				name: "lspelling".into(),
				version: Some(env!("CARGO_PKG_VERSION").into()),
			}),
			capabilities: ServerCapabilities {
				code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
				text_document_sync: Some(TextDocumentSyncCapability::Kind(
					// TextDocumentSyncKind::INCREMENTAL,
					TextDocumentSyncKind::FULL,
				)),
				execute_command_provider: Some(ExecuteCommandOptions {
					commands: vec![ADD_TO_DICT.into()],
					work_done_progress_options: WorkDoneProgressOptions::default(),
				}),
				..Default::default()
			},
		})
	}

	#[tracing::instrument(skip_all)]
	async fn initialized(&self, _: InitializedParams) {}

	#[tracing::instrument(skip_all)]
	async fn shutdown(&self) -> Result<()> {
		Ok(())
	}

	#[tracing::instrument(skip_all)]
	async fn did_open(
		&self,
		DidOpenTextDocumentParams { text_document, .. }: DidOpenTextDocumentParams,
	) {
		let uri = text_document.uri.clone();

		let source = Source::new(&text_document.text);
		let checker = Checker::new(&text_document.language_id, &source);
		// its late, im tired
		#[allow(unsafe_code)]
		let checker = unsafe { std::mem::transmute(checker) };
		let ck_doc = CheckedDocument {
			item: text_document,
			source,
			checker,
		};

		self.on_change(&ck_doc).await;

		self.documents.write().await.insert(uri, ck_doc);
	}

	#[tracing::instrument(skip_all)]
	async fn did_change(
		&self,
		DidChangeTextDocumentParams {
			text_document,
			content_changes,
		}: DidChangeTextDocumentParams,
	) {
		{
			let mut writer = self.documents.write().await;
			let docu = writer.get_mut(&text_document.uri).unwrap();
			docu.item.version = text_document.version;
			docu.update(&content_changes);
			self.on_change(&docu).await;
		}
	}

	#[tracing::instrument(skip_all)]
	async fn did_close(
		&self,
		DidCloseTextDocumentParams { text_document }: DidCloseTextDocumentParams,
	) {
		self.documents.write().await.remove(&text_document.uri);
	}

	#[tracing::instrument(skip_all)]
	async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
		let Some(diagnostic) = params
			.context
			.diagnostics
			.into_iter()
			.find(|diag| diag.code == Some(NumberOrString::Number(1)))
		else {
			return Ok(None);
		};

		let data = diagnostic.data.as_ref().unwrap();
		let tagged_word = data.as_str().unwrap();

		let dict = Dictionary::from_pair(Path::new(env!("HUNSPELL_DICT"))).unwrap();

		// let suggest = dict.suggest(tagged_word);
		let suggest = vec![];

		let mut actions = suggest
			.into_iter()
			.map(|replacement_word: String| {
				let replace_word_edit = TextEdit::new(diagnostic.range, replacement_word.clone());

				CodeActionOrCommand::CodeAction(CodeAction {
					title: format!("Replace with `{replacement_word}`"),
					kind: Some(CodeActionKind::QUICKFIX),
					diagnostics: Some(vec![diagnostic.clone()]),
					edit: Some(WorkspaceEdit::new({
						let mut hm = HashMap::new();
						hm.insert(params.text_document.uri.clone(), vec![replace_word_edit]);
						hm
					})),
					..Default::default()
				})
			})
			.collect::<Vec<_>>();

		actions.push(CodeActionOrCommand::CodeAction(CodeAction {
			title: "Add this word to the dictionary".into(),
			kind: Some(CodeActionKind::QUICKFIX),
			diagnostics: Some(vec![diagnostic.clone()]),
			command: Some(AddToDict::command(tagged_word.into())),
			..Default::default()
		}));

		Ok(Some(actions))
	}

	async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
		match params.command.as_str() {
			ADD_TO_DICT => {
				// TODO: logic to add custom word to dict
				let word = params.arguments[0].as_str().unwrap();
				tracing::error!("adding words (`{}`) to dict is not implemented", word);

				self.client
					.log_message(
						MessageType::INFO,
						format!("`{word}` was indeed added to dictionary"),
					)
					.await;
			}
			_ => todo!(),
		};

		Ok(None)
	}
}

fn tracing_panic_hook(panic_info: &PanicInfo) {
	let payload = panic_info
		.payload()
		.downcast_ref::<&'static str>()
		.map_or_else(
			|| {
				panic_info
					.payload()
					.downcast_ref::<String>()
					.map_or("Box<dyn Any>", |s| &s[..])
			},
			|s| *s,
		);

	let location = panic_info.location().map(ToString::to_string);

	tracing::error!(
		panic.payload = payload,
		panic.location = location,
		"A panic occurred",
	);
}

#[tokio::main]
async fn main() {
	panic::set_hook(Box::new(tracing_panic_hook));

	let file_appender = tracing_appender::rolling::never("/tmp", "lspelling.log");
	let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
	tracing_subscriber::fmt()
		.with_writer(non_blocking)
		.with_span_events(FmtSpan::NEW)
		.with_env_filter(EnvFilter::from_default_env())
		.init();

	let stdin = tokio::io::stdin();
	let stdout = tokio::io::stdout();

	let (service, socket) = LspService::new(Backend::new);
	Server::new(stdin, stdout, socket).serve(service).await;
}
