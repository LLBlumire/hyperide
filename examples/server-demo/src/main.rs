use axum::{response::Html, routing::get, Router};
use hyperide::{
    htmx::include_htmx, hyperide, hyperscript::include_hyperscript, tailwind::include_tailwind,
};
use std::net::SocketAddr;
use tower_livereload::LiveReloadLayer;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(todos))
        .layer(LiveReloadLayer::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn base_page(content: String) -> Html<String> {
    Html(hyperide! {
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <title>"Todo App"</title>
            { include_tailwind!() }
            { include_htmx!() }
            { include_hyperscript!() }
        </head>
        <body class="min-h-screen bg-gray-200">

            { content }
        </body>
        </html>
    })
}

struct Todo {
    value: String,
    completed: bool,
}
impl Todo {
    fn get(&self, _id: usize) -> String {
        hyperide! {
            <form class="flex gap-4 bg-gray-50 p-2 rounded">
                <input type="checkbox" checked={self.completed} _="on change log 'hi'"/>
                <div
                    class="flex-grow cursor-pointer select-none"
                    _="on click click() the previous <input/>"
                >
                    {self.value.as_ref()}
                </div>
                <input type="submit" value="Edit" class="text-gray-500" />
                <input type="submit" value="Delete" class="text-gray-500" />
            </form>
        }
    }

    fn edit(&self, _id: usize) -> String {
        hyperide! {
            <form class="flex gap-4 bg-gray-50 p-2 rounded">
                <input type="checkbox" />
                <input type="text" value={self.value.as_str()} class="flex-grow" />
                <input type="submit" value="Save" class="text-gray-500" />
                <input type="submit" value="Delete" class="text-gray-500" />
            </form>
        }
    }
}

async fn todos() -> Html<String> {
    let todos: &[Todo] = &[
        Todo {
            value: "Make program work".into(),
            completed: false,
        },
        Todo {
            value: "Setup program with a really really long text entry here that just keeps on going and doesn't stop".into(),
            completed: true,
        },
    ];
    base_page(hyperide! {
        <div class="my-4 p-4 max-w-screen-sm mx-auto">
            <h1 class="text-xl font-bold mb-4">Todo App</h1>
            <ul class="flex flex-col gap-2">{
                todos
                    .iter()
                    .enumerate()
                    .map(|(id, todo)| todo.get(id))
                    .collect::<String>()
            }</ul>
        </div>
    })
}
