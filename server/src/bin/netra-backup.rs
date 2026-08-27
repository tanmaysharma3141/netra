use std::io::Write;
use zip::write::SimpleFileOptions;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let output = args.get(1).map(|s| s.as_str()).unwrap_or("netra-backup.zip");

    println!("NETRA Backup Tool v{}", env!("CARGO_PKG_VERSION"));
    println!("Output: {output}");

    // Copy database
    let db_path = "netra.db";
    if !std::path::Path::new(db_path).exists() {
        eprintln!("Error: {db_path} not found. Is the server installed here?");
        std::process::exit(1);
    }

    let zip_file = std::fs::File::create(output).expect("create output file");
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Include database
    print!("Adding database... ");
    let db_bytes = std::fs::read(db_path).expect("read database");
    zip.start_file("netra.db", options).expect("start db file");
    zip.write_all(&db_bytes).expect("write db");
    println!("{} MB", db_bytes.len() / (1024 * 1024));

    // Include tower DB if present
    let tower_path = "data/towers.db";
    if std::path::Path::new(tower_path).exists() {
        print!("Adding tower database... ");
        let tower_bytes = std::fs::read(tower_path).expect("read tower db");
        zip.start_file("towers.db", options).expect("start tower file");
        zip.write_all(&tower_bytes).expect("write tower db");
        println!("{} KB", tower_bytes.len() / 1024);
    }

    // Include upload files
    let upload_dir = "data/uploads";
    if std::path::Path::new(upload_dir).is_dir() {
        print!("Adding uploaded files... ");
        let mut count = 0;
        for entry in std::fs::read_dir(upload_dir).expect("read uploads") {
            let entry = entry.expect("dir entry");
            if entry.file_type().expect("file type").is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                let bytes = std::fs::read(entry.path()).expect("read file");
                zip.start_file(&format!("uploads/{name}"), options).expect("start upload");
                zip.write_all(&bytes).expect("write upload");
                count += 1;
            }
        }
        println!("{count} files");
    }

    zip.finish().expect("finalize zip");

    let size = std::fs::metadata(output).expect("stat output").len();
    println!("\nBackup complete: {output} ({:.1} MB)", size as f64 / (1024.0 * 1024.0));
}
