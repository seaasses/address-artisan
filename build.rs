use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let kernel_dir = Path::new("src/opencl");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("combined_kernels.cl");

    let mut combined_file = fs::File::create(&dest_path).unwrap();

    println!("cargo:rerun-if-changed=src/opencl");

    process_directory(&kernel_dir, &mut combined_file);
}

fn process_directory(dir: &Path, combined_file: &mut fs::File) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    process_directory(&path, combined_file);
                } else if path.extension().map_or(false, |ext| ext == "cl") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        writeln!(combined_file, "// Kernel from file: {:?}", path).unwrap();
                        writeln!(combined_file, "{}", content).unwrap();
                    }
                }
            }
        }
    }
}
