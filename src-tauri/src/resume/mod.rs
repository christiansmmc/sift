//! Resume text extraction (PDF / DOCX) for onboarding.

/// Extract plain text from PDF bytes.
pub fn extract_pdf(bytes: &[u8]) -> Result<String, String> {
    pdf_extract::extract_text_from_mem(bytes).map_err(|e| format!("PDF extract failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_pdf_reads_embedded_text() {
        let bytes = include_bytes!("../../tests/fixtures/sample.pdf");
        let text = extract_pdf(bytes).expect("extract pdf");
        assert!(
            text.contains("Backend Engineer"),
            "expected extracted text to contain the CV phrase, got: {text:?}"
        );
    }
}
