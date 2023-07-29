#[macro_use]
extern crate dotenv_codegen;

use axum::extract::{Path, State};
use axum::http::Request;
use axum::routing::post;
use axum::Form;
use axum::{response::Html, routing::get, Router};
use hyperide::{
    htmx::include_htmx, hyperide, hyperscript::include_hyperscript, tailwind::include_tailwind,
};
use serde::Deserialize;
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

    fn not_htmx_predicate<Body>(req: &Request<Body>) -> bool {
        !req.headers().contains_key("hx-request")
    }

    // Methods are GET and POST for progressive enhancement reasons
    let app = Router::new()
        .route("/", get(todos_homepage).post(create_todo))
        .route("/todo/:id/update", post(update_todo))
        .route("/todo/:id/delete", get(delete_todo))
        .with_state(db)
        .layer(LiveReloadLayer::new().request_predicate(not_htmx_predicate));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to boot server");
}

async fn todos_homepage(State(db): State<SqlitePool>) -> Html<String> {
    let content = todos(db).await;

    Html(layout(content))
}

fn layout(content: String) -> String {
    hyperide! {
      <!DOCTYPE html>
          <html lang="en" hx-boost="true">
          <head>
            <title>"Todo App"</title>

            { include_tailwind!() }
            { include_htmx!() }
            { include_hyperscript!() }

            <noscript>
                <style>
                    .noscript-visible {
                        display: block !important;
                    }
                </style>
            </noscript>
          </head>
          <body class="min-h-screen bg-gray-200">
            <section class="my-4 p-4 max-w-screen-sm mx-auto">
                <h1 class="text-xl font-bold mb-4">Todo App</h1>

                <form id="add-todo-form" method="post" action="/" hx-swap="afterbegin" hx-target="#todo-list">
                    <label class="block font-semibold pb-1">
                        Todo
                    </label>
                    <input
                        required
                        type="text"
                        name="value"
                        placeholder="What needs to be done?"
                        class="w-full p-2 mb-4 rounded"
                    />

                    <button
                        type="submit"
                        value="Add Todo"
                        class="[&:not(.htmx-request)>.htmx-indicator]:hidden [&.htmx-request>*:not(.htmx-indicator)]:hidden w-full text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center mr-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800 inline-flex items-center justify-center"
                    >
                        <span>
                            Add Todo
                        </span>

                        <span class="htmx-indicator inline-flex items-center justify-center">
                            <svg aria-hidden="true" role="status" class="inline w-4 h-4 mr-3 text-white animate-spin" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg">
                                <path d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z" fill="#E5E7EB"/>
                                <path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="currentColor"/>
                            </svg>
                            <span>
                                Loading...
                            </span>
                        </span>
                    </button>
                </form>

                {content}
            </section>
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
    let todo_item_id = format!("todo-{}", id);
    let todo_item_target = format!("#{}", todo_item_id);

    let delete_formaction = format!("/todo/{}/delete", id);
    let update_formaction = format!("/todo/{}/update", id);

    hyperide! {
        <li
            id={todo_item_id.clone()}
            class="
                [&.htmx-added]:opacity-0
                [&.htmx-swapping]:opacity-0
                [&.htmx-swapping]:-translate-x-full
                translate-x-0
                opacity-1
                transition-all
                ease-in-out
                duration-300"
        >
            <form
                class="flex items-center gap-4 bg-gray-50 p-2 rounded"
                action={update_formaction.clone()}
                hx-swap="innerHTML"
                hx-target={todo_item_target.clone()}
                method="post"
            >
                <input
                    type="checkbox"
                    class="w-4 h-4"
                    name="is_completed"
                    value={is_completed.to_string()}
                    checked={is_completed}
                    hx-post={update_formaction.clone()}
                    hx-trigger="change"
                />

                <input
                    type="text"
                    name="value"
                    value={value}
                    hx-post={update_formaction}
                    hx-trigger="keyup changed delay:0.5s"
                    hx-sync="closest form:abort"
                />

                <div class="ml-auto flex gap-4">
                    <button type="submit" value="Edit" class="noscript-visible hidden text-gray-500">
                        Edit
                    </button>

                    <button
                        class="text-gray-500"
                        type="submit"
                        hx-get={delete_formaction.clone()}
                        hx-swap="delete swap:0.4s"
                        hx-target={todo_item_target}
                        formaction={delete_formaction}
                        formmethod="get"
                    >
                        Delete
                    </button>
                </div>
            </form>
      </li>
    }
}

struct ErrorAlertProps {
    message: String,
}

fn error_alert(ErrorAlertProps { message }: ErrorAlertProps) -> String {
    hyperide! {
      <div class="my-4 p-4 max-w-screen-sm mx-auto" role="alert">
        <p class="text-red-500">{message}</p>
      </div>
    }
}

async fn todos(db: SqlitePool) -> String {
    let todos = sqlx::query!(
        r#"
        SELECT * FROM todo
        ORDER BY created_at DESC
    "#
    )
    .fetch_all(&db)
    .await;

    match todos {
        Ok(todos) => {
            hyperide! {
                <ul id="todo-list" class="flex flex-col gap-2 py-4">{
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
            }
        }
        Err(_) => hyperide! {
          {error_alert(ErrorAlertProps {
            message: "There was an error fetching the todos".to_string(),
          })}
        },
    }
}

#[derive(Deserialize)]
struct CreateTodoInput {
    value: String,
}

async fn create_todo(
    State(db): State<SqlitePool>,
    Form(input): Form<CreateTodoInput>,
) -> Html<String> {
    let value = input.value.to_string();

    let result = sqlx::query!(
        "INSERT INTO todo (value, is_completed) VALUES (?, false)",
        value
    )
    .execute(&db)
    .await;

    let content = match result {
        Ok(query_result) => {
            let id = query_result.last_insert_rowid();

            hyperide! {
              {todo(TodoProps {
                id,
                value,
                is_completed: false,
              })}
            }
        }
        Err(_) => hyperide! {
            {error_alert(ErrorAlertProps {
                message: "There was an error creating the todo".to_string(),
            })}
        },
    };

    Html(content)
}

async fn delete_todo(State(db): State<SqlitePool>, Path(id): Path<i64>) -> Html<String> {
    let result = sqlx::query!("DELETE FROM todo WHERE id = ?", id)
        .execute(&db)
        .await;

    let content = match result {
        Ok(_) => id.to_string(),
        Err(_) => {
            hyperide! {
                {error_alert(ErrorAlertProps {
                    message: "There was an error creating the todo".to_string(),
                })}
            }
        }
    };

    Html(content)
}

#[derive(Deserialize)]
struct UpdateTodoInput {
    is_completed: Option<String>,
    value: Option<String>,
}

#[axum::debug_handler]
async fn update_todo(
    State(db): State<SqlitePool>,
    Path(id): Path<i64>,
    Form(input): Form<UpdateTodoInput>,
) -> Html<String> {
    let is_completed = match input.is_completed {
        Some(value) => match value.as_str() {
            "true" => true,
            "on" => true,
            "off" => true,
            "false" => true,
            "" => false,
            _ => false,
        },
        None => false,
    };

    let result = sqlx::query!(
        r#"
        UPDATE todo SET 
            is_completed = COALESCE(?, is_completed),
            value        = COALESCE(?, value)
        WHERE id = ?;
        SELECT * FROM todo WHERE id = ?;
        "#,
        is_completed,
        input.value,
        id,
        id
    )
    .fetch_one(&db)
    .await;

    let content = match result {
        Ok(query_result) => {
            hyperide! {
                {todo(TodoProps {
                    id: query_result.id,
                    value: query_result.value,
                    is_completed: query_result.is_completed,
                })}
            }
        }
        Err(_) => {
            hyperide! {
                {error_alert(ErrorAlertProps {
                    message: "There was an error updating the todo".to_string(),
                })}
            }
        }
    };

    Html(content)
}
