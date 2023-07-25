#[macro_use]
extern crate dotenv_codegen;

use axum::extract::State;
use axum::{response::Html, routing::get, Router};
use hyperide::{
    htmx::include_htmx, hyperide, hyperscript::include_hyperscript, tailwind::include_tailwind,
};
use sqlx::SqlitePool;
use std::env;
use std::net::SocketAddr;
use tower_livereload::LiveReloadLayer;

#[tokio::main]
async fn main() {
    let db = SqlitePool::connect(dotenv!("DATABASE_URL"))
        .await
        .expect("Failed to connect to SQLite");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let app = Router::new()
        .route("/", get(todos_homepage))
        .with_state(db)
        .layer(LiveReloadLayer::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn todos_homepage(State(db): State<SqlitePool>) -> Html<String> {
    let content = todos(db).await;

    Html(layout(content))
}

fn layout(content: String) -> String {
    hyperide! {
        <!DOCTYPE html>
          <html lang="en">
          <head>
              <title>"Todo App"</title>
              { include_tailwind!() }
              { include_htmx!() }
              { include_hyperscript!() }
          </head>
          <body class="min-h-screen bg-gray-200">
            {content}
          </body>
        </html>
    }
}

struct TodoProps {
    id: i64,
    value: String,
    is_completed: bool,
}

fn todo(
    TodoProps {
        id,
        value,
        is_completed,
    }: TodoProps,
) -> String {
    hyperide! {
        <form id={id} class="flex gap-4 bg-gray-50 p-2 rounded">
            <input type="checkbox" checked={is_completed} _="on change log 'hi'"/>
            <div
                class="flex-grow cursor-pointer select-none"
                _="on click click() the previous <input/>"
            >
                {value}
            </div>
            <input type="submit" value="Edit" class="text-gray-500" />
            <input type="submit" value="Delete" class="text-gray-500" />
        </form>
    }
}

async fn todos(db: SqlitePool) -> String {
    let todos = sqlx::query!("SELECT * FROM todos").fetch_all(&db).await;

    match todos {
        Ok(todos) => {
            hyperide! {
              <div class="my-4 p-4 max-w-screen-sm mx-auto">
                <h1 class="text-xl font-bold mb-4">Todo App</h1>

                <form>
                    <input
                        type="text"
                        name="todo"
                        placeholder="What needs to be done?"
                        class="w-full p-2 rounded"
                    />

                    <input
                        type="submit"
                        value="Add Todo"
                        class="w-full mt-2 p-2 rounded bg-blue-500 text-white"
                    />
                </form>

                <ul class="flex flex-col gap-2">{
                  todos
                    .iter()
                    .enumerate()
                    .map(move |(_, todo_record)| {
                            todo(TodoProps {
                                id: todo_record.id,
                                value: todo_record.value.clone(),
                                is_completed: todo_record.is_completed,
                            })
                        }
                    )
                    .collect::<String>()
                }</ul>
              </div>
            }
        }
        Err(_) => hyperide! {
          <div class="my-4 p-4 max-w-screen-sm mx-auto">
            <h1 class="text-xl font-bold mb-4">Todo App</h1>
            <p class="text-red-500">There was an error fetching the todos</p>
          </div>
        },
    }
}
