use std::io::Read;

/// Extrait le texte d'un CV à partir de ses octets et de son nom de fichier.
///
/// Formats supportés :
/// - **PDF** (via `pdf-extract`)
/// - **DOCX** (dézippage + extraction XML)
/// - **TXT / MD / CSV** (UTF-8)
/// - fallback : décodage UTF-8 tolérant
pub fn extract_text(filename: &str, bytes: &[u8]) -> anyhow::Result<String> {
    let lower = filename.to_lowercase();
    let text = if lower.ends_with(".pdf") {
        extract_pdf(bytes)?
    } else if lower.ends_with(".docx") {
        extract_docx(bytes)?
    } else if lower.ends_with(".doc") {
        // Format binaire ancien : on tente un décodage tolérant.
        decode_lossy(bytes)
    } else {
        decode_lossy(bytes)
    };
    Ok(clean(&text))
}

fn extract_pdf(bytes: &[u8]) -> anyhow::Result<String> {
    match pdf_extract::extract_text_from_mem(bytes) {
        Ok(t) => Ok(t),
        Err(e) => {
            tracing::warn!("échec extraction PDF: {e}");
            // Dernier recours : décodage brut (peut donner du bruit, mieux que rien).
            Ok(decode_lossy(bytes))
        }
    }
}

fn extract_docx(bytes: &[u8]) -> anyhow::Result<String> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut xml = String::new();
    {
        let mut file = zip.by_name("word/document.xml")?;
        file.read_to_string(&mut xml)?;
    }
    // Retire les balises XML et insère des espaces sur les fins de paragraphes.
    let xml = xml.replace("</w:p>", "\n").replace("</w:tr>", "\n");
    let mut out = String::with_capacity(xml.len());
    let mut in_tag = false;
    for ch in xml.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    Ok(out
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'"))
}

fn decode_lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Nettoie le texte extrait : lignes vides multiples, espaces parasites.
fn clean(text: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut last_empty = true;
    for raw in text.lines() {
        let line = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() {
            if !last_empty {
                lines.push(String::new());
                last_empty = true;
            }
        } else {
            lines.push(line);
            last_empty = false;
        }
    }
    lines.join("\n").trim().to_string()
}

/// Devine le type MIME depuis un nom de fichier.
#[allow(dead_code)]
pub fn guess_mime(filename: &str) -> String {
    mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string()
}
