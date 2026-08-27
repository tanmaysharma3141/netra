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
        eprintln!("usage: gen-ipdr <out.csv> <rows>");
        std::process::exit(1);
    }
    let out_path = args[1].clone();
    let rows: u64 = args[2].parse().unwrap_or(20_000);

    let mut rng = Lcg(0x4950_4452_4120_2026);

    // IP addresses assigned to phone entities (cross-domain linking)
    let phone_ips: Vec<(&str, &str)> = vec![
        ("+919800000001", "10.0.1.101"),  // Rajesh Kumar
        ("+919900000003", "10.0.1.103"),  // Priya Sharma
        ("+917000000007", "10.0.1.107"),  // Deepak Gupta
        ("+918000000004", "10.0.1.104"),  // Vikram Singh
        ("+918500000005", "10.0.1.105"),  // Sunita Devi
    ];

    // Normal IPs
    let normal_ips: Vec<&str> = (0..35)
        .map(|i| {
            // Leak the string into static lifetime
            Box::leak(format!("10.0.{}.{}", 2 + i / 256, i % 256).into_boxed_str()) as &str
        })
        .collect();

    // Benign URLs
    let benign_urls = [
        ("google.com", 443),
        ("youtube.com", 443),
        ("whatsapp.com", 443),
        ("facebook.com", 443),
        ("instagram.com", 443),
        ("twitter.com", 443),
        ("linkedin.com", 443),
        ("wikipedia.org", 443),
        ("amazon.in", 443),
        ("flipkart.com", 443),
    ];

    // Suspicious URLs (for investigative interest)
    let suspicious_urls = [
        ("pastebin.com", 443),
        ("bit.ly", 443),
        ("mega.nz", 443),
        ("protonmail.com", 443),
        ("tormail.org", 80),
        ("deepwebforums.onion", 80),
        ("cryptocurrency-exchange.io", 443),
        ("hawala-network.net", 443),
    ];

    let base_ts = 1_756_000_000u64;
    let span = 30 * 86_400;

    let mut f = std::io::BufWriter::new(std::fs::File::create(&out_path).expect("create out"));
    writeln!(
        f,
        "Source IP,Start Time,End Time,URL Visited,Destination Host,Port,Username,Phone Number"
    )
    .unwrap();

    for i in 0..rows {
        let ts = base_ts + (i * span / rows.max(1)) + rng.range(86_400);
        let dt = epoch_fmt(ts);

        // Session duration: 5min - 2 hours
        let duration = rng.range(7200) + 300;
        let end_ts = ts + duration;
        let end_dt = epoch_fmt(end_ts);

        // Pick IP
        let (ip, phone) = if rng.range(10) < 3 {
            let (p, ip_addr) = phone_ips[rng.range(phone_ips.len() as u64) as usize];
            (ip_addr, p)
        } else {
            (normal_ips[rng.range(normal_ips.len() as u64) as usize], "")
        };

        // Pick URL: 85% benign, 15% suspicious
        let (host, port) = if rng.range(100) < 15 {
            suspicious_urls[rng.range(suspicious_urls.len() as u64) as usize]
        } else {
            benign_urls[rng.range(benign_urls.len() as u64) as usize]
        };

        // Username (sometimes matches phone number)
        let username = if !phone.is_empty() && rng.range(3) == 0 {
            phone
        } else if rng.range(5) == 0 {
            "anonymous"
        } else {
            ""
        };

        writeln!(
            f,
            "{ip},{dt},{end_dt},{host},{host},{port},{username},{phone}"
        )
        .unwrap();
    }

    eprintln!("wrote {rows} IPDR rows to {out_path}");
}

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
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}
