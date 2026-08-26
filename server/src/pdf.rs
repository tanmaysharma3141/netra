use printpdf::*;
use std::collections::BTreeMap;

/// Generate a PDF from a markdown report string.
/// Converts markdown to simple HTML, then renders via printpdf's HTML engine.
pub fn generate_report_pdf(title: &str, markdown: &str) -> Vec<u8> {
    let html = markdown_to_html(title, markdown);

    let mut fonts: BTreeMap<String, Base64OrRaw> = BTreeMap::new();
    let images: BTreeMap<String, Base64OrRaw> = BTreeMap::new();

    // Try to find system fonts, fall back gracefully
    load_system_fonts(&mut fonts);

    let options = GeneratePdfOptions::default();
    let mut warnings = Vec::new();

    match PdfDocument::from_html(&html, &images, &fonts, &options, &mut warnings) {
        Ok(doc) => {
            let mut save_warnings = Vec::new();
            doc.save(&PdfSaveOptions::default(), &mut save_warnings)
        }
        Err(e) => {
            tracing::error!(err = %e, "PDF generation failed, falling back to text PDF");
            generate_fallback_pdf(title, markdown)
        }
    }
}

/// Load system fonts for PDF rendering
fn load_system_fonts(fonts: &mut BTreeMap<String, Base64OrRaw>) {
    let font_dirs = if cfg!(target_os = "windows") {
        vec!["C:\\Windows\\Fonts"]
    } else if cfg!(target_os = "macos") {
        vec!["/System/Library/Fonts", "/Library/Fonts"]
    } else {
        vec!["/usr/share/fonts", "/usr/local/share/fonts"]
    };

    // Try to load common sans-serif fonts
    let font_names = [
        ("Arial", vec!["arial.ttf", "Arial.ttf", "arialbd.ttf"]),
        ("Helvetica", vec!["helvetica.ttf", "Helvetica.ttf"]),
        ("DejaVuSans", vec!["DejaVuSans.ttf", "dejavu-sans-fonts/DejaVuSans.ttf"]),
        ("LiberationSans", vec![
            "LiberationSans-Regular.ttf",
            "liberation-sans/LiberationSans-Regular.ttf",
        ]),
    ];

    for (name, files) in &font_names {
        if fonts.contains_key(*name) {
            continue;
        }
        for font_dir in &font_dirs {
            for file in files {
                let path = format!("{}/{}", font_dir, file);
                if let Ok(data) = std::fs::read(&path) {
                    fonts.insert(name.to_string(), Base64OrRaw::Raw(data));
                    tracing::debug!(font = %name, path = %path, "loaded font for PDF");
                    break;
                }
            }
            if fonts.contains_key(*name) {
                break;
            }
        }
    }
}

/// Convert markdown to simple HTML for PDF rendering
fn markdown_to_html(title: &str, markdown: &str) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  body {{ font-family: Arial, Helvetica, sans-serif; font-size: 11pt; margin: 20mm; color: #1a1a1a; }}
  h1 {{ font-size: 22pt; border-bottom: 2px solid #333; padding-bottom: 8px; }}
  h2 {{ font-size: 16pt; color: #2c3e50; margin-top: 16px; }}
  h3 {{ font-size: 13pt; color: #34495e; }}
  p {{ margin: 6px 0; line-height: 1.5; }}
  ul {{ margin: 4px 0 4px 20px; }}
  li {{ margin: 3px 0; }}
  strong {{ font-weight: bold; }}
  hr {{ border: none; border-top: 1px solid #ccc; margin: 12px 0; }}
  .footer {{ font-size: 8pt; color: #888; margin-top: 40px; border-top: 1px solid #ddd; padding-top: 8px; }}
</style>
</head>
<body>
<h1>{}</h1>
"#,
        escape_html(title)
    );

    // Simple markdown → HTML conversion
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            html.push_str("<br>\n");
        } else if trimmed.starts_with("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", escape_html(&trimmed[4..])));
        } else if trimmed.starts_with("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", escape_html(&trimmed[3..])));
        } else if trimmed.starts_with("# ") {
            // Skip — we already rendered the title
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let text = &trimmed[2..];
            html.push_str(&format!("<li>{}</li>\n", markdown_inline(text)));
        } else if trimmed.starts_with("---") {
            html.push_str("<hr>\n");
        } else if trimmed.starts_with("**") && trimmed.ends_with("**") {
            let text = &trimmed[2..trimmed.len() - 2];
            html.push_str(&format!("<p><strong>{}</strong></p>\n", escape_html(text)));
        } else {
            html.push_str(&format!("<p>{}</p>\n", markdown_inline(trimmed)));
        }
    }

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");
    html.push_str(&format!(
        r#"<div class="footer">NETRA Intelligence Report | Generated {}</div>
</body>
</html>"#,
        now
    ));

    html
}

/// Handle inline markdown: **bold**, `code`, etc.
fn markdown_inline(text: &str) -> String {
    let mut result = text.to_string();
    // Bold: **text**
    while let Some(start) = result.find("**") {
        if let Some(end) = result[start + 2..].find("**") {
            let bold_text = &result[start + 2..start + 2 + end].to_string();
            let replacement = format!("<strong>{}</strong>", escape_html(bold_text));
            result = format!("{}{}{}", &result[..start], replacement, &result[start + 2 + end + 2..]);
        } else {
            break;
        }
    }
    // Inline code: `text`
    while let Some(start) = result.find('`') {
        if let Some(end) = result[start + 1..].find('`') {
            let code_text = &result[start + 1..start + 1 + end].to_string();
            let replacement = format!("<code>{}</code>", escape_html(code_text));
            result = format!("{}{}{}", &result[..start], replacement, &result[start + 1 + end + 1..]);
        } else {
            break;
        }
    }
    result
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Fallback PDF generation when HTML rendering fails — plain text PDF
fn generate_fallback_pdf(title: &str, text: &str) -> Vec<u8> {
    let html = format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"></head>
<body style="font-family: sans-serif; font-size: 11pt; margin: 20mm;">
<h1>{}</h1>
<pre style="white-space: pre-wrap; font-size: 10pt;">{}</pre>
</body></html>"#,
        escape_html(title),
        escape_html(text)
    );

    let fonts: BTreeMap<String, Base64OrRaw> = BTreeMap::new();
    let images: BTreeMap<String, Base64OrRaw> = BTreeMap::new();
    let options = GeneratePdfOptions::default();
    let mut warnings = Vec::new();

    match PdfDocument::from_html(&html, &images, &fonts, &options, &mut warnings) {
        Ok(doc) => {
            let mut save_warnings = Vec::new();
            doc.save(&PdfSaveOptions::default(), &mut save_warnings)
        }
        Err(e) => {
            tracing::error!(err = %e, "fallback PDF generation also failed");
            // Absolute last resort: return empty PDF
            b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/MediaBox[0 0 595 842]/Parent 2 0 R>>endobj\nxref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \ntrailer<</Size 4/Root 1 0 R>>\nstartxref\n190\n%%EOF".to_vec()
        }
    }
}
