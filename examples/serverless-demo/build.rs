use std::path::Path;

fn main() {
    hyperide::tailwind::bootstrap(
        Path::new("./tailwind.config.js"),
        Path::new("./tailwind.in.css"),
    );
}
