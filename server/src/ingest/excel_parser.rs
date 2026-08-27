use calamine::{Reader, open_workbook_auto, Data};

/// Parse an Excel file and return (headers, rows) in the same format as CSV.
/// Each row is a Vec<String> of cell values.
pub fn parse_excel(path: &std::path::Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| format!("failed to open Excel file: {e}"))?;

    // Get the first sheet name
    let sheet_name = workbook.sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| "Excel file has no sheets".to_string())?;

    let range = workbook.worksheet_range(&sheet_name)
        .map_err(|e| format!("failed to read sheet '{sheet_name}': {e}"))?;

    let mut rows: Vec<Vec<String>> = Vec::new();
    for row in range.rows() {
        let cells: Vec<String> = row.iter().map(|cell| {
            match cell {
                Data::Empty => String::new(),
                Data::String(s) => s.clone(),
                Data::Float(f) => {
                    if *f == (*f as i64) as f64 {
                        format!("{}", *f as i64)
                    } else {
                        format!("{f}")
                    }
                }
                Data::Int(i) => format!("{i}"),
                Data::Bool(b) => format!("{b}"),
                Data::Error(e) => format!("ERR:{e:?}"),
                Data::DateTime(d) => format!("{d}"),
                _ => String::new(),
            }
        }).collect();
        rows.push(cells);
    }

    if rows.is_empty() {
        return Err("Excel sheet is empty".to_string());
    }

    // First row is headers
    let headers: Vec<String> = rows.remove(0)
        .into_iter()
        .map(|h| if h.is_empty() { "column".to_string() } else { h })
        .collect();

    // Deduplicate header names
    let mut seen = std::collections::HashMap::new();
    let headers: Vec<String> = headers.into_iter().enumerate().map(|(_i, h)| {
        let count = seen.entry(h.clone()).or_insert(0u32);
        *count += 1;
        if *count > 1 {
            format!("{h}_{count}")
        } else {
            h
        }
    }).collect();

    Ok((headers, rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nonexistent() {
        let result = parse_excel(std::path::Path::new("/nonexistent.xlsx"));
        assert!(result.is_err());
    }
}
