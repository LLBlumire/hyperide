# Hyperide Macro

Provides the `hyperide!` macro. See https://crates.io/crate/hyperide for documentation.

## Instructions

1. Copy .env.example to root of the repo (same folder as `/target`) for using within the application

```bash
cp .env.example /PATH-TO-REPOs-ROOT-SAME-LEVEL-AS-TARGET/.env
```

2. Declare the database URL for using with SQLX CLI

```bash
export DATABASE_URL="sqlite:/YOUR_ABSOLUTE_PATH/todo.db"
```

2. Create the database.

```bash
cargo sqlx db create
```

3. Run sql migrations

```bash
cargo sqlx migrate run
```

4. Prepare queries

```bash
cargo sqlx prepare
```

5. Watch and Run

```bash
cargo watch -x run
```
