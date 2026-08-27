pub mod detect;
pub mod excel_parser;
pub mod pdf_parser;

use std::path::Path;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use uuid::Uuid;

use crate::models::{Event, EntityType, EventType, LatLng, SourceType};
use detect::{build_column_map, detect_domain, detect_operator, sniff_delimiter, Domain};

const PROGRESS_EVERY: u64 = 10_000;
const MAX_ERRORS: usize = 100;

pub struct IngestResult {
    pub parsed: u64,
    pub errors: Vec<String>,
    pub domain: &'static str,
    pub operator: Option<&'static str>,
    pub events: Vec<Event>,
}

struct RowMap {
    map: Vec<(&'static str, usize)>,
}

impl RowMap {
    fn get_str(&self, cells: &[String], canonical: &str) -> Option<String> {
        self.map
            .iter()
            .filter(|(c, _)| *c == canonical)
            .find_map(|(_, idx)| cells.get(*idx))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn get_num(&self, cells: &[String], canonical: &str) -> Option<f64> {
        self.get_str(cells, canonical)?
            .replace([',', ' '], "")
            .parse::<f64>()
            .ok()
    }
}

pub fn run_parse(
    path: &Path,
    case_id: Uuid,
    file_stem: &str,
    mut on_progress: impl FnMut(u64),
) -> Result<IngestResult, String> {
    // Detect format by file extension
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "xlsx" | "xls" => parse_excel_file(path, case_id, file_stem, &mut on_progress),
        "pdf" => parse_pdf_file(path, case_id, file_stem, &mut on_progress),
        _ => parse_csv_file(path, case_id, file_stem, &mut on_progress),
    }
}

/// Parse CSV/TSV/TXT files (existing logic)
fn parse_csv_file(
    path: &Path,
    case_id: Uuid,
    file_stem: &str,
    on_progress: &mut impl FnMut(u64),
) -> Result<IngestResult, String> {
    let raw = std::fs::read(path).map_err(|e| format!("read failed: {e}"))?;
    let text_head = String::from_utf8_lossy(&raw[..raw.len().min(65_536)]).to_string();
    let delim = sniff_delimiter(&text_head);

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(true)
        .flexible(true)
        .from_reader(raw.as_slice());

    let headers: Vec<String> = match reader.headers() {
        Ok(h) => h.iter().map(String::from).collect(),
        Err(e) => return Err(format!("header parse failed: {e}")),
    };
    if headers.is_empty() {
        return Err("empty header row".into());
    }

    let row_iter = reader.records().filter_map(|r| r.ok()).map(|rec| {
        let cells: Vec<String> = rec.iter().map(String::from).collect();
        Ok(cells) as Result<Vec<String>, Box<dyn std::error::Error>>
    });
    process_rows(headers, row_iter, case_id, file_stem, &text_head, on_progress)
}

/// Parse Excel files via calamine
fn parse_excel_file(
    path: &Path,
    case_id: Uuid,
    file_stem: &str,
    on_progress: &mut impl FnMut(u64),
) -> Result<IngestResult, String> {
    let (headers, rows) = excel_parser::parse_excel(path)?;
    let row_iter = rows.into_iter().map(|r| Ok(r) as Result<Vec<String>, Box<dyn std::error::Error>>);
    process_rows(headers, row_iter, case_id, file_stem, "", on_progress)
}

/// Parse PDF files via pdf-extract
fn parse_pdf_file(
    path: &Path,
    case_id: Uuid,
    file_stem: &str,
    on_progress: &mut impl FnMut(u64),
) -> Result<IngestResult, String> {
    let (headers, rows) = pdf_parser::parse_pdf(path)?;
    let row_iter = rows.into_iter().map(|r| Ok(r) as Result<Vec<String>, Box<dyn std::error::Error>>);
    process_rows(headers, row_iter, case_id, file_stem, "", on_progress)
}

/// Unified processing pipeline: headers + row iterator → events
fn process_rows<RowIter>(
    headers: Vec<String>,
    row_iter: RowIter,
    case_id: Uuid,
    file_stem: &str,
    text_head: &str,
    on_progress: &mut impl FnMut(u64),
) -> Result<IngestResult, String>
where
    RowIter: Iterator<Item = Result<Vec<String>, Box<dyn std::error::Error>>>,
{
    if headers.is_empty() {
        return Err("empty header row".into());
    }

    let fp = detect_domain(&headers);
    if fp.score < 2 {
        return Err(format!(
            "could not identify schema (best guess {:?}, score {})",
            fp.domain, fp.score
        ));
    }
    let colmap = RowMap { map: build_column_map(&headers) };
    let operator = detect_operator(file_stem)
        .or_else(|| text_head.lines().take(3).find_map(detect_operator));

    let source_type = match fp.domain {
        Domain::Cdr => SourceType::Cdr,
        Domain::Ipdr => SourceType::Ipdr,
        Domain::Bank => SourceType::Bank,
        Domain::Social => SourceType::Social,
    };

    let mut events = Vec::new();
    let mut errors = Vec::new();
    let mut parsed: u64 = 0;

    for row_result in row_iter {
        let cells = match row_result {
            Ok(c) => c,
            Err(e) => {
                if errors.len() < MAX_ERRORS {
                    errors.push(format!("row {}: {e}", parsed + 1));
                }
                continue;
            }
        };

        match build_event_from_cells(&colmap, &cells, &headers, case_id, source_type, operator) {
            Ok(ev) => {
                events.push(ev);
                parsed += 1;
                if parsed % PROGRESS_EVERY == 0 {
                    on_progress(parsed);
                }
            }
            Err(e) => {
                if errors.len() < MAX_ERRORS {
                    errors.push(format!("row {}: {e}", parsed + 1));
                }
            }
        }
    }
    on_progress(parsed);

    Ok(IngestResult { parsed, errors, domain: fp.domain.as_str(), operator, events })
}

/// Build an event from a slice of cell values (used by all parsers)
#[allow(clippy::too_many_arguments)]
fn build_event_from_cells(
    m: &RowMap,
    cells: &[String],
    headers: &[String],
    case_id: Uuid,
    source_type: SourceType,
    operator: Option<&'static str>,
) -> Result<Event, String> {
    let ts = parse_ts(
        m.get_str(cells, "ts").as_deref(),
        m.get_str(cells, "date").as_deref(),
        m.get_str(cells, "time").as_deref(),
    )
    .ok_or_else(|| "unparseable/missing timestamp".to_string())?;

    let (entity_id, entity_type, event_type, value) = match source_type {
        SourceType::Cdr | SourceType::Ipdr => {
            let party = m
                .get_str(cells, "a_party")
                .or_else(|| m.get_str(cells, "phone"))
                .or_else(|| m.get_str(cells, "subscriber"))
                .ok_or("missing subscriber number")?;
            let direction = m.get_str(cells, "direction").unwrap_or_default().to_lowercase();
            let et = if source_type == SourceType::Ipdr {
                EventType::Data
            } else if direction.contains("sms") {
                EventType::Sms
            } else {
                EventType::Call
            };
            let dur = m.get_num(cells, "duration").unwrap_or(0.0);
            (
                party,
                EntityType::Phone,
                et,
                if dur > 0.0 { Some(dur) } else { None },
            )
        }
        SourceType::Bank => {
            let account = m
                .get_str(cells, "account")
                .ok_or("missing account number")?;
            let debit = m.get_num(cells, "debit");
            let credit = m.get_num(cells, "credit");
            let amount = match (debit, credit) {
                (Some(d), Some(c)) => c - d,
                (Some(d), None) => -d,
                (None, Some(c)) => c,
                (None, None) => return Err("no amount columns populated".into()),
            };
            (account, EntityType::BankAcc, EventType::Txn, Some(amount))
        }
        SourceType::Social => {
            let handle = m.get_str(cells, "handle").ok_or("missing handle")?;
            (handle, EntityType::Handle, EventType::Post, None)
        }
    };

    let mut raw_obj = serde_json::Map::new();
    for (i, h) in headers.iter().enumerate() {
        if let Some(v) = cells.get(i) {
            raw_obj.insert(h.trim().to_string(), serde_json::Value::String(v.to_string()));
        }
    }
    if let Some(op) = operator {
        raw_obj.insert("_operator".into(), serde_json::Value::String(op.to_string()));
    }

    // Extract lat/lng from columns if present
    let location = match (m.get_num(cells, "lat"), m.get_num(cells, "lng")) {
        (Some(lat), Some(lng)) if lat != 0.0 && lng != 0.0 => {
            Some(LatLng { lat, lng })
        }
        _ => None,
    };

    Ok(Event {
        id: Uuid::new_v4(),
        case_id,
        timestamp: ts,
        source_type,
        entity_id,
        entity_type,
        event_type,
        value,
        location,
        raw: serde_json::Value::Object(raw_obj),
        notes: vec![],
    })
}

pub fn parse_ts(combined: Option<&str>, date: Option<&str>, time: Option<&str>) -> Option<DateTime<Utc>> {
    if let Some(c) = combined.or(date) {
        let t = time.unwrap_or("");
        let s = if c.contains('T') || c.len() > 10 && c.contains(['-', '/']) && t.is_empty() {
            c.to_string()
        } else if !t.is_empty() {
            format!("{c} {t}")
        } else {
            c.to_string()
        };
        let s = s.trim();

        if let Ok(n) = s.parse::<i64>() {
            return epoch_to_dt(n);
        }
        for fmt in [
            "%Y-%m-%dT%H:%M:%S%.f%z",
            "%Y-%m-%d %H:%M:%S",
            "%Y/%m/%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y/%m/%d %H:%M",
            "%d-%m-%Y %H:%M:%S",
            "%d/%m/%Y %H:%M:%S",
            "%d-%m-%Y %H:%M",
            "%d/%m/%Y %H:%M",
            "%d-%m-%Y",
            "%d/%m/%Y",
            "%Y-%m-%d",
        ] {
            if let Ok(dt) = DateTime::parse_from_str(s, fmt) {
                return Some(dt.with_timezone(&Utc));
            }
            if let Ok(nd) = NaiveDate::parse_from_str(s, fmt) {
                return Some(Utc.from_utc_datetime(&nd.and_hms_opt(0, 0, 0)?));
            }
        }
        return None;
    }
    None
}

fn epoch_to_dt(n: i64) -> Option<DateTime<Utc>> {
    match n.to_string().len() {
        10 => Utc.timestamp_opt(n, 0).single(),
        13 => Utc.timestamp_millis_opt(n).single(),
        _ => None,
    }
}
