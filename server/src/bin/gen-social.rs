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
        eprintln!("usage: gen-social <out.csv> <rows>");
        std::process::exit(1);
    }
    let out_path = args[1].clone();
    let rows: u64 = args[2].parse().unwrap_or(10_000);

    let mut rng = Lcg(0x534F_4349_414C_2026);

    // Handles that cross-reference with phone entities (same names used in bank data)
    // These will trigger cross-domain probabilistic matching
    let suspect_handles = [
        ("@rajk98", "+919800000001"),       // Rajesh Kumar's phone
        ("@priya_s", "+919900000003"),      // Priya Sharma's phone
        ("@amit_patel", "+917000000007"),   // Amit Patel's phone
        ("@vikram_123", "+918000000004"),   // Vikram Singh's phone
        ("@sunita_d", "+918500000005"),     // Sunita Devi's phone
        ("@mohd_khan", "+919800000006"),    // Mohammed Khan's phone
        ("@deepak_g", "+917000000007"),     // Deepak Gupta's phone
        ("@anita_v", "+918000000008"),      // Anita Verma's phone
    ];

    // Normal (non-suspect) handles
    let normal_handles = [
        "@chd_foodie", "@punjabi_kudi", "@travel_addict",
        "@tech_geek42", "@fitness_first", "@bookworm_anna",
        "@music_lover99", "@photo_studio", "@local_news_chd",
        "@student_vibes", "@shop_local", "@cricket_fan",
        "@chef_ramya", "@dr_patel", "@lawyer_singh",
    ];

    let platforms = ["twitter", "telegram", "instagram", "facebook"];
    let platform_weights = [40, 30, 20, 10]; // percentage

    let contents = [
        "Good morning everyone!",
        "Just landed in Chandigarh, beautiful city",
        "Great meeting with the team today",
        "New project starting next week",
        "Had an amazing dinner at Sector 17",
        "Weekend plans? Anyone up for a trip?",
        "Traffic is terrible near IT Park today",
        "Just finished a 10km run, feeling great",
        "Check out this new cafe in Mohali",
        "Working late tonight, deadline approaching",
        "Happy birthday to my best friend!",
        "This weather is perfect for a bike ride",
        "Finally got my new phone, loving it",
        "Anyone know a good mechanic in Ludhiana?",
        "Protest march this weekend, be prepared",
        "DM for business inquiries",
        "Latest update on the case - check DM",
        "Meeting at Sector 17 tomorrow, 3pm",
        "Transfer done, check your account",
        "Urgent: need to talk, call me",
        "Sent the documents, please review",
        "Payment confirmed, goods dispatched",
        "Location shared, see you there",
        "Code word: alpha bravo charlie",
        "Don't mention this to anyone",
    ];

    let base_ts = 1_756_000_000u64; // ~Aug 24, 2025
    let span = 30 * 86_400;

    let mut f = std::io::BufWriter::new(std::fs::File::create(&out_path).expect("create out"));
    writeln!(
        f,
        "Handle,Timestamp,Content,Platform,URL"
    )
    .unwrap();

    for i in 0..rows {
        let ts = base_ts + (i * span / rows.max(1)) + rng.range(86_400);
        let dt = epoch_fmt(ts);

        // Pick handle: 30% suspect, 70% normal
        let (handle, phone) = if rng.range(10) < 3 {
            suspect_handles[rng.range(suspect_handles.len() as u64) as usize]
        } else {
            (normal_handles[rng.range(normal_handles.len() as u64) as usize], "")
        };

        // Pick platform with weights
        let platform = {
            let r = rng.range(100);
            if r < platform_weights[0] { platforms[0] }
            else if r < platform_weights[0] + platform_weights[1] { platforms[1] }
            else if r < platform_weights[0] + platform_weights[1] + platform_weights[2] { platforms[2] }
            else { platforms[3] }
        };

        let content = contents[rng.range(contents.len() as u64) as usize];

        // Generate URL (some posts have links)
        let url = if rng.range(5) == 0 {
            match platform {
                "twitter" => format!("https://twitter.com/{}/status/{}", &handle[1..], 1000000 + i),
                "telegram" => format!("https://t.me/{}/{}", &handle[1..], i),
                "instagram" => format!("https://instagram.com/p/{}", format!("{:x}", i * 31)),
                _ => format!("https://facebook.com/posts/{}", 1000000 + i),
            }
        } else {
            String::new()
        };

        // If suspect, sometimes reference phone or financial activity in content
        let final_content = if !phone.is_empty() && rng.range(8) == 0 {
            format!("{} [ref: {}]", content, phone)
        } else {
            content.to_string()
        };

        writeln!(
            f,
            "{handle},{dt},{final_content},{platform},{url}"
        )
        .unwrap();
    }

    eprintln!("wrote {rows} social rows to {out_path}");
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
