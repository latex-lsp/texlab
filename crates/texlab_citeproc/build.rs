use std::process::Command;

fn sh(command: &'static str) {
    let (executable, args) = if cfg!(windows) {
        ("cmd", vec!["/C", command])
    } else {
        ("sh", vec!["-c", command])
    };

    Command::new(executable)
        .args(args)
        .current_dir("script")
        .output()
        .expect(&format!("Failed to execute \"{}\"", command));
}

fn main() {
    sh("npm ci");
    sh("npm run dist");
}
