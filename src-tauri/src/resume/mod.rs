//! Resume text extraction (PDF / DOCX) for onboarding.

/// Extract plain text from PDF bytes.
pub fn extract_pdf(bytes: &[u8]) -> Result<String, String> {
    pdf_extract::extract_text_from_mem(bytes).map_err(|e| format!("PDF extract failed: {e}"))
}

use std::io::Read;

/// Extract plain text from DOCX bytes by reading `word/document.xml`.
pub fn extract_docx(bytes: &[u8]) -> Result<String, String> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| format!("not a valid docx (zip): {e}"))?;
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .map_err(|e| format!("docx missing word/document.xml: {e}"))?
        .read_to_string(&mut xml)
        .map_err(|e| format!("read document.xml: {e}"))?;

    use quick_xml::events::Event;
    use quick_xml::reader::Reader;
    let mut reader = Reader::from_str(&xml);
    let mut out = String::new();
    let mut in_text = false;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"w:t" => in_text = true,
            Ok(Event::End(e)) if e.name().as_ref() == b"w:t" => in_text = false,
            Ok(Event::End(e)) if e.name().as_ref() == b"w:p" => out.push('\n'),
            Ok(Event::Text(t)) if in_text => {
                out.push_str(&t.unescape().map_err(|e| format!("xml unescape: {e}"))?);
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("xml parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok(out.trim().to_string())
}

/// Choose an extractor by file extension and read the file from disk.
pub fn extract_from_path(path: &str) -> Result<String, String> {
    let lower = path.to_lowercase();
    if !(lower.ends_with(".pdf") || lower.ends_with(".docx")) {
        return Err("Formato não suportado: use PDF ou DOCX".to_string());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
    if lower.ends_with(".pdf") {
        extract_pdf(&bytes)
    } else {
        extract_docx(&bytes)
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn extract_from_path_rejects_unknown_extension() {
        let err = extract_from_path("resume.txt").unwrap_err();
        assert!(err.contains("Formato não suportado"));
    }
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

#[cfg(test)]
mod docx_tests {
    use super::*;

    #[test]
    fn extract_docx_reads_paragraph_text() {
        let bytes = include_bytes!("../../tests/fixtures/sample.docx");
        let text = extract_docx(bytes).expect("extract docx");
        assert!(text.contains("Senior Frontend Developer"), "got: {text:?}");
        assert!(text.contains("React and TypeScript"), "got: {text:?}");
    }
}
