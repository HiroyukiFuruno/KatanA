use super::HtmlBrowserSurface;
use katana_document_viewer::browser_session::BrowserSessionAdapterError;

impl HtmlBrowserSurface {
    pub(super) fn record_adapter_error(
        &mut self,
        operation: &'static str,
        document_origin: Option<String>,
        error: BrowserSessionAdapterError,
    ) {
        let document = document_origin
            .or_else(|| self.document_origin.clone())
            .unwrap_or_else(|| "unknown".to_string());
        if is_secondary_worker_error(&error)
            && let Some(primary_error) = self.error.as_deref()
        {
            tracing::error!(
                layer = "KDV worker",
                operation,
                document,
                cause = %error,
                primary_error,
                "HTML browser worker failed after a primary error"
            );
            return;
        }
        self.record_failure("KDV worker", operation, document, error.to_string());
    }

    pub(super) fn record_failure(
        &mut self,
        layer: &'static str,
        operation: &'static str,
        document: String,
        cause: String,
    ) {
        tracing::error!(
            layer,
            operation,
            document,
            cause,
            "HTML browser operation failed"
        );
        self.record_error(browser_failure_report(layer, operation, &document, &cause));
    }
}

fn is_secondary_worker_error(error: &BrowserSessionAdapterError) -> bool {
    matches!(
        error,
        BrowserSessionAdapterError::WorkerStopped | BrowserSessionAdapterError::WorkerPanicked
    )
}

fn browser_failure_report(layer: &str, operation: &str, document: &str, cause: &str) -> String {
    format!("Layer: {layer}\nOperation: {operation}\nDocument: {document}\nCause: {cause}")
}
