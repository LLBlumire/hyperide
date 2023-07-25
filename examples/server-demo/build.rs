use std::path::Path;

use dotenv_codegen::dotenv;

fn main() {
    dotenv::dotenv().ok();

    let database_url = dotenv!("DATABASE_URL");

    println!("DATABASE_URL: {}", database_url);

    hyperide::tailwind::bootstrap(
        Path::new("./tailwind.config.js"),
        Path::new("./src/tailwind.css"),
    );
}
