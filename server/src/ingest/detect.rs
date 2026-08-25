#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Cdr,
    Ipdr,
    Bank,
    Social,
}

impl Domain {
    pub fn as_str(self) -> &'static str {
        match self {
            Domain::Cdr => "cdr",
            Domain::Ipdr => "ipdr",
            Domain::Bank => "bank",
            Domain::Social => "social",
        }
    }
}

pub const OPERATORS: [&str; 5] = ["jio", "airtel", "bsnl", "vi", "mtnl"];

pub fn sniff_delimiter(sample: &str) -> u8 {
    let mut best = b',';
    let mut best_count = 0usize;
    for candidate in [b',', b';', b'\t', b'|'] {
        let count: usize = sample
            .lines()
            .take(10)
            .map(|line| count_unquoted(line, candidate))
            .sum();
        if count > best_count {
            best_count = count;
            best = candidate;
        }
    }
    best
}

fn count_unquoted(line: &str, delim: u8) -> usize {
    let mut in_quotes = false;
    let mut n = 0;
    for b in line.bytes() {
        match b {
            b'"' => in_quotes = !in_quotes,
            _ if b == delim && !in_quotes => n += 1,
            _ => {}
        }
    }
    n
}

pub fn normalize_header(h: &str) -> String {
    h.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

type Aliases = &'static [(&'static str, &'static str)];

const CDR_ALIASES: Aliases = &[
    ("callingnumber", "a_party"),
    ("callingnum", "a_party"),
    ("anumber", "a_party"),
    ("aparty", "a_party"),
    ("callernumber", "a_party"),
    ("caller", "a_party"),
    ("msisdn", "a_party"),
    ("originatingnumber", "a_party"),
    ("callednumber", "b_party"),
    ("callednum", "b_party"),
    ("bnumber", "b_party"),
    ("bparty", "b_party"),
    ("callee", "b_party"),
    ("terminatingnumber", "b_party"),
    ("datetime", "ts"),
    ("date", "date"),
    ("time", "time"),
    ("starttime", "ts"),
    ("callstart", "ts"),
    ("durationsec", "duration"),
    ("durationsecs", "duration"),
    ("callduration", "duration"),
    ("duration", "duration"),
    ("calltype", "direction"),
    ("type", "direction"),
    ("direction", "direction"),
    ("towersid", "cell_id"),
    ("cellid", "cell_id"),
    ("cgi", "cell_id"),
    ("cell", "cell_id"),
    ("firstcell", "cell_id"),
    ("imei", "imei"),
    ("imsi", "imsi"),
    ("operatorname", "operator"),
    ("operator", "operator"),
    ("circle", "circle"),
];

const BANK_ALIASES: Aliases = &[
    (" valuedate", "date"),
    ("valuedate", "date"),
    ("txndate", "date"),
    ("transactiondate", "date"),
    ("date", "date"),
    ("narration", "description"),
    ("description", "description"),
    ("particulars", "description"),
    ("remarks", "description"),
    ("referencenumber", "reference"),
    ("reference", "reference"),
    ("refno", "reference"),
    ("ref", "reference"),
    ("utr", "reference"),
    ("chqno", "reference"),
    ("withdrawalamt", "debit"),
    ("withdrawalamount", "debit"),
    ("withdrawal", "debit"),
    ("debit", "debit"),
    ("depositamt", "credit"),
    ("depositamount", "credit"),
    ("deposit", "credit"),
    ("credit", "credit"),
    ("balance", "balance"),
    ("accountno", "account"),
    ("accountnumber", "account"),
    ("account", "account"),
    ("acct", "account"),
];

const IPDR_ALIASES: Aliases = &[
    ("sourceip", "ip"),
    ("framedipaddress", "ip"),
    ("framedip", "ip"),
    ("ipaddress", "ip"),
    ("ip", "ip"),
    ("customerip", "ip"),
    ("starttime", "ts"),
    ("logintime", "ts"),
    ("sessionstart", "ts"),
    ("endtime", "end_ts"),
    ("logouttime", "end_ts"),
    ("sessionend", "end_ts"),
    ("urlvisited", "url"),
    ("url", "url"),
    ("destinationhost", "host"),
    ("host", "host"),
    ("domain", "host"),
    ("port", "port"),
    ("username", "subscriber"),
    ("subscriberid", "subscriber"),
    ("phonenumber", "phone"),
    ("msisdn", "phone"),
];

const SOCIAL_ALIASES: Aliases = &[
    ("handle", "handle"),
    ("username", "handle"),
    ("user", "handle"),
    ("screenname", "handle"),
    ("account", "handle"),
    ("timestamp", "ts"),
    ("datetime", "ts"),
    ("createdat", "ts"),
    ("postdate", "ts"),
    ("date", "date"),
    ("time", "time"),
    ("content", "content"),
    ("text", "content"),
    ("body", "content"),
    ("post", "content"),
    ("message", "content"),
    ("url", "url"),
    ("link", "url"),
];

pub struct Fingerprint {
    pub domain: Domain,
    pub score: usize,
}

pub fn detect_domain(headers: &[String]) -> Fingerprint {
    let norm: Vec<String> = headers.iter().map(|h| normalize_header(h)).collect();
    let hit = |aliases: Aliases| -> usize {
        norm.iter()
            .filter(|h| aliases.iter().any(|(a, _)| a == h))
            .count()
    };
    let scores = [
        (Domain::Cdr, hit(CDR_ALIASES)),
        (Domain::Ipdr, hit(IPDR_ALIASES)),
        (Domain::Bank, hit(BANK_ALIASES)),
        (Domain::Social, hit(SOCIAL_ALIASES)),
    ];
    let (domain, score) = scores
        .iter()
        .copied()
        .max_by_key(|(_, s)| *s)
        .unwrap_or((Domain::Cdr, 0));
    Fingerprint { domain, score }
}

pub fn build_column_map(headers: &[String]) -> Vec<(&'static str, usize)> {
    let aliases: Aliases = match detect_domain(headers).domain {
        Domain::Cdr => CDR_ALIASES,
        Domain::Bank => BANK_ALIASES,
        Domain::Ipdr => IPDR_ALIASES,
        Domain::Social => SOCIAL_ALIASES,
    };
    let mut map = Vec::new();
    for (idx, h) in headers.iter().enumerate() {
        let nh = normalize_header(h);
        if let Some(found) = aliases.iter().find(|(a, _)| *a == nh) {
            map.push((found.1, idx));
        }
    }
    map
}

pub fn detect_operator(text: &str) -> Option<&'static str> {
    let lower = text.to_lowercase();
    OPERATORS.iter().copied().find(|op| lower.contains(op))
}
