use clap::Parser;
use std::collections::HashMap;
use std::fmt::Write;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionOptions, DiagnosticOptions, DiagnosticServerCapabilities,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, Hover, HoverContents, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, MarkedString,
    MessageType, PositionEncodingKind, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, WorkDoneProgressOptions,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};
use zspell::Dictionary;

use crate::definitions::{get_definitions, word_at_position};
use crate::spellcheck::{get_dict, spellcheck_diagnostics};

mod args;
mod data_dirs;
mod definitions;
mod spellcheck;
mod typo_correction;

#[derive(Debug)]
struct Backend {
    client: Client,
    dictionary: Dictionary,
    text: Mutex<String>,
    // TODO: add garbage collector to not grow this too much
    // maybe extract to seperate specialised struct but we'll see
    lookup_cache: Mutex<HashMap<String, String>>,
}

impl Backend {
    fn new(client: Client, dictionary: Dictionary) -> Self {
        Self {
            client,
            dictionary,
            text: Mutex::new(String::new()),
            lookup_cache: HashMap::new().into(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(text) = &params.content_changes.first() {
            let mut file = self.text.lock().await;
            file.clone_from(&text.text);
        }
        let dict = &self.dictionary;
        let text = self.text.lock().await;
        let diags = spellcheck_diagnostics(&*text, dict);
        self.client
            .publish_diagnostics(params.text_document.uri, diags, None)
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let text = params.text_document.text;
        let mut file = self.text.lock().await;
        *file = text;
        self.client
            .publish_diagnostics(
                params.text_document.uri,
                spellcheck_diagnostics(&*file, &self.dictionary),
                None,
            )
            .await;
    }

    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                position_encoding: Some(PositionEncodingKind::UTF8),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: None,
                        inter_file_dependencies: false,
                        workspace_diagnostics: true,
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    // async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
    //     Ok(Some(CompletionResponse::Array(vec![])))
    // }

    async fn hover(&self, p: HoverParams) -> Result<Option<Hover>> {
        let pos = p.text_document_position_params.position;
        let text = self.text.lock().await;
        let word = word_at_position(&text, pos);
        let mut hover = String::new();
        if let Some(word) = word {
            if let Some(cached_word) = self.lookup_cache.lock().await.get(word.trim()) {
                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(cached_word.clone())),
                    range: None,
                }));
            }
            let definitions = get_definitions(word).await;
            if let Ok(definitions) = definitions {
                let mut cache = self.lookup_cache.lock().await;
                definitions.iter().fold(&mut hover, |acc, d| {
                    _ = writeln!(acc, "{d}");
                    acc
                });
                cache.insert(word.trim().to_string(), hover.clone());
            }
        }
        if hover.is_empty() {
            return Ok(None);
        }
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(hover)),
            range: None,
        }))
    }
}

#[tokio::main]
async fn main() {
    let args = args::Args::parse();
    let client = reqwest::Client::new();
    let dict = get_dict(&client, args.lang).await.unwrap();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client, dict));
    Server::new(stdin, stdout, socket).serve(service).await;
}
