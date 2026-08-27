use std::collections::HashMap;
use std::io::Write as _;

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 11
    }
    fn range(&mut self, n: u64) -> u64 {
        self.next() % n
    }

}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: gen-synthetic <out.csv> <rows> [cdr|bank] [--scenario full-scene]");
        std::process::exit(1);
    }
    let out_path = args[1].clone();
    let rows: u64 = args[2].parse().unwrap_or(100_000);
    let domain = args.get(3).map(String::as_str).unwrap_or("cdr");

    match domain {
        "bank" => gen_bank(&out_path, rows),
        _ => gen_cdr(&out_path, rows),
    }
}

// ─── Tower database (Chandigarh / Punjab region) ────────────────────────

struct Tower {
    name: &'static str,
    lat: f64,
    lng: f64,
}

const TOWERS: &[Tower] = &[
    Tower { name: "JIO-CHD-001", lat: 30.7333, lng: 76.7794 },  // Chandigarh Sector 17
    Tower { name: "JIO-CHD-002", lat: 30.7420, lng: 76.7880 },  // Chandigarh IT Park
    Tower { name: "JIO-CHD-003", lat: 30.7510, lng: 76.8010 },  // Chandigarh Sector 34
    Tower { name: "JIO-CHD-004", lat: 30.7190, lng: 76.7620 },  // Mohali Phase 7
    Tower { name: "JIO-CHD-005", lat: 30.7050, lng: 76.7450 },  // Mohali Phase 5
    Tower { name: "JIO-CHD-006", lat: 30.6940, lng: 76.7270 },  // Zirakpur
    Tower { name: "Airtel-LDH-001", lat: 30.9120, lng: 75.8530 }, // Ludhiana Civil Lines
    Tower { name: "Airtel-LDH-002", lat: 30.9250, lng: 75.8640 }, // Ludhiana Model Town
    Tower { name: "JIO-LDH-001", lat: 30.8890, lng: 75.8210 },  // Ludhiana PAU
    Tower { name: "BSNL-JAL-001", lat: 30.9530, lng: 75.5730 },  // Jalandhar
    Tower { name: "JIO-PHR-001", lat: 31.0150, lng: 75.3480 },  // Phagwara
    Tower { name: "VI-PTA-001", lat: 30.3750, lng: 76.7820 },   // Patiala
    Tower { name: "Airtel-AMB-001", lat: 30.5200, lng: 76.6600 }, // Ambala
    Tower { name: "BSNL-HSP-001", lat: 31.1050, lng: 75.7040 }, // Hoshiarpur
    Tower { name: "JIO-PTK-001", lat: 31.3260, lng: 75.5760 },  // Pathankot
    // Meeting point tower (for co-location events)
    Tower { name: "JIO-CHD-MEET", lat: 30.7380, lng: 76.7830 }, // Sector 17/18 border
    // Coordinated silence home tower
    Tower { name: "JIO-CHD-HOME", lat: 30.7280, lng: 76.7750 }, // Residential area
];

// ─── Operator pool ──────────────────────────────────────────────────────

// ─── CDR generator ──────────────────────────────────────────────────────

