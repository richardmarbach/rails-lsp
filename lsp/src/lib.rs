pub mod workspace;

use lsp_server::{Connection, IoThreads, Message};
use lsp_types::{
    FileOperationFilter, FileOperationPattern, FileOperationPatternKind,
    FileOperationRegistrationOptions, InitializeParams, OneOf, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, WorkspaceFileOperationsServerCapabilities,
    WorkspaceFolder, WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};
use thiserror::Error;

use crate::workspace::Workspace;

pub fn run() -> anyhow::Result<()> {
    let (connection, io_threads) = Connection::stdio();
    start(connection, io_threads)
}

pub fn start(connection: Connection, io_threads: IoThreads) -> anyhow::Result<()> {
    eprintln!("Starting Rails LSP");

    let filters: Vec<FileOperationFilter> = vec![FileOperationFilter {
        pattern: FileOperationPattern {
            glob: "*.rb".to_string(),
            matches: Some(FileOperationPatternKind::File),
            ..Default::default()
        },
        ..Default::default()
    }];

    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        definition_provider: Some(OneOf::Left(true)),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Right(
                    "workspace/didChangeWorkspaceFolders".to_string(),
                )),
            }),
            file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                did_create: Some(FileOperationRegistrationOptions {
                    filters: filters.clone(),
                }),
                did_rename: Some(FileOperationRegistrationOptions {
                    filters: filters.clone(),
                }),
                did_delete: Some(FileOperationRegistrationOptions {
                    filters: filters.clone(),
                }),
                ..Default::default()
            }),
        }),

        ..Default::default()
    })
    .unwrap();

    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    eprintln!("Shutting down server");

    Ok(())
}

fn main_loop(connection: Connection, params: serde_json::Value) -> anyhow::Result<()> {
    let params: InitializeParams = serde_json::from_value(params).unwrap();

    let workspace = load_workspace(params.workspace_folders)?;

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                eprintln!("got request: {req:?}");
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }

                match req.method.as_str() {
                    _ => {
                        eprintln!("Unsupported method: {}", req.method);
                        continue;
                    }
                }
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
            Message::Notification(not) => {
                eprintln!("got notification: {not:?}");
            }
        }
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum LspError {
    #[error("failed to load workspace")]
    WorkspaceError(#[from] std::io::Error),
    #[error("unknown lsp error")]
    Unknown,
}

fn load_workspace(workspace_folders: Option<Vec<WorkspaceFolder>>) -> Result<Workspace, LspError> {
    let mut workspace_folders: Vec<_> = workspace_folders
        .into_iter()
        .flat_map(|folders| {
            folders.into_iter().filter_map(|folder| {
                eprintln!("Folder: {:#?}", folder.uri.scheme());
                if folder.uri.scheme() == "file" {
                    folder.uri.to_file_path().ok()
                } else {
                    eprintln!("Remote workspace not supported: {}", folder.uri);
                    None
                }
            })
        })
        .collect();

    if workspace_folders.is_empty() {
        let current_dir = std::env::current_dir().map_err(|err| LspError::WorkspaceError(err))?;
        workspace_folders.push(current_dir);
    }

    Ok(Workspace::from_paths(workspace_folders))
}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_load_workspaces() {
//         let workspace_folders = vec!["../../rails-sample/".into()];
//         let workspace = Workspace::from_paths(workspace_folders);
//         eprintln!("Constants: {:#?}", workspace.constants);
//     }
// }
