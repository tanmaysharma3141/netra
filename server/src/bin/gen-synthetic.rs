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
        eprintln!("usage: gen-synthetic <out.csv> <rows> [cdr|bank]");
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

fn gen_cdr(out_path: &str, rows: u64) {
    let mut rng = Lcg(0x4E57_4554_5241_2026);
    let mut numbers: Vec<String> = (0..60)
        .map(|i| format!("+9198{:07}", i * 137 + rng.range(999) as usize))
        .collect();
    numbers.sort();
    numbers.dedup();

    let imeis: Vec<String> = (0..55)
        .map(|i| format!("35{i:013}"))
        .collect();
    let hot_imei = "354809104512345".to_string();

    let suspects: Vec<usize> = (0..6).map(|i| (i * 7) % numbers.len()).collect();
    let mut personal: HashMap<usize, String> = HashMap::new();
    for (i, n) in numbers.iter().enumerate() {
        personal.insert(i, imeis[i % imeis.len()].clone());
        let _ = n;
    }

    let base_ts = 1_756_000_000u64;
    let span = 30 * 86_400;

    let mut f = std::io::BufWriter::new(std::fs::File::create(out_path).expect("create out"));
    writeln!(
        f,
        "A Number,B Number,Date Time,Duration (sec),Call Type,Tower ID (First),Last Tower ID,IMEI,Operator Name"
    )
    .unwrap();

    for i in 0..rows {
        let a_idx = rng.range(numbers.len() as u64) as usize;
        let b_idx = rng.range(numbers.len() as u64) as usize;
        let a = &numbers[a_idx];
        let b = &numbers[b_idx];
        let ts = base_ts + (i * span / rows.max(1)) + rng.range(60);
        let dt = epoch_fmt(ts);
        let dur = if rng.range(10) == 0 { rng.range(1800) } else { rng.range(120) };
        let call_type = match rng.range(20) {
            0..=2 => "SMS",
            3..=9 => "IN",
            _ => "OUT",
        };
        let tower = format!("JIO-PB-{}", rng.range(400));
        let last_tower = format!("JIO-PB-{}", rng.range(400));
        let imei = if suspects.contains(&a_idx) && rng.range(10) < 7 {
            &hot_imei
        } else {
            &personal[&a_idx]
        };
        writeln!(
            f,
            "{a},{b},{dt},{dur},{call_type},{tower},{last_tower},{imei},JIO"
        )
        .unwrap();
    }
    eprintln!("wrote {rows} CDR rows to {out_path}");
}

fn gen_bank(out_path: &str, rows: u64) {
    let mut rng = Lcg(0x4841_5741_4C41_2026);
    let accounts: Vec<String> = (0..20)
        .map(|i| format!("XXXX{:04}", 1000 + i))
        .collect();
    let narrations = [
        "IMPS/P2A", "UPI/CR", "NEFT-CR", "ATM WDL", "POS/DEBIT", "IB FUNDS TRF",
    ];
    let base_ts = 1_756_000_000u64;
    let span = 30 * 86_400;

    let mut f = std::io::BufWriter::new(std::fs::File::create(out_path).expect("create out"));
    writeln!(
        f,
        "Account No,Value Date,Narration,Ref No/Cheque No,Withdrawal Amt.,Deposit Amt.,Balance(*)"
    )
    .unwrap();

    for i in 0..rows {
        let acct = &accounts[rng.range(accounts.len() as u64) as usize];
        let ts = base_ts + (i * span / rows.max(1)) + rng.range(3600);
        let dt = epoch_fmt(ts);
        let narration = narrations[rng.range(narrations.len() as u64) as usize];
        let rno = format!("IMPS-{}/{}", ts, rng.range(99_999));
        let hawala = rng.range(15) == 0;
        let (wdl, dep): (f64, f64) = if hawala {
            if rng.range(2) == 0 {
                ((rng.range(9000) + 100) as f64, 0.0)
            } else {
                (0.0, (rng.range(8000) + 200) as f64)
            }
        } else if rng.range(2) == 0 {
            ((rng.range(50_000) + 500) as f64, 0.0)
        } else {
            (0.0, (rng.range(60_000) + 500) as f64)
        };
        let bal = rng.range(900_000) as f64 + 10_000.0;
        writeln!(
            f,
            "{acct},{dt},{narration},{rno},{wdl:.2},{dep:.2},{bal:.2}"
        )
        .unwrap();
    }
    eprintln!("wrote {rows} bank rows to {out_path}");
}

fn epoch_fmt(ts: u64) -> String {
    // civil-from-days algorithm (Howard Hinnant)
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