fn gen_cdr(out_path: &str, rows: u64) {
    let mut rng = Lcg(0x4E57_4554_5241_2026);

    // Generate phone numbers
    let mut numbers: Vec<String> = (0..80)
        .map(|i| {
            let prefix = match i % 5 {
                0 => 98, 1 => 99, 2 => 70, 3 => 80, _ => 85,
            };
            format!("+91{}{:07}", prefix, i * 137 + rng.range(999) as usize)
        })
        .collect();
    numbers.sort();
    numbers.dedup();

    // Assign operators (50% JIO, 25% Airtel, 15% BSNL, 10% VI)
    let phone_ops: Vec<&str> = numbers.iter().map(|_| {
        let r = rng.range(100);
        if r < 50 { "JIO" }
        else if r < 75 { "Airtel" }
        else if r < 90 { "BSNL" }
        else { "VI" }
    }).collect();

    // IMEIs — each phone gets a personal IMEI
    let imeis: Vec<String> = numbers.iter().enumerate()
        .map(|(i, _)| format!("35{:013}", i * 7 + 4809100000000))
        .collect();

    // Hot IMEIs for the ring
    let hot_imei_1 = "354809104512345".to_string();
    let hot_imei_2 = "354809104519999".to_string();

    // Suspect indices (8 phones for IMEI ring)
    let suspects: Vec<usize> = (0..8).map(|i| i * 5 % numbers.len()).collect();

    // Silence suspects (5 phones go dark on day 20)
    let silence_suspects: Vec<usize> = (0..5).map(|i| i * 9 % numbers.len()).collect();

    // Co-location suspects (3 phones meet at same tower)
    let coloc_suspects: Vec<usize> = suspects[..3.min(suspects.len())].to_vec();

    let base_ts = 1_756_000_000u64; // ~Aug 24, 2025

    let mut f = std::io::BufWriter::new(std::fs::File::create(out_path).expect("create out"));
    writeln!(
        f,
        "A Number,B Number,Date Time,Duration (sec),Call Type,Tower ID,IMEI,IMSI,Operator Name,Lat,Lng"
    )
    .unwrap();

    let mut rows_written = 0u64;

    for i in 0..rows {
        let a_idx = rng.range(numbers.len() as u64) as usize;
        let b_idx = rng.range(numbers.len() as u64) as usize;
        let a = &numbers[a_idx];
        let b = &numbers[b_idx];

        // Calculate day of this event (0-29)
        let day = (i * 30 / rows.max(1)) as u64;
        let day_start = base_ts + day * 86_400;
        let ts = day_start + rng.range(86_400);

        // Check if silence suspect should have stopped (day >= 20)
        if silence_suspects.contains(&a_idx) && day >= 20 {
            continue; // Phone went dark
        }

        let dt = epoch_fmt(ts);

        // Duration: 70% short calls, 20% medium, 10% long
        let dur = match rng.range(10) {
            0..=6 => rng.range(120),
            7..=8 => rng.range(600) + 60,
            _ => rng.range(1800) + 300,
        };

        // Call type
        let call_type = match rng.range(20) {
            0..=2 => "SMS",
            3..=9 => "IN",
            _ => "OUT",
        };

        // Tower selection
        let tower_idx = if coloc_suspects.contains(&a_idx) && day >= 18 && day <= 20 {
            // Co-location: suspects meet at meeting tower
            16 // MEET tower
        } else if silence_suspects.contains(&a_idx) {
            // Before silence: use home tower more often
            if rng.range(3) == 0 { 16 } else { rng.range(16) as usize }
        } else {
            rng.range(16) as usize
        };
        let tower = &TOWERS[tower_idx];

        // IMEI selection
        let imei = if suspects.contains(&a_idx) && rng.range(10) < 7 {
            if a_idx % 2 == 0 { &hot_imei_1 } else { &hot_imei_2 }
        } else {
            &imeis[a_idx]
        };

        // IMSI: deterministic from phone number hash
        let imsi = format!("404{:012}", (a_idx as u64 * 7919 + 1000) % 1_000_000_000_000);

        let op = phone_ops[a_idx];

        writeln!(
            f,
            "{a},{b},{dt},{dur},{call_type},{},{imei},{imsi},{op},{:.4},{:.4}",
            tower.name, tower.lat, tower.lng
        )
        .unwrap();
        rows_written += 1;
    }

    eprintln!("wrote {rows_written} CDR rows to {out_path} ({rows} requested, {} skipped for silence)", rows - rows_written);
}

// ─── Bank generator ─────────────────────────────────────────────────────

const ACCOUNT_NAMES: &[(&str, &str)] = &[
    // (account_number, holder_name)
    ("XXXX1001", "Rajesh Kumar"),
    ("XXXX1002", "Priya Sharma"),
    ("XXXX1003", "Amit Patel"),
    ("XXXX1004", "Vikram Singh"),
    ("XXXX1005", "Sunita Devi"),
    ("XXXX1006", "Mohammed Khan"),
    ("XXXX1007", "Deepak Gupta"),
    ("XXXX1008", "Anita Verma"),
    ("XXXX1009", "Ravi Shankar"),
    ("XXXX1010", "Meena Kumari"),
    ("XXXX1011", "Rajesh Kumar Singh"),  // variant of Rajesh Kumar
    ("XXXX1012", "Priya Sharma Devi"),    // variant of Priya Sharma
    ("XXXX1013", "Amitbhai Patel"),      // variant of Amit Patel
    ("XXXX1014", "Vikram Signh"),        // typo of Vikram Singh
    ("XXXX1015", "Mohammad Khan"),       // variant of Mohammed Khan
    ("XXXX1016", "Deepak Grupta"),       // typo of Deepak Gupta
    ("XXXX1017", "Ravi Shanker"),        // variant of Ravi Shankar
    ("XXXX1018", "Neha Aggarwal"),
    ("XXXX1019", "Sanjay Mishra"),
    ("XXXX1020", "Pooja Choudhary"),
];

