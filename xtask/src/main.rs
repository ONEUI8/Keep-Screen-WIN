use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

/// This is a simple build script for post-compilation tasks.
fn main() {
    // Simple argument parsing to find a --target flag.
    let args: Vec<String> = env::args().collect();
    let mut target: Option<&str> = None;
    if let Some(index) = args.iter().position(|r| r == "--target") {
        if let Some(t) = args.get(index + 1) {
            target = Some(t);
        }
    }

    // 1. Compile the main project in release mode, passing the target if it exists.
    println!("Building project...");
    let mut command = Command::new("cargo");
    command.arg("build").arg("--release");
    if let Some(target_str) = target {
        println!("Cross-compiling for target: {}", target_str);
        command.arg("--target").arg(target_str);
    }

    let status = command.status().expect("Failed to run cargo build");

    if !status.success() {
        eprintln!("Build failed");
        std::process::exit(1);
    }

    // 2. Locate and rename the output executable, accounting for the target directory.
    let target_dir = if let Some(t) = target {
        Path::new("target").join(t).join("release")
    } else {
        Path::new("target").join("release")
    };

    let exe_src = target_dir.join("keep_screen.exe");
    let exe_dst = target_dir.join("Keep Screen.exe");

    if exe_src.exists() {
        // To prevent an error, remove the destination file if it already exists.
        let _ = fs::remove_file(&exe_dst);
        fs::rename(&exe_src, &exe_dst).expect("Failed to rename exe");
        println!("Renamed to: {}", exe_dst.display());
    } else {
        eprintln!("Compiled exe not found: {}", exe_src.display());
        std::process::exit(1);
    }
}
