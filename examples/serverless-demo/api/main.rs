use axum::{extract::Path, response::Html, routing::get, Router};
use hyperide::{hyperide, tailwind::include_tailwind};
use vercel_runtime::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = Router::new()
        .route("/", get(root))
        .route("/hello/:name", get(greet));
    hyperide::vercel::run(app).await
}

async fn root() -> Html<String> {
    Html(greeting("world"))
}

async fn greet(Path((name,)): Path<(String,)>) -> Html<String> {
    Html(greeting(&name))
}

fn greeting(name: &str) -> String {
    hyperide! {
        <!DOCTYPE html>
        <html lang="en">
        <head>
            {include_tailwind!()}
        </head>
        <body>
            <p class="text-xl m-5">{"Hello, "}<strong>{name}</strong>{"!"}</p>
            <div data-foo="bar"></div>
        </body>
        </html>
    }
}