// Phone numbers for cross-domain linking (same numbers appear in CDR)
const CROSS_DOMAIN_PHONES: &[(&str, &str)] = &[
    ("XXXX1001", "+919800000001"),  // Rajesh Kumar's phone
    ("XXXX1003", "+919900000003"),  // Amit Patel's phone
    ("XXXX1007", "+917000000007"),  // Deepak Gupta's phone
];

const NARRATIONS: &[&str] = &[
    "IMPS/P2A", "UPI/CR", "NEFT-CR", "ATM WDL", "POS/DEBIT",
    "IB FUNDS TRF", "IMPS/P2P", "UPI/DR", "NEFT-DR", "RTGS",
];

fn gen_bank(out_path: &str, rows: u64) {
    let mut rng = Lcg(0x4841_5741_4C41_2026);

    let base_ts = 1_756_000_000u64;

    // Hawala accounts (3 accounts with structured deposits)
    let hawala_accounts: Vec<usize> = vec![0, 5, 7]; // Rajesh Kumar, Mohammed Khan, Anita Verma
    // Rapid burst accounts (2 accounts with big transfers)
    let rapid_accounts: Vec<usize> = vec![2, 6]; // Amit Patel, Deepak Gupta
    let mut balances: HashMap<usize, f64> = HashMap::new();
    for i in 0..ACCOUNT_NAMES.len() {
        balances.insert(i, rng.range(500_000) as f64 + 50_000.0);
    }

    let mut f = std::io::BufWriter::new(std::fs::File::create(out_path).expect("create out"));
    writeln!(
        f,
        "Account No,Value Date,Narration,Ref No,Withdrawal Amt.,Deposit Amt.,Balance(*),Counterparty,Account Name"
    )
    .unwrap();

    for i in 0..rows {
        let acct_idx = rng.range(ACCOUNT_NAMES.len() as u64) as usize;
        let (acct_no, acct_name) = ACCOUNT_NAMES[acct_idx];

        let day = (i * 30 / rows.max(1)) as u64;
        let ts = base_ts + day * 86_400 + rng.range(86_400);
        let dt = epoch_fmt(ts);

        let narration = NARRATIONS[rng.range(NARRATIONS.len() as u64) as usize];
        let rno = format!("TXN-{}/{}", ts, rng.range(99_999));

        // Determine if this is a hawala, rapid, or normal transaction
        let (wdl, dep): (f64, f64) = if hawala_accounts.contains(&acct_idx) && day >= 10 && day <= 15 {
            // Structured deposits: small amounts, multiple
            let amount = (rng.range(7000) + 2000) as f64; // 2000-9000
            (0.0, amount)
        } else if rapid_accounts.contains(&acct_idx) && day >= 15 && day <= 18 {
            // Rapid burst: large transfers
            let amount = (rng.range(70_000) + 80_000) as f64; // 80k-150k
            (0.0, amount)
        } else if rng.range(2) == 0 {
            ((rng.range(50_000) + 500) as f64, 0.0)
        } else {
            (0.0, (rng.range(60_000) + 500) as f64)
        };

        let bal = balances.entry(acct_idx).and_modify(|b| {
            *b += dep - wdl;
        }).or_insert(100_000.0);

        // Counterparty (another account or external)
        let counterparty = if rng.range(3) == 0 {
            let cp_idx = rng.range(ACCOUNT_NAMES.len() as u64) as usize;
            ACCOUNT_NAMES[cp_idx].0
        } else {
            "EXT"
        };

        // Phone cross-ref for certain accounts
        let phone_ref = CROSS_DOMAIN_PHONES.iter()
            .find(|(a, _)| *a == acct_no)
            .map(|(_, p)| *p)
            .unwrap_or("");

        writeln!(
            f,
            "{acct_no},{dt},{narration},{rno},{wdl:.2},{dep:.2},{bal:.2},{counterparty},{acct_name}{phone_suffix}",
            phone_suffix = if !phone_ref.is_empty() { format!(" [{}]", phone_ref) } else { String::new() }
        )
        .unwrap();
    }

    eprintln!("wrote {rows} bank rows to {out_path}");
}

// ─── Timestamp formatting ───────────────────────────────────────────────

fn epoch_fmt(ts: u64) -> String {
    let days = (ts / 86_400) as i64;
    let secs = ts % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    let hh = secs / 3600;
    let mm = (secs % 3600) / 60;
    let ss = secs % 60;
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02}")
}
