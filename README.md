<img src="https://jwt.io/img/badge-compatible.svg">

### Thusa Template

#### Template Type: API

#### Template Stack

1. Axum
2. SQLX
3. Postgres

#### Template Description

This is the Thusa API template that uses axum, sqlx and postgres to lay out a basic api server. No need to worry about migrations being run, as long as the database is created and you have migrations in the "migrations" directory, the server will run migrations.

#### Template Author: Connor Davis <<connor.davis@thusa.co.za>>

### Template Instructions

1. Clone the repository with (note: you can choose the directory by replacing "./thusa-axsqpg-template")

```bash
git clone https://github.com/connor-davis/thusa-axsqpg-template.git ./thusa-axsqpg-template
```

2. Change directory into the folder

```bash
cd ./thusa-axsqpg-template
```

3. Build the project

```bash
cargo build
```

4. Run the project

```bash
cargo run
```

### Migrations instructions

Well if you are looking here, you don't trust the server automatically running migrations. Here is the instruction list on how to run migrations.

1. Ensure that the database url is in the .env file

```env
DATABASE_URL="postgres://postgres:password@localhost:5432/database"
```

2. Run the database creation command. (This template assumes that you have sqlx install globally)

```bash
sql db create
```

3. Run the migration run command.

```bash
sqlx migrate run
```

4. Revert migrations if needed.

```bash
sqlx migrate revert
```