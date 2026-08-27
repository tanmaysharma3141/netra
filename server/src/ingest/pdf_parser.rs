use std::path::Path;

/// Parse a PDF file and return (headers, rows).
/// Extracts text and attempts to parse tabular data (CDR/bank exports).
pub fn parse_pdf(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let text = pdf_extract::extract_text(path)
        .map_err(|e| format!("PDF extraction failed: {e}"))?;

    if text.trim().is_empty() {
        return Err("PDF contains no extractable text (may be scanned/image-based)".to_string());
    }

    // Try to parse as table
    parse_text_table(&text)
}

/// Parse extracted text into a table structure.
/// Handles common patterns in CDR/bank PDF exports:
/// - Tab-separated values
/// - Pipe-separated values
/// - Multiple-space separated columns
/// - Aligned columns (fixed-width)
fn parse_text_table(text: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let lines: Vec<&str> = text.lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return Err("No text content in PDF".to_string());
    }

    // Detect delimiter by counting occurrences in first few lines
    let sample: String = lines.iter().take(10).cloned().collect::<Vec<_>>().join("\n");

    let delimiter = detect_table_delimiter(&sample);

    // Parse all lines using detected delimiter
    let parsed_lines: Vec<Vec<String>> = lines.iter()
        .map(|line| split_row(line, delimiter))
        .collect();

    // Find header row: first row with 3+ non-empty cells that differs from the next
    let header_idx = find_header_row(&parsed_lines);

    if header_idx.is_none() {
        // No clear header — try to synthesize one
        let num_cols = parsed_lines.first().map(|r| r.len()).unwrap_or(0);
        if num_cols < 2 {
            return Err("Could not detect table structure in PDF".to_string());
        }
        let headers: Vec<String> = (0..num_cols)
            .map(|i| format!("column_{i}"))
            .collect();
        let rows: Vec<Vec<String>> = parsed_lines.into_iter()
            .filter(|r| r.len() >= num_cols - 1)
            .collect();
        return Ok((headers, rows));
    }

    let hi = header_idx.unwrap();
    let headers = parsed_lines[hi].clone();
    let num_cols = headers.len();

    let rows: Vec<Vec<String>> = parsed_lines.into_iter()
        .skip(hi + 1)
        .filter(|r| r.len() >= (num_cols.saturating_sub(1)))
        .map(|r| {
            // Pad or truncate to match header length
            let mut row = r;
            while row.len() < num_cols {
                row.push(String::new());
            }
            row.truncate(num_cols);
            row
        })
        .collect();

    if rows.is_empty() {
        return Err("PDF table detected but no data rows found".to_string());
    }

    Ok((headers, rows))
}

/// Detect the most likely delimiter for a table
fn detect_table_delimiter(sample: &str) -> char {
    let counts = [
        ('\t', count_char(sample, '\t')),
        ('|', count_char(sample, '|')),
        (',', count_char(sample, ',')),
        (';', count_char(sample, ';')),
    ];

    // Find the delimiter with the highest consistent count
    let line_count = sample.lines().count().max(1) as f64;

    let best = counts.iter()
        .max_by_key(|(_, count)| {
            let per_line = *count as f64 / line_count;
            // Prefer delimiters that appear roughly once per line (table columns)
            (per_line * 100.0) as u64
        })
        .map(|(c, _)| *c)
        .unwrap_or(',');

    best
}

fn count_char(text: &str, c: char) -> usize {
    text.chars().filter(|&ch| ch == c).count()
}

/// Split a row by delimiter, handling quoted fields
fn split_row(line: &str, delimiter: char) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' if delimiter != '"' => in_quotes = !in_quotes,
            c if c == delimiter && !in_quotes => {
                result.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    result.push(current.trim().to_string());

    // Filter out completely empty trailing cells
    while result.last().map(|s| s.is_empty()).unwrap_or(false) {
        result.pop();
    }

    result
}

/// Find the most likely header row index
fn find_header_row(rows: &[Vec<String>]) -> Option<usize> {
    if rows.len() < 2 {
        return Some(0);
    }

    for (i, row) in rows.iter().enumerate().take(5) {
        // Header should have 2+ non-empty cells
        let non_empty = row.iter().filter(|s| !s.is_empty()).count();
        if non_empty < 2 {
            continue;
        }

        // Check if this row looks like a header (contains alphabetic text)
        let has_alpha = row.iter().any(|s| s.chars().any(|c| c.is_alphabetic()));
        if !has_alpha {
            continue;
        }

        // Check if next row is different (data row)
        if i + 1 < rows.len() {
            let next = &rows[i + 1];
            // Data rows typically have more numeric content
            let next_numeric = next.iter()
                .filter(|s| s.parse::<f64>().is_ok())
                .count();
            if next_numeric > 0 || row != next {
                return Some(i);
            }
        } else {
            return Some(i);
        }
    }

    Some(0)
}
