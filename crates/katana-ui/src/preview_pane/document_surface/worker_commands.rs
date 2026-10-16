use katana_document_viewer::{
    DocumentSession, DocumentSessionCommand, DocumentSessionError, DocumentSessionEvent,
    SpreadsheetFilterCommand, SpreadsheetFilterEvent,
};

#[derive(Debug, Clone, PartialEq)]
pub(in super::super) enum DocumentWorkerCommand {
    Viewer(katana_document_viewer::DocumentViewerCommand),
    Surface(katana_document_viewer::DocumentSurfaceCommand),
    SpreadsheetFilter(SpreadsheetFilterCommand),
}

pub(super) fn apply(
    session: &mut DocumentSession,
    command: DocumentWorkerCommand,
) -> Result<(DocumentSessionEvent, Option<SpreadsheetFilterEvent>), DocumentSessionError> {
    match command {
        DocumentWorkerCommand::Viewer(command) => session
            .apply(DocumentSessionCommand::Viewer(command))
            .map(|event| (event, None)),
        DocumentWorkerCommand::Surface(command) => session
            .apply(DocumentSessionCommand::Surface(command))
            .map(|event| (event, None)),
        DocumentWorkerCommand::SpreadsheetFilter(command) => session
            .apply_spreadsheet_filter(command)
            .map(|event| (DocumentSessionEvent::None, Some(event))),
    }
}
