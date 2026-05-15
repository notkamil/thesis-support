use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub fn results_dir() -> PathBuf {
    let dir = std::env::var("OTTERBRIX_BENCH_RESULTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {

            let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            here.parent()
                .and_then(Path::parent)
                .expect("CARGO_MANIFEST_DIR has two parents")
                .join("results")
                .join("raw")
        });
    create_dir_all(&dir).expect("create results directory");
    dir
}

pub fn write_samples(name: &str, samples: &[u128]) {
    let path = results_dir().join(format!("{name}.csv"));
    let file = File::create(&path).expect("create CSV file");
    let mut w = BufWriter::new(file);
    writeln!(w, "ns").expect("write header");
    for s in samples {
        writeln!(w, "{s}").expect("write sample");
    }
    w.flush().expect("flush CSV");
    eprintln!(
        "[bench] {} samples → {}",
        samples.len(),
        path.display()
    );
}
