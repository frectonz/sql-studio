use clap::{Parser, Subcommand};
use color_eyre::eyre::OptionExt;
use tokio::sync::mpsc;
use warp::Filter;

const ROWS_PER_PAGE: i32 = 50;
const SAMPLE_DB: &[u8] = include_bytes!("../sample.sqlite3");

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[clap(subcommand)]
    db: Command,

    /// The address to bind to.
    #[arg(short, long, env, default_value = "127.0.0.1:3030")]
    address: String,

    /// Timeout duration for queries sent from the query page.
    #[clap(short, long, env, default_value = "5secs")]
    timeout: humantime::Duration,

    /// Base path to be provided to the UI. [e.g /sql-studio]
    #[clap(short, long, env)]
    base_path: Option<String>,

    /// Don't open URL in the system browser.
    #[clap(long, env)]
    no_browser: bool,

    /// Don't show the shutdown button in the UI.
    #[clap(long, env)]
    no_shutdown: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// A local SQLite database.
    Sqlite {
        /// Path to the sqlite database file. [use the path "preview" if you don't have an sqlite db at
        /// hand, a sample db will be created for you]
        #[arg(env)]
        database: String,
    },

    /// A remote SQLite database via libSQL.
    Libsql {
        /// libSQL server address
        #[arg(env)]
        url: String,

        /// libSQL authentication token.
        #[arg(env)]
        auth_token: String,
    },

    /// A local SQLite database via libSQL.
    LocalLibsql {
        /// Path to the sqlite database file to be opened with libSQL.
        #[arg(env)]
        database: String,
    },

    /// A PostgreSQL database.
    Postgres {
        /// PostgreSQL connection url [postgresql://postgres:postgres@127.0.0.1/sample]
        #[arg(env)]
        url: String,

        /// PostgreSQL schema
        #[arg(short, long, env, default_value = "public")]
        schema: String,
    },

    /// A MySQL/MariaDB database.
    Mysql {
        /// mysql connection url [mysql://user:password@localhost/sample]
        #[arg(env)]
        url: String,
    },

    /// A local DuckDB database.
    Duckdb {
        /// Path to the the duckdb file.
        #[arg(env)]
        database: String,
    },

    /// A local Parquet file.
    Parquet {
        /// Path to the parquet file.
        #[arg(env)]
        file: String,
    },

    /// A local CSV file.
    Csv {
        /// Path to the CSV file.
        #[arg(env)]
        file: String,
    },

    /// A ClickHouse database.
    Clickhouse {
        /// Address to the clickhouse server.
        #[arg(env, default_value = "http://localhost:8123")]
        url: String,

        /// User we want to authentticate as.
        #[arg(env, default_value = "default")]
        user: String,

        /// Password we want to authentticate with.
        #[arg(env, default_value = "")]
        password: String,

        /// Name of the database.
        #[arg(env, default_value = "default")]
        database: String,
    },

    /// A Microsoft SQL Server database.
    Mssql {
        /// ADO.NET connection string.
        connection: String,
    },
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "tracing=info,warp=debug,sql_studio=debug".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let args = Args::parse();

    let db = match args.db {
        Command::Sqlite { database } => {
            AllDbs::Sqlite(sqlite::Db::open(database, args.timeout.into()).await?)
        }
        Command::Libsql { url, auth_token } => {
            AllDbs::Libsql(libsql::Db::open(url, auth_token, args.timeout.into()).await?)
        }
        Command::LocalLibsql { database } => {
            AllDbs::Libsql(libsql::Db::open_local(database, args.timeout.into()).await?)
        }
        Command::Postgres { url, schema } => {
            AllDbs::Postgres(postgres::Db::open(url, schema, args.timeout.into()).await?)
        }
        Command::Mysql { url } => AllDbs::Mysql(mysql::Db::open(url, args.timeout.into()).await?),
        Command::Duckdb { database } => {
            AllDbs::Duckdb(duckdb::Db::open(database, args.timeout.into()).await?)
        }
        Command::Parquet { file } => {
            AllDbs::Parquet(parquet::Db::open(file, args.timeout.into()).await?)
        }
        Command::Csv { file } => AllDbs::Csv(csv::Db::open(file, args.timeout.into()).await?),
        Command::Clickhouse {
            url,
            user,
            password,
            database,
        } => AllDbs::Clickhouse(Box::new(
            clickhouse::Db::open(url, user, password, database, args.timeout.into()).await?,
        )),
        Command::Mssql { connection } => {
            AllDbs::MsSql(mssql::Db::open(connection, args.timeout.into()).await?)
        }
    };

    let mut index_html = statics::get_index_html()?;
    if let Some(ref base_path) = args.base_path {
        let base = format!(r#"<meta name="BASE_PATH" content="{base_path}" />"#);
        index_html = index_html.replace(r#"<!-- __BASE__ -->"#, &base);
        index_html = index_html.replace("/__ASSETS_PATH__", base_path);
    } else {
        index_html = index_html.replace("/__ASSETS_PATH__", "");
    }

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["Content-Length", "Content-Type"]);

    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

    let api = warp::path("api").and(handlers::routes(db, args.no_shutdown, shutdown_tx));
    let homepage = statics::homepage(index_html.clone());
    let statics = statics::routes(
        match args.base_path.as_ref() {
            Some(base) => base,
            None => "",
        }
        .to_owned(),
    );

    let routes = api
        .or(statics)
        .or(homepage)
        .recover(rejections::handle_rejection)
        .with(cors);

    if args.base_path.is_none() && !args.no_browser {
        let res = open::that(format!("http://{}", args.address));
        tracing::info!("tried to open in browser: {res:?}");
    }

    let routes = if let Some(ref base_path) = args.base_path {
        let path = base_path
            .strip_prefix("/")
            .ok_or_eyre("base path should have a forward slash (/) prefix")?;
        warp::path(path.to_owned()).and(routes).boxed()
    } else {
        routes.boxed()
    };

    let address = args.address.parse::<std::net::SocketAddr>()?;
    let (_, fut) = warp::serve(routes).bind_with_graceful_shutdown(address, async move {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                    println!();
            }
            _ = shutdown_rx.recv() => {
                tracing::info!("received shutdown signal")
            }
        }
    });

    fut.await;
    tracing::info!("shutting down...");

    Ok(())
}

mod statics {
    use std::path::Path;

    use color_eyre::eyre::OptionExt;
    use include_dir::{Dir, include_dir};
    use warp::{
        Filter,
        http::{
            Response,
            header::{CACHE_CONTROL, CONTENT_TYPE},
        },
    };

    static STATIC_DIR: Dir = include_dir!("ui/dist");

    async fn send_file(
        path: warp::path::Tail,
        base_path_replacer: String,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let path = Path::new(path.as_str());
        let file = STATIC_DIR
            .get_file(path)
            .ok_or_else(warp::reject::not_found)?;

        let content_type = match file.path().extension() {
            Some(ext) if ext == "css" => "text/css",
            Some(ext) if ext == "svg" => "image/svg+xml",
            Some(ext) if ext == "js" => "text/javascript",
            Some(ext) if ext == "html" => "text/html",
            _ => "application/octet-stream",
        };

        let resp = Response::builder()
            .header(CONTENT_TYPE, content_type)
            .header(CACHE_CONTROL, "max-age=3600, must-revalidate")
            .body(match file.contents_utf8() {
                Some(c) => c
                    .replace("/__ASSETS_PATH__", &base_path_replacer)
                    .into_bytes(),
                None => file.contents().to_vec(),
            })
            .unwrap();

        Ok(resp)
    }

    pub fn get_index_html() -> color_eyre::Result<String> {
        let file = STATIC_DIR
            .get_file("index.html")
            .ok_or_eyre("could not find index.html")?;

        Ok(file
            .contents_utf8()
            .ok_or_eyre("contents of index.html is not utf-8")?
            .to_owned())
    }

    pub fn homepage(
        file: String,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::any().map(move || file.clone()))
            .map(|file| {
                Response::builder()
                    .header(CONTENT_TYPE, "text/html")
                    .body(file)
                    .unwrap()
            })
    }

    pub fn routes(
        base_path_replacer: String,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        let base_path_replacer = base_path_replacer.to_owned();
        warp::path::tail()
            .and(warp::any().map(move || base_path_replacer.to_owned()))
            .and_then(send_file)
    }
}

trait Database: Sized + Clone + Send {
    fn overview(
        &self,
    ) -> impl std::future::Future<Output = color_eyre::Result<responses::Overview>> + Send;

    fn tables(
        &self,
    ) -> impl std::future::Future<Output = color_eyre::Result<responses::Tables>> + Send;

    fn table(
        &self,
        name: String,
    ) -> impl std::future::Future<Output = color_eyre::Result<responses::Table>> + Send;

    fn table_data(
        &self,
        name: String,
        page: i32,
    ) -> impl std::future::Future<Output = color_eyre::Result<responses::TableData>> + Send;

    fn tables_with_columns(
        &self,
    ) -> impl std::future::Future<Output = color_eyre::Result<responses::TablesWithColumns>> + Send;

    fn query(
        &self,
        query: String,
    ) -> impl std::future::Future<Output = color_eyre::Result<responses::Query>> + Send;

    fn erd(&self) -> impl std::future::Future<Output = color_eyre::Result<responses::Erd>> + Send;
}

#[derive(Clone)]
enum AllDbs {
    Sqlite(sqlite::Db),
    Libsql(libsql::Db),
    Postgres(postgres::Db),
    Mysql(mysql::Db),
    Duckdb(duckdb::Db),
    Parquet(parquet::Db),
    Csv(csv::Db),
    Clickhouse(Box<clickhouse::Db>),
    MsSql(mssql::Db),
}

impl Database for AllDbs {
    async fn overview(&self) -> color_eyre::Result<responses::Overview> {
        match self {
            AllDbs::Sqlite(x) => x.overview().await,
            AllDbs::Libsql(x) => x.overview().await,
            AllDbs::Postgres(x) => x.overview().await,
            AllDbs::Mysql(x) => x.overview().await,
            AllDbs::Duckdb(x) => x.overview().await,
            AllDbs::Parquet(x) => x.overview().await,
            AllDbs::Csv(x) => x.overview().await,
            AllDbs::Clickhouse(x) => x.overview().await,
            AllDbs::MsSql(x) => x.overview().await,
        }
    }

    async fn tables(&self) -> color_eyre::Result<responses::Tables> {
        match self {
            AllDbs::Sqlite(x) => x.tables().await,
            AllDbs::Libsql(x) => x.tables().await,
            AllDbs::Postgres(x) => x.tables().await,
            AllDbs::Mysql(x) => x.tables().await,
            AllDbs::Duckdb(x) => x.tables().await,
            AllDbs::Parquet(x) => x.tables().await,
            AllDbs::Csv(x) => x.tables().await,
            AllDbs::Clickhouse(x) => x.tables().await,
            AllDbs::MsSql(x) => x.tables().await,
        }
    }

    async fn table(&self, name: String) -> color_eyre::Result<responses::Table> {
        match self {
            AllDbs::Sqlite(x) => x.table(name).await,
            AllDbs::Libsql(x) => x.table(name).await,
            AllDbs::Postgres(x) => x.table(name).await,
            AllDbs::Mysql(x) => x.table(name).await,
            AllDbs::Duckdb(x) => x.table(name).await,
            AllDbs::Parquet(x) => x.table(name).await,
            AllDbs::Csv(x) => x.table(name).await,
            AllDbs::Clickhouse(x) => x.table(name).await,
            AllDbs::MsSql(x) => x.table(name).await,
        }
    }

    async fn table_data(
        &self,
        name: String,
        page: i32,
    ) -> color_eyre::Result<responses::TableData> {
        match self {
            AllDbs::Sqlite(x) => x.table_data(name, page).await,
            AllDbs::Libsql(x) => x.table_data(name, page).await,
            AllDbs::Postgres(x) => x.table_data(name, page).await,
            AllDbs::Mysql(x) => x.table_data(name, page).await,
            AllDbs::Duckdb(x) => x.table_data(name, page).await,
            AllDbs::Parquet(x) => x.table_data(name, page).await,
            AllDbs::Csv(x) => x.table_data(name, page).await,
            AllDbs::Clickhouse(x) => x.table_data(name, page).await,
            AllDbs::MsSql(x) => x.table_data(name, page).await,
        }
    }

    async fn tables_with_columns(&self) -> color_eyre::Result<responses::TablesWithColumns> {
        match self {
            AllDbs::Sqlite(x) => x.tables_with_columns().await,
            AllDbs::Libsql(x) => x.tables_with_columns().await,
            AllDbs::Postgres(x) => x.tables_with_columns().await,
            AllDbs::Mysql(x) => x.tables_with_columns().await,
            AllDbs::Duckdb(x) => x.tables_with_columns().await,
            AllDbs::Parquet(x) => x.tables_with_columns().await,
            AllDbs::Csv(x) => x.tables_with_columns().await,
            AllDbs::Clickhouse(x) => x.tables_with_columns().await,
            AllDbs::MsSql(x) => x.tables_with_columns().await,
        }
    }

    async fn query(&self, query: String) -> color_eyre::Result<responses::Query> {
        match self {
            AllDbs::Sqlite(x) => x.query(query).await,
            AllDbs::Libsql(x) => x.query(query).await,
            AllDbs::Postgres(x) => x.query(query).await,
            AllDbs::Mysql(x) => x.query(query).await,
            AllDbs::Duckdb(x) => x.query(query).await,
            AllDbs::Parquet(x) => x.query(query).await,
            AllDbs::Csv(x) => x.query(query).await,
            AllDbs::Clickhouse(x) => x.query(query).await,
            AllDbs::MsSql(x) => x.query(query).await,
        }
    }

    async fn erd(&self) -> color_eyre::Result<responses::Erd> {
        match self {
            AllDbs::Sqlite(x) => x.erd().await,
            AllDbs::Libsql(x) => x.erd().await,
            AllDbs::Postgres(x) => x.erd().await,
            AllDbs::Mysql(x) => x.erd().await,
            AllDbs::Duckdb(x) => x.erd().await,
            AllDbs::Parquet(x) => x.erd().await,
            AllDbs::Csv(x) => x.erd().await,
            AllDbs::Clickhouse(x) => x.erd().await,
            AllDbs::MsSql(x) => x.erd().await,
        }
    }
}

mod sqlite {
    use color_eyre::eyre::OptionExt;
    use std::{collections::HashMap, path::Path, sync::Arc, time::Duration};
    use tokio_rusqlite::{Connection, OpenFlags};

    use crate::{Database, ROWS_PER_PAGE, SAMPLE_DB, helpers, responses};

    #[derive(Clone)]
    pub struct Db {
        path: String,
        conn: Arc<Connection>,
        query_timeout: Duration,
    }

    impl Db {
        pub async fn open(path: String, query_timeout: Duration) -> color_eyre::Result<Self> {
            let conn = if path == "preview" {
                tokio::fs::write("sample.db", SAMPLE_DB).await?;
                Connection::open_with_flags("sample.db", OpenFlags::SQLITE_OPEN_READ_ONLY).await?
            } else {
                Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_WRITE).await?
            };

            // This is meant to test if the file at path is actually a DB.
            let tables = conn
                .call(|conn| {
                    Ok(conn.query_row(
                        r#"
            SELECT count(*) FROM sqlite_master
            WHERE type="table"
                "#,
                        (),
                        |r| r.get::<_, i32>(0),
                    )?)
                })
                .await?;

            tracing::info!("found {tables} tables in {path}");
            Ok(Self {
                path: if path == "preview" {
                    "sample.db".to_owned()
                } else {
                    path
                },
                query_timeout,
                conn: Arc::new(conn),
            })
        }
    }

    impl Database for Db {
        async fn overview(&self) -> color_eyre::Result<responses::Overview> {
            let file_name = Path::new(&self.path)
                .file_name()
                .ok_or_eyre("failed to get file name overview")?
                .to_str()
                .ok_or_eyre("file name is not utf-8")?
                .to_owned();

            let metadata = tokio::fs::metadata(&self.path).await?;

            let sqlite_version = tokio_rusqlite::version().to_owned();
            let db_size = helpers::format_size(metadata.len() as f64);
            let modified = Some(metadata.modified()?.into());
            let created = metadata.created().ok().map(Into::into);

            let (tables, indexes, triggers, views, row_counts, column_counts, index_counts) = self
                .conn
                .call(move |conn| {
                    let tables = conn.query_row(
                        r#"
            SELECT count(*) FROM sqlite_master
            WHERE type="table"
                "#,
                        (),
                        |r| r.get::<_, i32>(0),
                    )?;

                    let indexes = conn.query_row(
                        r#"
            SELECT count(*) FROM sqlite_master
            WHERE type="index"
                "#,
                        (),
                        |r| r.get::<_, i32>(0),
                    )?;

                    let triggers = conn.query_row(
                        r#"
            SELECT count(*) FROM sqlite_master
            WHERE type="trigger"
                "#,
                        (),
                        |r| r.get::<_, i32>(0),
                    )?;

                    let views = conn.query_row(
                        r#"
            SELECT count(*) FROM sqlite_master
            WHERE type="view"
                "#,
                        (),
                        |r| r.get::<_, i32>(0),
                    )?;

                    let mut stmt =
                        conn.prepare(r#"SELECT name FROM sqlite_master WHERE type="table""#)?;
                    let table_names = stmt.query_map([], |row| row.get::<_, String>(0))?;
                    let table_names = table_names.collect::<Result<Vec<_>, _>>()?;

                    let mut row_counts = HashMap::with_capacity(tables as usize);
                    for name in table_names.iter() {
                        let count = conn
                            .query_row(&format!("SELECT count(*) FROM '{name}'"), (), |r| {
                                r.get::<_, i32>(0)
                            })
                            .unwrap_or(0);

                        row_counts.insert(name.to_owned(), count);
                    }

                    let mut row_counts = row_counts
                        .into_iter()
                        .map(|(name, count)| responses::Count { name, count })
                        .collect::<Vec<_>>();

                    row_counts.sort_by(|a, b| b.count.cmp(&a.count));

                    let mut column_counts = HashMap::with_capacity(tables as usize);
                    for name in table_names.iter() {
                        let count = conn
                            .prepare(&format!("PRAGMA table_info('{name}')"))
                            .and_then(|mut columns| {
                                Ok(columns.query_map((), |r| r.get::<_, String>(1))?.count()
                                    as i32)
                            })
                            .unwrap_or(0);

                        column_counts.insert(name.to_owned(), count);
                    }

                    let mut column_counts = column_counts
                        .into_iter()
                        .map(|(name, count)| responses::Count { name, count })
                        .collect::<Vec<_>>();

                    column_counts.sort_by(|a, b| b.count.cmp(&a.count));

                    let mut index_counts = HashMap::with_capacity(tables as usize);
                    for name in table_names.iter() {
                        let count = conn
                            .query_row(
                                "SELECT count(*) FROM sqlite_master WHERE type='index' AND tbl_name=?1",
                                [name],
                                |r| r.get::<_, i32>(0),
                            )
                            .unwrap_or(0);

                        let has_primary_key = conn
                            .query_row(&format!("PRAGMA table_info('{name}')"), [], |r| {
                                r.get::<_, i32>(5)
                            })
                            .map(|v| v == 1)
                            .unwrap_or(false);

                        let count = if has_primary_key { count + 1 } else { count };

                        index_counts.insert(name.to_owned(), count);
                    }

                    let mut index_counts = index_counts
                        .into_iter()
                        .map(|(name, count)| responses::Count { name, count })
                        .collect::<Vec<_>>();

                    index_counts.sort_by(|a, b| b.count.cmp(&a.count));

                    Ok((
                        tables,
                        indexes,
                        triggers,
                        views,
                        row_counts,
                        column_counts,
                        index_counts,
                    ))
                })
                .await?;

            Ok(responses::Overview {
                file_name,
                sqlite_version: Some(sqlite_version),
                db_size,
                created,
                modified,
                tables,
                indexes,
                triggers,
                views,
                row_counts,
                column_counts,
                index_counts,
            })
        }

        async fn tables(&self) -> color_eyre::Result<responses::Tables> {
            let tables = self
                .conn
                .call(move |conn| {
                    let mut stmt =
                        conn.prepare(r#"SELECT name FROM sqlite_master WHERE type="table""#)?;
                    let table_names = stmt
                        .query_map([], |row| row.get::<_, String>(0))?
                        .collect::<Vec<_>>();

                    let mut table_counts = HashMap::with_capacity(table_names.len());
                    for name in table_names {
                        let name = name?;
                        let count = conn
                            .query_row(&format!("SELECT count(*) FROM '{name}'"), (), |r| {
                                r.get::<_, i32>(0)
                            })
                            .unwrap_or(0);

                        table_counts.insert(name, count);
                    }

                    let mut counts = table_counts
                        .into_iter()
                        .map(|(name, count)| responses::Count { name, count })
                        .collect::<Vec<_>>();

                    counts.sort_by(|a, b| a.count.cmp(&b.count).then(a.name.cmp(&b.name)));

                    Ok(counts)
                })
                .await?;

            Ok(responses::Tables { tables })
        }

        async fn table(&self, name: String) -> color_eyre::Result<responses::Table> {
            let metadata = tokio::fs::metadata(&self.path).await?;
            let more_than_five = metadata.len() > 5_000_000_000;

            Ok(self
                .conn
                .call(move |conn| {
                    let sql = conn.query_row(
                        r#"
                        SELECT sql FROM sqlite_master
                        WHERE type="table" AND name = ?1
                        "#,
                        [&name],
                        |r| r.get::<_, String>(0),
                    )?;

                    let row_count = conn
                        .query_row(&format!("SELECT count(*) FROM '{name}'"), (), |r| {
                            r.get::<_, i32>(0)
                        })
                        .unwrap_or(0);

                    let table_size = if more_than_five {
                        "> 5GB".to_owned()
                    } else {
                        conn.query_row(
                            "SELECT SUM(pgsize) FROM dbstat WHERE name = ?1",
                            [&name],
                            |r| r.get::<_, i64>(0),
                        )
                        .map(|size| helpers::format_size(size as f64))
                        .unwrap_or_else(|_| "N/A".to_owned())
                    };

                    let index_count = conn
                        .query_row(
                            "SELECT count(*) FROM sqlite_master WHERE type='index' AND tbl_name=?1",
                            [&name],
                            |r| r.get::<_, i32>(0),
                        )
                        .unwrap_or(0);

                    let has_primary_key = conn
                        .query_row(&format!("PRAGMA table_info('{name}')"), [], |r| {
                            r.get::<_, i32>(5)
                        })
                        .map(|v| v == 1)
                        .unwrap_or(false);
                    let index_count = if has_primary_key {
                        index_count + 1
                    } else {
                        index_count
                    };

                    let column_count = conn
                        .prepare(&format!("PRAGMA table_info('{name}')"))
                        .and_then(|mut columns| {
                            Ok(columns.query_map((), |r| r.get::<_, String>(1))?.count() as i32)
                        })
                        .unwrap_or(0);

                    Ok(responses::Table {
                        name,
                        sql: Some(sql),
                        row_count,
                        table_size,
                        index_count,
                        column_count,
                    })
                })
                .await?)
        }

        async fn table_data(
            &self,
            name: String,
            page: i32,
        ) -> color_eyre::Result<responses::TableData> {
            Ok(self
                .conn
                .call(move |conn| {
                    let first_column =
                        match conn.query_row(&format!("PRAGMA table_info('{name}')"), [], |r| {
                            r.get::<_, String>(1)
                        }) {
                            Ok(col) => col,
                            Err(_) => {
                                return Ok(responses::TableData {
                                    columns: vec![],
                                    rows: vec![],
                                });
                            }
                        };

                    let offset = (page - 1) * ROWS_PER_PAGE;
                    let mut stmt = match conn.prepare(&format!(
                        r#"
                        SELECT *
                        FROM '{name}'
                        ORDER BY {first_column}
                        LIMIT {ROWS_PER_PAGE}
                        OFFSET {offset}
                        "#
                    )) {
                        Ok(stmt) => stmt,
                        Err(_) => {
                            return Ok(responses::TableData {
                                columns: vec![],
                                rows: vec![],
                            });
                        }
                    };
                    let columns = stmt
                        .column_names()
                        .into_iter()
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>();

                    let columns_len = columns.len();
                    let rows = stmt
                        .query_map((), |r| {
                            let mut rows = Vec::with_capacity(columns_len);
                            for i in 0..columns_len {
                                let val = helpers::rusqlite_value_to_json(r.get_ref(i)?);
                                rows.push(val);
                            }
                            Ok(rows)
                        })?
                        .filter_map(|x| x.ok())
                        .collect::<Vec<_>>();

                    Ok(responses::TableData { columns, rows })
                })
                .await?)
        }

        async fn tables_with_columns(&self) -> color_eyre::Result<responses::TablesWithColumns> {
            Ok(self
                .conn
                .call(move |conn| {
                    let mut stmt =
                        conn.prepare(r#"SELECT name FROM sqlite_master WHERE type="table""#)?;
                    let table_names = stmt
                        .query_map([], |row| row.get::<_, String>(0))?
                        .collect::<Vec<_>>();

                    let mut tables = Vec::with_capacity(table_names.len());
                    for name in table_names {
                        let table_name = name?;

                        let columns = conn
                            .prepare(&format!("PRAGMA table_info('{table_name}')"))
                            .and_then(|mut columns| {
                                Ok(columns
                                    .query_map((), |r| r.get::<_, String>(1))?
                                    .filter_map(|res| res.ok())
                                    .collect::<Vec<_>>())
                            })
                            .unwrap_or_default();

                        tables.push(responses::TableWithColumns {
                            table_name,
                            columns,
                        });
                    }

                    tables.sort_by_key(|t| t.table_name.len());

                    Ok(responses::TablesWithColumns { tables })
                })
                .await?)
        }

        async fn query(&self, query: String) -> color_eyre::Result<responses::Query> {
            let res = self.conn.call(move |conn| {
                let mut stmt = conn.prepare(&query)?;
                let columns = stmt
                    .column_names()
                    .into_iter()
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();

                let columns_len = columns.len();
                let rows: Result<Vec<_>, _> = stmt
                    .query_map((), |r| {
                        let mut rows = Vec::with_capacity(columns_len);
                        for i in 0..columns_len {
                            let val = helpers::rusqlite_value_to_json(r.get_ref(i)?);
                            rows.push(val);
                        }
                        Ok(rows)
                    })?
                    .collect();
                let rows = rows?;

                Ok(responses::Query { columns, rows })
            });

            let res = tokio::time::timeout(self.query_timeout, res).await??;

            Ok(res)
        }

        async fn erd(&self) -> color_eyre::Result<responses::Erd> {
            Ok(self
                .conn
                .call(move |conn| {
                    // Get all table names
                    let mut stmt =
                        conn.prepare(r#"SELECT name FROM sqlite_master WHERE type="table""#)?;
                    let table_names = stmt
                        .query_map([], |row| row.get::<_, String>(0))?
                        .filter_map(|r| r.ok())
                        .collect::<Vec<_>>();

                    let mut tables = Vec::with_capacity(table_names.len());
                    let mut relationships = Vec::new();

                    for table_name in table_names {
                        // Get column info: cid, name, type, notnull, dflt_value, pk
                        let mut col_stmt =
                            conn.prepare(&format!("PRAGMA table_info('{table_name}')"))?;
                        let columns = col_stmt
                            .query_map((), |r| {
                                Ok(responses::ErdColumn {
                                    name: r.get::<_, String>(1)?,
                                    data_type: r.get::<_, String>(2)?,
                                    nullable: r.get::<_, i32>(3)? == 0,
                                    is_primary_key: r.get::<_, i32>(5)? > 0,
                                })
                            })?
                            .filter_map(|r| r.ok())
                            .collect::<Vec<_>>();

                        // Get foreign keys: id, seq, table, from, to, on_update, on_delete, match
                        let mut fk_stmt =
                            conn.prepare(&format!("PRAGMA foreign_key_list('{table_name}')"))?;
                        let fks = fk_stmt
                            .query_map((), |r| {
                                Ok(responses::ErdRelationship {
                                    from_table: table_name.clone(),
                                    from_column: r.get::<_, String>(3)?,
                                    to_table: r.get::<_, String>(2)?,
                                    to_column: r.get::<_, String>(4)?,
                                })
                            })?
                            .filter_map(|r| r.ok())
                            .collect::<Vec<_>>();

                        relationships.extend(fks);
                        tables.push(responses::ErdTable {
                            name: table_name,
                            columns,
                        });
                    }

                    Ok(responses::Erd {
                        tables,
                        relationships,
                    })
                })
                .await?)
        }
    }
}

mod libsql {
    use std::{collections::HashMap, sync::Arc, time::Duration};

    use color_eyre::eyre::OptionExt;
    use futures::{StreamExt, TryStreamExt};
    use libsql::Builder;

    use crate::{Database, ROWS_PER_PAGE, helpers, responses};

    #[derive(Clone)]
    pub struct Db {
        name: String,
        db: Arc<libsql::Database>,
        query_timeout: Duration,
    }

    impl Db {
        pub async fn open(
            url: String,
            auth_token: String,
            query_timeout: Duration,
        ) -> color_eyre::Result<Self> {
            let db = Builder::new_remote(url.to_owned(), auth_token)
                .build()
                .await?;
            let conn = db.connect()?;

            let tables = conn
                .query(
                    r#"
            SELECT count(*) FROM sqlite_master
            WHERE type="table"
                "#,
                    (),
                )
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<i32>(0)?;

            tracing::info!(
                "found {tables} table{} in {url}",
                if tables == 1 { "" } else { "s" }
            );

            Ok(Self {
                name: url,
                query_timeout,
                db: Arc::new(db),
            })
        }

        pub async fn open_local(
            database: String,
            query_timeout: Duration,
        ) -> color_eyre::Result<Self> {
            let db = Builder::new_local(&database).build().await?;
            let conn = db.connect()?;

            let tables = conn
                .query(
                    r#"
            SELECT count(*) FROM sqlite_master
            WHERE type="table"
                "#,
                    (),
                )
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<i32>(0)?;

            tracing::info!(
                "found {tables} table{} in {database}",
                if tables == 1 { "" } else { "s" }
            );

            Ok(Self {
                name: database,
                query_timeout,
                db: Arc::new(db),
            })
        }
    }

    impl Database for Db {
        async fn overview(&self) -> color_eyre::Result<responses::Overview> {
            let file_name = self.name.to_owned();

            let conn = self.db.connect()?;

            let sqlite_version = conn
                .query("SELECT sqlite_version();", ())
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<String>(0)?;

            let page_count = conn
                .query("PRAGMA page_count;", ())
                .await?
                .next()
                .await?
                .ok_or_eyre("could not get page count")?
                .get::<u64>(0)?;
            let page_size = conn
                .query("PRAGMA page_size;", ())
                .await?
                .next()
                .await?
                .ok_or_eyre("could not get page size")?
                .get::<u64>(0)?;

            let db_size = helpers::format_size((page_count * page_size) as f64);
            let modified = None;
            let created = None;

            let tables = conn
                .query(
                    r#"
            SELECT count(*) FROM sqlite_master
            WHERE type="table"
                    "#,
                    (),
                )
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<i32>(0)?;

            let indexes = conn
                .query(
                    r#"
            SELECT count(*) FROM sqlite_master
            WHERE type="index"
                    "#,
                    (),
                )
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<i32>(0)?;

            let triggers = conn
                .query(
                    r#"
            SELECT count(*) FROM sqlite_master
            WHERE type="trigger"
                    "#,
                    (),
                )
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<i32>(0)?;

            let views = conn
                .query(
                    r#"
            SELECT count(*) FROM sqlite_master
            WHERE type="view"
                    "#,
                    (),
                )
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<i32>(0)?;

            let table_names = conn
                .query(r#"SELECT name FROM sqlite_master WHERE type="table""#, ())
                .await?
                .into_stream()
                .map_ok(|r| r.get::<String>(0))
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .filter_map(|r| r.ok())
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();

            let mut row_counts = HashMap::with_capacity(table_names.len());
            for name in table_names.iter() {
                let count = conn
                    .query(&format!("SELECT count(*) FROM '{name}'"), ())
                    .await?
                    .next()
                    .await?
                    .ok_or_eyre("no row returned from db")?
                    .get::<i32>(0)?;

                row_counts.insert(name.to_owned(), count);
            }

            let mut row_counts = row_counts
                .into_iter()
                .map(|(name, count)| responses::Count { name, count })
                .collect::<Vec<_>>();

            row_counts.sort_by(|a, b| b.count.cmp(&a.count));

            let mut column_counts = HashMap::with_capacity(table_names.len());
            for name in table_names.iter() {
                let count = conn
                    .query(&format!("PRAGMA table_info('{name}')"), ())
                    .await?
                    .into_stream()
                    .map_ok(|r| r.get::<String>(1))
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .filter_map(|r| r.ok())
                    .filter_map(|r| r.ok())
                    .count() as i32;

                column_counts.insert(name.to_owned(), count);
            }

            let mut column_counts = column_counts
                .into_iter()
                .map(|(name, count)| responses::Count { name, count })
                .collect::<Vec<_>>();

            column_counts.sort_by(|a, b| b.count.cmp(&a.count));

            let mut index_counts = HashMap::with_capacity(table_names.len());
            for name in table_names.iter() {
                let index_count = conn
                    .query(
                        "SELECT count(*) FROM sqlite_master WHERE type='index' AND tbl_name=?1",
                        [name.to_owned()],
                    )
                    .await?
                    .next()
                    .await?
                    .ok_or_eyre("no row returned from db")?
                    .get::<i32>(0)?;

                let has_primary_key = conn
                    .query(&format!("PRAGMA table_info('{name}')"), ())
                    .await?
                    .next()
                    .await?
                    .ok_or_eyre("no row returned from db")?
                    .get::<i32>(5)?
                    == 1;

                let index_count = if has_primary_key {
                    index_count + 1
                } else {
                    index_count
                };

                index_counts.insert(name.to_owned(), index_count);
            }

            let mut index_counts = index_counts
                .into_iter()
                .map(|(name, count)| responses::Count { name, count })
                .collect::<Vec<_>>();

            index_counts.sort_by(|a, b| b.count.cmp(&a.count));

            Ok(responses::Overview {
                file_name,
                sqlite_version: Some(sqlite_version),
                db_size,
                created,
                modified,
                tables,
                indexes,
                triggers,
                views,
                row_counts,
                column_counts,
                index_counts,
            })
        }

        async fn tables(&self) -> color_eyre::Result<responses::Tables> {
            let conn = self.db.connect()?;

            let table_names = conn
                .query(r#"SELECT name FROM sqlite_master WHERE type="table""#, ())
                .await?
                .into_stream()
                .map_ok(|r| r.get::<String>(0))
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .filter_map(|r| r.ok())
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();

            let mut table_counts = HashMap::with_capacity(table_names.len());
            for name in table_names {
                let count = conn
                    .query(&format!("SELECT count(*) FROM '{name}'"), ())
                    .await?
                    .next()
                    .await?
                    .ok_or_eyre("no row returned from db")?
                    .get::<i32>(0)?;

                table_counts.insert(name, count);
            }

            let mut tables = table_counts
                .into_iter()
                .map(|(name, count)| responses::Count { name, count })
                .collect::<Vec<_>>();

            tables.sort_by_key(|r| r.count);

            Ok(responses::Tables { tables })
        }

        async fn table(&self, name: String) -> color_eyre::Result<responses::Table> {
            let conn = self.db.connect()?;

            let sql = conn
                .query(
                    r#"
                SELECT sql FROM sqlite_master
                WHERE type="table" AND name = ?1
                "#,
                    [name.to_owned()],
                )
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<String>(0)?;

            let row_count = conn
                .query(&format!("SELECT count(*) FROM '{name}'"), ())
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<i32>(0)?;

            let table_size = conn
                .query(
                    "SELECT SUM(pgsize) FROM dbstat WHERE name = ?1",
                    [name.to_owned()],
                )
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<i64>(0)?;
            let table_size = helpers::format_size(table_size as f64);

            let index_count = conn
                .query(
                    "SELECT count(*) FROM sqlite_master WHERE type='index' AND tbl_name=?1",
                    [name.to_owned()],
                )
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<i32>(0)?;

            let has_primary_key = conn
                .query(&format!("PRAGMA table_info('{name}')"), ())
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<i32>(5)?
                == 1;

            let index_count = if has_primary_key {
                index_count + 1
            } else {
                index_count
            };

            let column_count = conn
                .query(&format!("PRAGMA table_info('{name}')"), ())
                .await?
                .into_stream()
                .map_ok(|r| r.get::<String>(1))
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .filter_map(|r| r.ok())
                .filter_map(|r| r.ok())
                .count() as i32;

            Ok(responses::Table {
                name,
                sql: Some(sql),
                row_count,
                table_size,
                index_count,
                column_count,
            })
        }

        async fn table_data(
            &self,
            name: String,
            page: i32,
        ) -> color_eyre::Result<responses::TableData> {
            let conn = self.db.connect()?;

            let first_column = conn
                .query(&format!("PRAGMA table_info('{name}')"), ())
                .await?
                .next()
                .await?
                .ok_or_eyre("no row returned from db")?
                .get::<String>(1)?;

            let columns = conn
                .query(&format!("PRAGMA table_info('{name}')"), ())
                .await?
                .into_stream()
                .map_ok(|r| r.get::<String>(1))
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .filter_map(|r| r.ok())
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();

            let columns_len = columns.len();
            let offset = (page - 1) * ROWS_PER_PAGE;
            let rows = conn
                .query(
                    &format!(
                        r#"
                SELECT *
                FROM '{name}'
                ORDER BY {first_column}
                LIMIT {ROWS_PER_PAGE}
                OFFSET {offset}
                        "#,
                    ),
                    (),
                )
                .await?
                .into_stream()
                .map_ok(|r| {
                    let mut rows = Vec::with_capacity(columns_len);
                    for i in 0..columns_len {
                        let val = helpers::libsql_value_to_json(r.get_value(i as i32)?);
                        rows.push(val);
                    }
                    color_eyre::eyre::Ok(rows)
                })
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .filter_map(|r| r.ok())
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();

            Ok(responses::TableData { columns, rows })
        }

        async fn tables_with_columns(&self) -> color_eyre::Result<responses::TablesWithColumns> {
            let conn = self.db.connect()?;

            let table_names = conn
                .query(r#"SELECT name FROM sqlite_master WHERE type="table""#, ())
                .await?
                .into_stream()
                .map_ok(|r| r.get::<String>(0))
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .filter_map(|r| r.ok())
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();

            let mut tables = Vec::with_capacity(table_names.len());
            for table_name in table_names {
                let columns = conn
                    .query(&format!("PRAGMA table_info('{table_name}')"), ())
                    .await?
                    .into_stream()
                    .map_ok(|r| r.get::<String>(1))
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .filter_map(|r| r.ok())
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                tables.push(responses::TableWithColumns {
                    table_name,
                    columns,
                });
            }

            tables.sort_by_key(|t| t.table_name.len());

            Ok(responses::TablesWithColumns { tables })
        }

        async fn query(&self, query: String) -> color_eyre::Result<responses::Query> {
            let conn = self.db.connect()?;
            let mut stmt = conn.prepare(&query).await?;

            let rows = stmt
                .query(())
                .await?
                .into_stream()
                .map_ok(|r| {
                    let mut rows = HashMap::new();
                    let mut index = 0;

                    while let Some(name) = r.column_name(index) {
                        let val = helpers::libsql_value_to_json(r.get_value(index)?);
                        rows.insert(name.to_owned(), val);
                        index += 1;
                    }

                    color_eyre::eyre::Ok(rows)
                })
                .collect::<Vec<_>>();

            let rows = tokio::time::timeout(self.query_timeout, rows).await?;

            let rows = rows
                .into_iter()
                .filter_map(|r| r.ok())
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();

            let columns = rows
                .first()
                .map(|r| r.keys().map(ToOwned::to_owned).collect::<Vec<_>>())
                .unwrap_or_default();

            let rows = rows
                .into_iter()
                .map(|mut r| {
                    let mut rows = Vec::with_capacity(columns.len());
                    for col in columns.iter() {
                        rows.push(r.remove(col).unwrap());
                    }
                    rows
                })
                .collect::<Vec<_>>();

            Ok(responses::Query { columns, rows })
        }

        async fn erd(&self) -> color_eyre::Result<responses::Erd> {
            let conn = self.db.connect()?;

            // Get all table names
            let table_names = conn
                .query(r#"SELECT name FROM sqlite_master WHERE type="table""#, ())
                .await?
                .into_stream()
                .map_ok(|r| r.get::<String>(0))
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .filter_map(|r| r.ok())
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();

            let mut tables = Vec::with_capacity(table_names.len());
            let mut relationships = Vec::new();

            for table_name in table_names {
                // Get column info: cid, name, type, notnull, dflt_value, pk
                let columns = conn
                    .query(&format!("PRAGMA table_info('{table_name}')"), ())
                    .await?
                    .into_stream()
                    .map_ok(|r| {
                        color_eyre::eyre::Ok(responses::ErdColumn {
                            name: r.get::<String>(1)?,
                            data_type: r.get::<String>(2)?,
                            nullable: r.get::<i32>(3)? == 0,
                            is_primary_key: r.get::<i32>(5)? > 0,
                        })
                    })
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .filter_map(|r| r.ok())
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                // Get foreign keys: id, seq, table, from, to, on_update, on_delete, match
                let fks = conn
                    .query(&format!("PRAGMA foreign_key_list('{table_name}')"), ())
                    .await?
                    .into_stream()
                    .map_ok(|r| {
                        let tn = table_name.clone();
                        color_eyre::eyre::Ok(responses::ErdRelationship {
                            from_table: tn,
                            from_column: r.get::<String>(3)?,
                            to_table: r.get::<String>(2)?,
                            to_column: r.get::<String>(4)?,
                        })
                    })
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .filter_map(|r| r.ok())
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                relationships.extend(fks);
                tables.push(responses::ErdTable {
                    name: table_name,
                    columns,
                });
            }

            Ok(responses::Erd {
                tables,
                relationships,
            })
        }
    }
}

mod postgres {
    use std::{sync::Arc, time::Duration};

    use tokio_postgres::Client;

    use crate::{
        Database, ROWS_PER_PAGE, helpers,
        responses::{self, Count},
    };

    #[derive(Clone)]
    pub struct Db {
        client: Arc<Client>,
        schema: String,
        query_timeout: Duration,
    }

    impl Db {
        pub async fn open(
            url: String,
            schema: String,
            query_timeout: Duration,
        ) -> color_eyre::Result<Self> {
            let connector = native_tls::TlsConnector::builder().build()?;
            let connector = postgres_native_tls::MakeTlsConnector::new(connector);

            let (client, connection) = tokio_postgres::connect(&url, connector).await?;

            // The connection object performs the actual communication with the database,
            // so spawn it off to run on its own.
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("postgres connection error: {e}");
                }
            });

            let tables: i64 = client
                .query_one(
                    &format!(
                        r#"
            SELECT count(*)
            FROM information_schema.tables
            WHERE table_schema = '{schema}'
            AND table_type = 'BASE TABLE'
                        "#
                    ),
                    &[],
                )
                .await?
                .get(0);

            tracing::info!(
                "found {tables} table{} in {url}",
                if tables == 1 { "" } else { "s" }
            );

            Ok(Self {
                schema,
                query_timeout,
                client: Arc::new(client),
            })
        }
    }

    impl Database for Db {
        async fn overview(&self) -> color_eyre::Result<responses::Overview> {
            let schema = &self.schema;

            let file_name: String = self
                .client
                .query_one("SELECT current_database()", &[])
                .await?
                .get(0);

            let db_size: i64 = self
                .client
                .query_one("SELECT pg_database_size($1)", &[&file_name])
                .await?
                .get(0);
            let db_size = helpers::format_size(db_size as f64);

            let modified = None;
            let created = None;

            let tables: i64 = self
                .client
                .query_one(
                    &format!(
                        r#"
            SELECT count(*)
            FROM information_schema.tables
            WHERE table_schema = '{schema}'
            AND table_type = 'BASE TABLE'
                        "#
                    ),
                    &[],
                )
                .await?
                .get(0);

            let indexes: i64 = self
                .client
                .query_one(
                    &format!(
                        r#"
            SELECT count(*) 
            FROM pg_indexes 
            WHERE schemaname = '{schema}'
                       "#
                    ),
                    &[],
                )
                .await?
                .get(0);

            let triggers: i64 = self
                .client
                .query_one(
                    &format!(
                        r#"
            SELECT count(*)
            FROM information_schema.triggers
            WHERE trigger_schema = '{schema}'
                        "#
                    ),
                    &[],
                )
                .await?
                .get(0);

            let views: i64 = self
                .client
                .query_one(
                    &format!(
                        r#"
            SELECT count(*)
            FROM information_schema.views
            WHERE table_schema = '{schema}';
                        "#
                    ),
                    &[],
                )
                .await?
                .get(0);

            let mut row_counts = self
                .client
                .query(
                    &format!(
                        r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = '{schema}'
                        "#
                    ),
                    &[],
                )
                .await?
                .into_iter()
                .map(|r| Count {
                    name: r.get(0),
                    count: 0,
                })
                .collect::<Vec<_>>();

            for table in row_counts.iter_mut() {
                let count: i64 = self
                    .client
                    .query_one(&format!(r#"SELECT count(*) FROM "{}""#, table.name), &[])
                    .await?
                    .get(0);
                table.count = count as i32;
            }

            row_counts.sort_by(|a, b| b.count.cmp(&a.count));

            let mut column_counts = self
                .client
                .query(
                    &format!(
                        r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = '{schema}'
                        "#
                    ),
                    &[],
                )
                .await?
                .into_iter()
                .map(|r| Count {
                    name: r.get(0),
                    count: 0,
                })
                .collect::<Vec<_>>();

            for table in column_counts.iter_mut() {
                let count: i64 = self
                    .client
                    .query_one(
                        &format!(
                            r#"
                SELECT count(*)
                FROM information_schema.columns
                WHERE table_schema = '{schema}'
                AND table_name = '{}'
                            "#,
                            table.name
                        ),
                        &[],
                    )
                    .await?
                    .get(0);

                table.count = count as i32;
            }

            column_counts.sort_by(|a, b| b.count.cmp(&a.count));

            let mut index_counts = self
                .client
                .query(
                    &format!(
                        r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = '{schema}'
                        "#
                    ),
                    &[],
                )
                .await?
                .into_iter()
                .map(|r| Count {
                    name: r.get(0),
                    count: 0,
                })
                .collect::<Vec<_>>();

            for table in index_counts.iter_mut() {
                let count: i64 = self
                    .client
                    .query_one(
                        &format!(
                            r#"
                SELECT count(*)
                FROM pg_indexes
                WHERE tablename = '{}'
                            "#,
                            table.name
                        ),
                        &[],
                    )
                    .await?
                    .get(0);

                table.count = count as i32;
            }

            index_counts.sort_by(|a, b| b.count.cmp(&a.count));

            Ok(responses::Overview {
                file_name,
                sqlite_version: None,
                db_size,
                created,
                modified,
                tables: tables as i32,
                indexes: indexes as i32,
                triggers: triggers as i32,
                views: views as i32,
                row_counts,
                column_counts,
                index_counts,
            })
        }

        async fn tables(&self) -> color_eyre::Result<responses::Tables> {
            let schema = &self.schema;

            let mut tables = self
                .client
                .query(
                    &format!(
                        r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = '{schema}'
                        "#
                    ),
                    &[],
                )
                .await?
                .into_iter()
                .map(|r| Count {
                    name: r.get(0),
                    count: 0,
                })
                .collect::<Vec<_>>();

            for table in tables.iter_mut() {
                let count: i64 = self
                    .client
                    .query_one(&format!(r#"SELECT count(*) FROM "{}""#, table.name), &[])
                    .await?
                    .get(0);
                table.count = count as i32;
            }

            tables.sort_by_key(|r| r.count);

            Ok(responses::Tables { tables })
        }

        async fn table(&self, name: String) -> color_eyre::Result<responses::Table> {
            let schema = &self.schema;

            let row_count: i64 = self
                .client
                .query_one(&format!(r#"SELECT count(*) FROM "{name}""#), &[])
                .await?
                .get(0);

            let table_size: i64 = self
                .client
                .query_one(&format!("SELECT pg_total_relation_size('{name}')"), &[])
                .await?
                .get(0);
            let table_size = helpers::format_size(table_size as f64);

            let index_count: i64 = self
                .client
                .query_one(
                    &format!(
                        r#"
            SELECT count(*)
            FROM pg_indexes
            WHERE tablename = '{name}'
                        "#
                    ),
                    &[],
                )
                .await?
                .get(0);

            let column_count: i64 = self
                .client
                .query_one(
                    &format!(
                        r#"
            SELECT count(*)
            FROM information_schema.columns
            WHERE table_schema = '{schema}'
            AND table_name = '{name}'
                        "#
                    ),
                    &[],
                )
                .await?
                .get(0);

            Ok(responses::Table {
                name,
                sql: None,
                row_count: row_count as i32,
                table_size,
                index_count: index_count as i32,
                column_count: column_count as i32,
            })
        }

        async fn table_data(
            &self,
            name: String,
            page: i32,
        ) -> color_eyre::Result<responses::TableData> {
            let schema = &self.schema;

            let first_column: String = self
                .client
                .query_one(
                    &format!(
                        r#"
            SELECT column_name
            FROM information_schema.columns
            WHERE table_schema = '{schema}'
            AND table_name = '{name}'
            LIMIT 1
                        "#
                    ),
                    &[],
                )
                .await?
                .get(0);

            let offset = (page - 1) * ROWS_PER_PAGE;
            let sql = format!(
                r#"
            SELECT * FROM "{name}"
            ORDER BY {first_column}
            LIMIT {ROWS_PER_PAGE}
            OFFSET {offset}
                "#
            );

            let stmt = self.client.prepare(&sql).await?;
            let columns = stmt
                .columns()
                .iter()
                .map(|c| c.name().to_owned())
                .collect::<Vec<_>>();

            let columns_len = columns.len();
            let rows = self
                .client
                .simple_query(&sql)
                .await?
                .into_iter()
                .filter_map(|r| {
                    if let tokio_postgres::SimpleQueryMessage::Row(row) = r {
                        Some(row)
                    } else {
                        None
                    }
                })
                .map(|r| {
                    let mut rows = Vec::with_capacity(columns_len);
                    for i in 0..columns_len {
                        let val = r.get(i).unwrap_or_default();
                        let val = serde_json::Value::String(val.to_owned());
                        rows.push(val);
                    }
                    rows
                })
                .collect::<Vec<_>>();

            Ok(responses::TableData { columns, rows })
        }

        async fn tables_with_columns(&self) -> color_eyre::Result<responses::TablesWithColumns> {
            let schema = &self.schema;

            let table_names = self
                .client
                .query(
                    &format!(
                        r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = '{schema}'
                        "#
                    ),
                    &[],
                )
                .await?
                .into_iter()
                .map(|r| r.get(0))
                .collect::<Vec<String>>();

            let mut tables = Vec::with_capacity(table_names.len());
            for table_name in table_names {
                let columns = self
                    .client
                    .query(
                        &format!(
                            r#"
                SELECT column_name
                FROM information_schema.columns
                WHERE table_schema = '{schema}'
                AND table_name = '{table_name}'
                            "#
                        ),
                        &[],
                    )
                    .await?
                    .into_iter()
                    .map(|r| r.get(0))
                    .collect::<Vec<String>>();

                tables.push(responses::TableWithColumns {
                    table_name,
                    columns,
                });
            }

            tables.sort_by_key(|t| t.table_name.len());

            Ok(responses::TablesWithColumns { tables })
        }

        async fn query(&self, query: String) -> color_eyre::Result<responses::Query> {
            let stmt = self.client.prepare(&query).await?;
            let columns = stmt
                .columns()
                .iter()
                .map(|c| c.name().to_owned())
                .collect::<Vec<_>>();

            let columns_len = columns.len();
            let rows = self.client.simple_query(&query);
            let rows = tokio::time::timeout(self.query_timeout, rows)
                .await??
                .into_iter()
                .filter_map(|r| {
                    if let tokio_postgres::SimpleQueryMessage::Row(row) = r {
                        Some(row)
                    } else {
                        None
                    }
                })
                .map(|r| {
                    let mut rows = Vec::with_capacity(columns_len);
                    for i in 0..columns_len {
                        let val = r.get(i).unwrap_or_default();
                        let val = serde_json::Value::String(val.to_owned());
                        rows.push(val);
                    }
                    rows
                })
                .collect::<Vec<_>>();

            Ok(responses::Query { columns, rows })
        }

        async fn erd(&self) -> color_eyre::Result<responses::Erd> {
            let schema = &self.schema;

            // Get all tables with columns
            let columns_query = format!(
                r#"
                SELECT
                    c.table_name,
                    c.column_name,
                    c.data_type,
                    c.is_nullable,
                    CASE WHEN kcu.column_name IS NOT NULL THEN true ELSE false END as is_primary_key
                FROM information_schema.columns c
                LEFT JOIN information_schema.table_constraints tc
                    ON c.table_schema = tc.table_schema
                    AND c.table_name = tc.table_name
                    AND tc.constraint_type = 'PRIMARY KEY'
                LEFT JOIN information_schema.key_column_usage kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                    AND c.column_name = kcu.column_name
                    AND c.table_name = kcu.table_name
                WHERE c.table_schema = '{schema}'
                ORDER BY c.table_name, c.ordinal_position
                "#
            );

            let column_rows = self.client.query(&columns_query, &[]).await?;

            let mut table_map: std::collections::HashMap<String, Vec<responses::ErdColumn>> =
                std::collections::HashMap::new();

            for row in column_rows {
                let table_name: String = row.get(0);
                let column = responses::ErdColumn {
                    name: row.get(1),
                    data_type: row.get(2),
                    nullable: row.get::<_, String>(3) == "YES",
                    is_primary_key: row.get(4),
                };
                table_map.entry(table_name).or_default().push(column);
            }

            let tables: Vec<responses::ErdTable> = table_map
                .into_iter()
                .map(|(name, columns)| responses::ErdTable { name, columns })
                .collect();

            // Get foreign key relationships
            let fk_query = format!(
                r#"
                SELECT
                    kcu.table_name as from_table,
                    kcu.column_name as from_column,
                    ccu.table_name as to_table,
                    ccu.column_name as to_column
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                JOIN information_schema.constraint_column_usage ccu
                    ON ccu.constraint_name = tc.constraint_name
                    AND ccu.table_schema = tc.table_schema
                WHERE tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_schema = '{schema}'
                "#
            );

            let fk_rows = self.client.query(&fk_query, &[]).await?;
            let relationships: Vec<responses::ErdRelationship> = fk_rows
                .into_iter()
                .map(|row| responses::ErdRelationship {
                    from_table: row.get(0),
                    from_column: row.get(1),
                    to_table: row.get(2),
                    to_column: row.get(3),
                })
                .collect();

            Ok(responses::Erd {
                tables,
                relationships,
            })
        }
    }
}

mod mysql {
    use std::time::Duration;

    use color_eyre::eyre::OptionExt;
    use mysql_async::{Pool, prelude::*};

    use crate::{
        Database, ROWS_PER_PAGE, helpers,
        responses::{self, Count},
    };

    #[derive(Clone)]
    pub struct Db {
        pool: Pool,
        query_timeout: Duration,
    }

    impl Db {
        pub async fn open(url: String, query_timeout: Duration) -> color_eyre::Result<Self> {
            let pool = Pool::from_url(&url)?;
            let conn = pool.get_conn().await?;

            let tables = r#"
            SELECT count(*) as count
            FROM information_schema.tables
            WHERE table_schema = DATABASE()
            AND table_type = 'BASE TABLE'
                "#
            .with(())
            .first(conn)
            .await?
            .map(|count: i32| count)
            .ok_or_eyre("couldn't count tables")?;

            tracing::info!(
                "found {tables} table{} in {url}",
                if tables == 1 { "" } else { "s" }
            );

            Ok(Self {
                pool,
                query_timeout,
            })
        }
    }

    impl Database for Db {
        async fn overview(&self) -> color_eyre::Result<responses::Overview> {
            let mut conn = self.pool.get_conn().await?;

            let file_name = "SELECT database() AS name"
                .with(())
                .first(&mut conn)
                .await?
                .map(|name: String| name)
                .ok_or_eyre("couldn't get database name")?;

            let db_size = r#"
            SELECT sum(data_length + index_length) AS size
            FROM information_schema.tables
            WHERE table_schema = database()
                "#
            .with(())
            .first(&mut conn)
            .await?
            .map(|size: i64| size)
            .ok_or_eyre("couldn't get database size")?;
            let db_size = helpers::format_size(db_size as f64);

            let modified = None;
            let created = None;

            let tables = r#"
            SELECT count(*) AS count
            FROM information_schema.tables
            WHERE table_schema = database()
            AND table_type = 'BASE TABLE'
                "#
            .with(())
            .first(&mut conn)
            .await?
            .map(|count: i32| count)
            .ok_or_eyre("couldn't count tables")?;

            let indexes = r#"
            SELECT count(*) AS count
            FROM information_schema.statistics
            WHERE table_schema = database()
                "#
            .with(())
            .first(&mut conn)
            .await?
            .map(|count: i32| count)
            .ok_or_eyre("couldn't count indexes")?;

            let triggers = r#"
            SELECT count(*) AS count
            FROM information_schema.triggers
            WHERE trigger_schema = database()
                "#
            .with(())
            .first(&mut conn)
            .await?
            .map(|count: i32| count)
            .ok_or_eyre("couldn't count triggers")?;

            let views = r#"
            SELECT COUNT(*) AS count
            FROM information_schema.views
            WHERE table_schema = database()
                "#
            .with(())
            .first(&mut conn)
            .await?
            .map(|count: i32| count)
            .ok_or_eyre("couldn't count views")?;

            let mut row_counts = r#"
            SELECT TABLE_NAME AS name
            FROM information_schema.tables
            WHERE table_schema = database()
                "#
            .with(())
            .map(&mut conn, |name| Count { name, count: 0 })
            .await?;

            for count in row_counts.iter_mut() {
                count.count = format!("SELECT count(*) AS count FROM {}", count.name)
                    .with(())
                    .first(&mut conn)
                    .await?
                    .map(|count: i32| count)
                    .ok_or_eyre("couldn't count rows")?;
            }

            row_counts.sort_by(|a, b| b.count.cmp(&a.count));

            let mut column_counts = r#"
            SELECT TABLE_NAME AS name
            FROM information_schema.tables
            WHERE table_schema = database()
                "#
            .with(())
            .map(&mut conn, |name| Count { name, count: 0 })
            .await?;

            for count in column_counts.iter_mut() {
                count.count = r#"
                SELECT count(*) AS count
                FROM information_schema.columns
                WHERE table_schema = database() AND table_name = :table_name
                "#
                .with(params! {
                    "table_name" => &count.name
                })
                .first(&mut conn)
                .await?
                .map(|count: i32| count)
                .ok_or_eyre("couldn't count columns")?;
            }

            column_counts.sort_by(|a, b| b.count.cmp(&a.count));

            let mut index_counts = r#"
            SELECT TABLE_NAME AS name
            FROM information_schema.tables
            WHERE table_schema = database()
                "#
            .with(())
            .map(&mut conn, |name| Count { name, count: 0 })
            .await?;

            for count in index_counts.iter_mut() {
                count.count = r#"
                SELECT COUNT(*) AS count
                FROM information_schema.statistics
                WHERE table_schema = database() AND table_name = :table_name
                "#
                .with(params! {
                    "table_name" => &count.name
                })
                .first(&mut conn)
                .await?
                .map(|count: i32| count)
                .ok_or_eyre("couldn't count indexes")?;
            }

            index_counts.sort_by(|a, b| b.count.cmp(&a.count));

            Ok(responses::Overview {
                file_name,
                sqlite_version: None,
                db_size,
                created,
                modified,
                tables,
                indexes,
                triggers,
                views,
                row_counts,
                column_counts,
                index_counts,
            })
        }

        async fn tables(&self) -> color_eyre::Result<responses::Tables> {
            let mut conn = self.pool.get_conn().await?;

            let mut tables = r#"
            SELECT TABLE_NAME AS name
            FROM information_schema.tables
            WHERE table_schema = database()
                "#
            .with(())
            .map(&mut conn, |name| Count { name, count: 0 })
            .await?;

            for table in tables.iter_mut() {
                table.count = format!("SELECT count(*) AS count FROM {}", table.name)
                    .with(())
                    .first(&mut conn)
                    .await?
                    .map(|count: i32| count)
                    .ok_or_eyre("couldn't count rows")?;
            }

            tables.sort_by_key(|r| r.count);

            Ok(responses::Tables { tables })
        }

        async fn table(&self, name: String) -> color_eyre::Result<responses::Table> {
            let mut conn = self.pool.get_conn().await?;

            let sql = format!("SHOW CREATE TABLE {name}")
                .with(())
                .first(&mut conn)
                .await?
                .map(|(_, sql): (String, String)| sql)
                .ok_or_eyre("couldn't get table sql")?;

            let row_count = format!("SELECT count(*) AS count FROM {name}")
                .with(())
                .first(&mut conn)
                .await?
                .map(|count: i32| count)
                .ok_or_eyre("couldn't count rows")?;

            let table_size = r#"
            SELECT (data_length + index_length) AS size
            FROM information_schema.tables
            WHERE table_schema = database() AND table_name = :table_name
                "#
            .with(params! {
                "table_name" => &name
            })
            .first(&mut conn)
            .await?
            .map(|size: i64| size)
            .ok_or_eyre("couldn't get table size")?;
            let table_size = helpers::format_size(table_size as f64);

            let index_count = r#"
            SELECT COUNT(*) AS count
            FROM information_schema.statistics
            WHERE table_schema = database() AND table_name = :table_name
                "#
            .with(params! {
                "table_name" => &name
            })
            .first(&mut conn)
            .await?
            .map(|count: i32| count)
            .ok_or_eyre("couldn't count indexes")?;

            let column_count = r#"
            SELECT count(*) AS count
            FROM information_schema.columns
            WHERE table_schema = database() AND table_name = :table_name
                "#
            .with(params! {
                "table_name" => &name
            })
            .first(&mut conn)
            .await?
            .map(|count: i32| count)
            .ok_or_eyre("couldn't count columns")?;

            Ok(responses::Table {
                name,
                sql: Some(sql),
                row_count,
                table_size,
                index_count,
                column_count,
            })
        }

        async fn table_data(
            &self,
            name: String,
            page: i32,
        ) -> color_eyre::Result<responses::TableData> {
            let mut conn = self.pool.get_conn().await?;

            let first_column = r#"
            SELECT column_name FROM information_schema.columns
            WHERE table_schema = DATABASE() AND table_name = :table_name LIMIT 1
                "#
            .with(params! {
                "table_name" => &name
            })
            .first(&mut conn)
            .await?
            .map(|count: String| count)
            .ok_or_eyre("couldn't get first column")?;

            let offset = (page - 1) * ROWS_PER_PAGE;
            let sql = format!(
                r#"
            SELECT * FROM {name}
            ORDER BY {first_column}
            LIMIT {ROWS_PER_PAGE}
            OFFSET {offset}
                "#
            );

            let stmt = conn.prep(&sql).await?;
            let columns = stmt
                .columns()
                .iter()
                .map(|c| c.name_str().to_string())
                .collect::<Vec<_>>();

            let columns_len = columns.len();
            let rows = conn
                .query_iter(sql)
                .await?
                .map_and_drop(|mut r| {
                    let mut row: Vec<mysql_async::Value> = Vec::with_capacity(columns_len);

                    for i in 0..columns_len {
                        row.push(r.take(i).unwrap())
                    }

                    row
                })
                .await?;
            let rows = rows
                .into_iter()
                .map(|r| {
                    r.into_iter()
                        .map(|c| c.as_sql(true))
                        .map(serde_json::Value::String)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            Ok(responses::TableData { columns, rows })
        }

        async fn tables_with_columns(&self) -> color_eyre::Result<responses::TablesWithColumns> {
            let mut conn = self.pool.get_conn().await?;

            let table_names = r#"
            SELECT TABLE_NAME AS name
            FROM information_schema.tables
            WHERE table_schema = database()
                "#
            .with(())
            .map(&mut conn, |name: String| name)
            .await?;

            let mut tables = Vec::with_capacity(table_names.len());
            for table_name in table_names {
                let columns = r#"
                SELECT COLUMN_NAME AS name
                FROM information_schema.columns
                WHERE table_schema = database() AND table_name = :table_name
                "#
                .with(params! {
                    "table_name" => &table_name
                })
                .map(&mut conn, |name: String| name)
                .await?;

                tables.push(responses::TableWithColumns {
                    table_name,
                    columns,
                });
            }

            tables.sort_by_key(|t| t.table_name.len());

            Ok(responses::TablesWithColumns { tables })
        }

        async fn query(&self, query: String) -> color_eyre::Result<responses::Query> {
            let mut conn = self.pool.get_conn().await?;

            let stmt = conn.prep(&query).await?;
            let columns = stmt
                .columns()
                .iter()
                .map(|c| c.name_str().to_string())
                .collect::<Vec<_>>();

            let columns_len = columns.len();
            let rows = conn.query_iter(query);
            let rows = tokio::time::timeout(self.query_timeout, rows)
                .await??
                .map_and_drop(|mut r| {
                    let mut row: Vec<mysql_async::Value> = Vec::with_capacity(columns_len);

                    for i in 0..columns_len {
                        row.push(r.take(i).unwrap())
                    }

                    row
                })
                .await?;
            let rows = rows
                .into_iter()
                .map(|r| {
                    r.into_iter()
                        .map(|c| c.as_sql(true))
                        .map(serde_json::Value::String)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            Ok(responses::Query { columns, rows })
        }

        async fn erd(&self) -> color_eyre::Result<responses::Erd> {
            let mut conn = self.pool.get_conn().await?;

            // Get all tables with columns
            let columns_query = r#"
                SELECT
                    c.TABLE_NAME,
                    c.COLUMN_NAME,
                    c.DATA_TYPE,
                    c.IS_NULLABLE,
                    c.COLUMN_KEY
                FROM information_schema.columns c
                WHERE c.TABLE_SCHEMA = database()
                ORDER BY c.TABLE_NAME, c.ORDINAL_POSITION
            "#;

            let column_rows: Vec<(String, String, String, String, String)> = columns_query
                .with(())
                .map(
                    &mut conn,
                    |(table_name, column_name, data_type, is_nullable, column_key): (
                        String,
                        String,
                        String,
                        String,
                        String,
                    )| {
                        (table_name, column_name, data_type, is_nullable, column_key)
                    },
                )
                .await?;

            let mut table_map: std::collections::HashMap<String, Vec<responses::ErdColumn>> =
                std::collections::HashMap::new();

            for (table_name, column_name, data_type, is_nullable, column_key) in column_rows {
                let column = responses::ErdColumn {
                    name: column_name,
                    data_type,
                    nullable: is_nullable == "YES",
                    is_primary_key: column_key == "PRI",
                };
                table_map.entry(table_name).or_default().push(column);
            }

            let tables: Vec<responses::ErdTable> = table_map
                .into_iter()
                .map(|(name, columns)| responses::ErdTable { name, columns })
                .collect();

            // Get foreign key relationships
            let fk_query = r#"
                SELECT
                    TABLE_NAME as from_table,
                    COLUMN_NAME as from_column,
                    REFERENCED_TABLE_NAME as to_table,
                    REFERENCED_COLUMN_NAME as to_column
                FROM information_schema.KEY_COLUMN_USAGE
                WHERE TABLE_SCHEMA = database()
                AND REFERENCED_TABLE_NAME IS NOT NULL
            "#;

            let relationships: Vec<responses::ErdRelationship> = fk_query
                .with(())
                .map(
                    &mut conn,
                    |(from_table, from_column, to_table, to_column): (
                        String,
                        String,
                        String,
                        String,
                    )| {
                        responses::ErdRelationship {
                            from_table,
                            from_column,
                            to_table,
                            to_column,
                        }
                    },
                )
                .await?;

            Ok(responses::Erd {
                tables,
                relationships,
            })
        }
    }
}

mod duckdb {
    use color_eyre::eyre;
    use color_eyre::eyre::OptionExt;
    use duckdb::{Config, Connection};
    use std::{
        path::Path,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::{
        Database, ROWS_PER_PAGE, helpers,
        responses::{self, Count},
    };

    #[derive(Clone)]
    pub struct Db {
        path: String,
        conn: Arc<Mutex<Connection>>,
        query_timeout: Duration,
    }

    impl Db {
        pub async fn open(path: String, query_timeout: Duration) -> color_eyre::Result<Self> {
            let p = path.to_owned();
            let conn = tokio::task::spawn_blocking(move || {
                let config = Config::default().access_mode(duckdb::AccessMode::ReadWrite)?;
                let conn = Connection::open_with_flags(p, config)?;

                eyre::Ok(conn)
            })
            .await??;

            let c = conn.try_clone()?;
            let tables = tokio::task::spawn_blocking(move || {
                let tables: i32 = c.query_row(
                    r#"
                SELECT count(*) 
                FROM information_schema.tables 
                WHERE table_schema = 'main' AND table_type = 'BASE TABLE'
                    "#,
                    [],
                    |row| row.get(0),
                )?;

                eyre::Ok(tables)
            })
            .await??;

            tracing::info!(
                "found {tables} table{} in {path}",
                if tables == 1 { "" } else { "s" }
            );
            Ok(Self {
                path,
                query_timeout,
                conn: Arc::new(Mutex::new(conn)),
            })
        }
    }

    impl Database for Db {
        async fn overview(&self) -> color_eyre::Result<responses::Overview> {
            let file_name = Path::new(&self.path)
                .file_name()
                .ok_or_eyre("failed to get file name overview")?
                .to_str()
                .ok_or_eyre("file name is not utf-8")?
                .to_owned();

            let metadata = tokio::fs::metadata(&self.path).await?;

            let db_size = helpers::format_size(metadata.len() as f64);
            let modified = Some(metadata.modified()?.into());
            let created = metadata.created().ok().map(Into::into);

            let c = self.conn.clone();
            let (tables, indexes, triggers, views, row_counts, column_counts, index_counts) =
                tokio::task::spawn_blocking(move || {
                    let c = c.lock().expect("could not get lock on connection");

                    let tables: i32 = c.query_row(
                        r#"
                    SELECT count(*) 
                    FROM information_schema.tables 
                    WHERE table_schema = 'main' AND table_type = 'BASE TABLE'
                    "#,
                        [],
                        |row| row.get(0),
                    )?;

                    let indexes: i32 =
                        c.query_row("SELECT count(*) FROM duckdb_indexes;", [], |row| row.get(0))?;

                    let triggers: i32 = c.query_row(
                        r#"
                    SELECT count(*)
                    FROM duckdb_constraints
                    WHERE constraint_type = 'TRIGGER'
                    "#,
                        [],
                        |row| row.get(0),
                    )?;

                    let views: i32 = c.query_row(
                        r#"
                    SELECT count(*)
                    FROM information_schema.tables
                    WHERE table_type = 'VIEW'
                    "#,
                        [],
                        |row| row.get(0),
                    )?;

                    let mut table_names_stmt = c.prepare(
                        r#"
                    SELECT table_name
                    FROM information_schema.tables
                    WHERE table_type = 'BASE TABLE'
                        "#,
                    )?;
                    let table_names = table_names_stmt
                        .query_map([], |row| row.get(0))?
                        .filter_map(|n| n.ok())
                        .collect::<Vec<String>>();

                    let mut row_counts = Vec::with_capacity(table_names.len());
                    for name in table_names.iter() {
                        let count: i32 =
                            c.query_row(&format!(r#"SELECT count(*) FROM "{name}""#), [], |row| {
                                row.get(0)
                            })?;

                        row_counts.push(Count {
                            name: name.to_owned(),
                            count,
                        });
                    }

                    row_counts.sort_by(|a, b| b.count.cmp(&a.count));

                    let mut column_counts = Vec::with_capacity(table_names.len());
                    for name in table_names.iter() {
                        let count: i32 = c.query_row(
                            "SELECT column_count FROM duckdb_tables WHERE table_name = ?",
                            [&name],
                            |row| row.get(0),
                        )?;

                        column_counts.push(Count {
                            name: name.to_owned(),
                            count,
                        });
                    }

                    column_counts.sort_by(|a, b| b.count.cmp(&a.count));

                    let mut index_counts = Vec::with_capacity(table_names.len());
                    for name in table_names.iter() {
                        let count: i32 = c.query_row(
                            "SELECT index_count FROM duckdb_tables WHERE table_name = ?",
                            [&name],
                            |row| row.get(0),
                        )?;

                        index_counts.push(Count {
                            name: name.to_owned(),
                            count,
                        });
                    }

                    index_counts.sort_by(|a, b| b.count.cmp(&a.count));

                    eyre::Ok((
                        tables,
                        indexes,
                        triggers,
                        views,
                        row_counts,
                        column_counts,
                        index_counts,
                    ))
                })
                .await??;

            Ok(responses::Overview {
                file_name,
                sqlite_version: None,
                db_size,
                created,
                modified,
                tables,
                indexes,
                triggers,
                views,
                row_counts,
                column_counts,
                index_counts,
            })
        }

        async fn tables(&self) -> color_eyre::Result<responses::Tables> {
            let c = self.conn.clone();
            let tables = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let mut table_names_stmt = c.prepare(
                    r#"
                    SELECT table_name
                    FROM information_schema.tables
                    WHERE table_type = 'BASE TABLE'
                        "#,
                )?;
                let table_names = table_names_stmt
                    .query_map([], |row| row.get(0))?
                    .filter_map(|n| n.ok())
                    .collect::<Vec<String>>();

                let mut counts = Vec::with_capacity(table_names.len());
                for name in table_names {
                    let count: i32 =
                        c.query_row(&format!(r#"SELECT count(*) FROM "{name}""#), [], |row| {
                            row.get(0)
                        })?;

                    counts.push(Count { name, count });
                }

                counts.sort_by_key(|r| r.count);

                eyre::Ok(counts)
            })
            .await??;

            Ok(responses::Tables { tables })
        }

        async fn table(&self, name: String) -> color_eyre::Result<responses::Table> {
            let c = self.conn.clone();

            let (name, sql, row_count, table_size, index_count, column_count) =
                tokio::task::spawn_blocking(move || {
                    let c = c.lock().expect("could not get lock on connection");

                    let sql = None;

                    let row_count: i32 =
                        c.query_row(&format!(r#"SELECT count(*) FROM "{name}""#), [], |row| {
                            row.get(0)
                        })?;

                    let table_size: i64 = c.query_row(
                        "SELECT estimated_size FROM duckdb_tables WHERE table_name = ?",
                        [&name],
                        |row| row.get(0),
                    )?;
                    let table_size = helpers::format_size(table_size as f64);

                    let index_count: i32 = c.query_row(
                        "SELECT index_count FROM duckdb_tables WHERE table_name = ?",
                        [&name],
                        |row| row.get(0),
                    )?;

                    let column_count: i32 = c.query_row(
                        "SELECT column_count FROM duckdb_tables WHERE table_name = ?",
                        [&name],
                        |row| row.get(0),
                    )?;

                    eyre::Ok((name, sql, row_count, table_size, index_count, column_count))
                })
                .await??;

            Ok(responses::Table {
                name,
                sql,
                row_count,
                table_size,
                index_count,
                column_count,
            })
        }

        async fn table_data(
            &self,
            name: String,
            page: i32,
        ) -> color_eyre::Result<responses::TableData> {
            let c = self.conn.clone();

            let (columns, rows) = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let first_column: String =
                    c.query_row(&format!("PRAGMA table_info('{name}')"), [], |row| {
                        row.get(1)
                    })?;

                let offset = (page - 1) * ROWS_PER_PAGE;
                let sql = format!(
                    r#"
                SELECT * FROM "{name}"
                ORDER BY "{first_column}"
                LIMIT {ROWS_PER_PAGE}
                OFFSET {offset};
                    "#
                );
                let mut stmt = c.prepare(&sql)?;

                let rows = stmt
                    .query_map([], |r| {
                        let mut rows = Vec::new();
                        let mut index = 0;

                        while let Ok(val) = r.get_ref(index) {
                            let val = helpers::duckdb_value_to_json(val);
                            rows.push(val);
                            index += 1;
                        }

                        Ok(rows)
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                let columns = stmt.column_names();

                eyre::Ok((columns, rows))
            })
            .await??;

            Ok(responses::TableData { columns, rows })
        }

        async fn tables_with_columns(&self) -> color_eyre::Result<responses::TablesWithColumns> {
            let c = self.conn.clone();
            tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let mut table_names_stmt = c.prepare(
                    r#"
                    SELECT table_name
                    FROM information_schema.tables
                    WHERE table_type = 'BASE TABLE'
                        "#,
                )?;
                let table_names = table_names_stmt
                    .query_map([], |row| row.get(0))?
                    .filter_map(|n| n.ok())
                    .collect::<Vec<String>>();

                let mut tables = Vec::with_capacity(table_names.len());
                for table_name in table_names {
                    let sql = format!(r#"SELECT * FROM "{table_name}" WHERE false"#);
                    let mut stmt = c.prepare(&sql)?;
                    let _ = stmt.query_map([], |_| Ok(()))?;
                    let columns = stmt.column_names();

                    tables.push(responses::TableWithColumns {
                        table_name,
                        columns,
                    });
                }

                tables.sort_by_key(|t| t.table_name.len());

                eyre::Ok(responses::TablesWithColumns { tables })
            })
            .await?
        }

        async fn query(&self, query: String) -> color_eyre::Result<responses::Query> {
            let c = self.conn.clone();

            let future = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let mut stmt = c.prepare(&query)?;

                let rows = stmt
                    .query_map([], |r| {
                        let mut rows = Vec::new();
                        let mut index = 0;

                        while let Ok(val) = r.get_ref(index) {
                            let val = helpers::duckdb_value_to_json(val);
                            rows.push(val);
                            index += 1;
                        }

                        Ok(rows)
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                let columns = stmt.column_names();

                eyre::Ok((columns, rows))
            });

            let (columns, rows) = tokio::time::timeout(self.query_timeout, future).await???;

            Ok(responses::Query { columns, rows })
        }

        async fn erd(&self) -> color_eyre::Result<responses::Erd> {
            let c = self.conn.clone();
            tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                // Get all tables with columns from information_schema
                let mut col_stmt = c.prepare(
                    r#"
                    SELECT
                        table_name,
                        column_name,
                        data_type,
                        is_nullable
                    FROM information_schema.columns
                    WHERE table_schema = current_schema()
                    ORDER BY table_name, ordinal_position
                    "#,
                )?;

                let column_rows = col_stmt
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                        ))
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<(String, String, String, String)>>();

                // Get primary key info - using a query that returns table_name and column_name pairs
                let mut pk_stmt = c.prepare(
                    r#"
                    SELECT table_name, unnest(constraint_column_names) as column_name
                    FROM duckdb_constraints()
                    WHERE constraint_type = 'PRIMARY KEY'
                    "#,
                )?;

                let pk_columns: std::collections::HashSet<(String, String)> = pk_stmt
                    .query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })?
                    .filter_map(|r| r.ok())
                    .collect();

                let mut table_map: std::collections::HashMap<String, Vec<responses::ErdColumn>> =
                    std::collections::HashMap::new();

                for (table_name, column_name, data_type, is_nullable) in column_rows {
                    let is_pk = pk_columns.contains(&(table_name.clone(), column_name.clone()));

                    let column = responses::ErdColumn {
                        name: column_name,
                        data_type,
                        nullable: is_nullable == "YES",
                        is_primary_key: is_pk,
                    };
                    table_map.entry(table_name).or_default().push(column);
                }

                let tables: Vec<responses::ErdTable> = table_map
                    .into_iter()
                    .map(|(name, columns)| responses::ErdTable { name, columns })
                    .collect();

                // DuckDB has limited FK support, return empty relationships
                eyre::Ok(responses::Erd {
                    tables,
                    relationships: Vec::new(),
                })
            })
            .await?
        }
    }
}

mod parquet {
    use color_eyre::eyre;
    use color_eyre::eyre::OptionExt;
    use duckdb::Connection;
    use std::{
        path::Path,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::{
        Database, ROWS_PER_PAGE, helpers,
        responses::{self, Count},
    };

    #[derive(Clone)]
    pub struct Db {
        path: String,
        table_name: String,
        conn: Arc<Mutex<Connection>>,
        query_timeout: Duration,
    }

    impl Db {
        pub async fn open(path: String, query_timeout: Duration) -> color_eyre::Result<Self> {
            let p = path.clone();
            let table_name = Path::new(&path)
                .file_stem()
                .ok_or_eyre("failed to get file stem")?
                .to_str()
                .ok_or_eyre("file stem is not utf-8")?
                .to_owned();

            let tn = table_name.clone();
            let conn = tokio::task::spawn_blocking(move || {
                let conn = Connection::open_in_memory()?;

                // Create a view that reads from the parquet file
                conn.execute(
                    &format!(r#"CREATE VIEW "{tn}" AS SELECT * FROM read_parquet('{p}')"#),
                    [],
                )?;

                eyre::Ok(conn)
            })
            .await??;

            tracing::info!("opened parquet file {path} as table '{table_name}'");
            Ok(Self {
                path,
                table_name,
                query_timeout,
                conn: Arc::new(Mutex::new(conn)),
            })
        }
    }

    impl Database for Db {
        async fn overview(&self) -> color_eyre::Result<responses::Overview> {
            let file_name = Path::new(&self.path)
                .file_name()
                .ok_or_eyre("failed to get file name")?
                .to_str()
                .ok_or_eyre("file name is not utf-8")?
                .to_owned();

            let metadata = tokio::fs::metadata(&self.path).await?;

            let db_size = helpers::format_size(metadata.len() as f64);
            let modified = Some(metadata.modified()?.into());
            let created = metadata.created().ok().map(Into::into);

            let c = self.conn.clone();
            let table_name = self.table_name.clone();
            let (row_count, column_count) = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let row_count: i32 = c.query_row(
                    &format!(r#"SELECT count(*) FROM "{table_name}""#),
                    [],
                    |row| row.get(0),
                )?;

                let mut columns_stmt =
                    c.prepare(&format!(r#"PRAGMA table_info('{table_name}')"#))?;
                let column_count = columns_stmt.query_map([], |_| Ok(()))?.count() as i32;

                eyre::Ok((row_count, column_count))
            })
            .await??;

            let row_counts = vec![Count {
                name: self.table_name.clone(),
                count: row_count,
            }];

            let column_counts = vec![Count {
                name: self.table_name.clone(),
                count: column_count,
            }];

            let index_counts = vec![Count {
                name: self.table_name.clone(),
                count: 0,
            }];

            Ok(responses::Overview {
                file_name,
                sqlite_version: None,
                db_size,
                created,
                modified,
                tables: 1,
                indexes: 0,
                triggers: 0,
                views: 0,
                row_counts,
                column_counts,
                index_counts,
            })
        }

        async fn tables(&self) -> color_eyre::Result<responses::Tables> {
            let c = self.conn.clone();
            let table_name = self.table_name.clone();
            let count = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let count: i32 = c.query_row(
                    &format!(r#"SELECT count(*) FROM "{table_name}""#),
                    [],
                    |row| row.get(0),
                )?;

                eyre::Ok(count)
            })
            .await??;

            Ok(responses::Tables {
                tables: vec![Count {
                    name: self.table_name.clone(),
                    count,
                }],
            })
        }

        async fn table(&self, name: String) -> color_eyre::Result<responses::Table> {
            let c = self.conn.clone();
            let file_size = tokio::fs::metadata(&self.path).await?.len();

            let (row_count, column_count) = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let row_count: i32 =
                    c.query_row(&format!(r#"SELECT count(*) FROM "{name}""#), [], |row| {
                        row.get(0)
                    })?;

                let mut columns_stmt = c.prepare(&format!(r#"PRAGMA table_info('{name}')"#))?;
                let column_count = columns_stmt.query_map([], |_| Ok(()))?.count() as i32;

                eyre::Ok((row_count, column_count))
            })
            .await??;

            Ok(responses::Table {
                name: self.table_name.clone(),
                sql: None,
                row_count,
                index_count: 0,
                column_count,
                table_size: helpers::format_size(file_size as f64),
            })
        }

        async fn table_data(
            &self,
            name: String,
            page: i32,
        ) -> color_eyre::Result<responses::TableData> {
            let c = self.conn.clone();

            let (columns, rows) = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let first_column: String =
                    c.query_row(&format!("PRAGMA table_info('{name}')"), [], |row| {
                        row.get(1)
                    })?;

                let offset = (page - 1) * ROWS_PER_PAGE;
                let sql = format!(
                    r#"
                SELECT * FROM "{name}"
                ORDER BY "{first_column}"
                LIMIT {ROWS_PER_PAGE}
                OFFSET {offset};
                    "#
                );
                let mut stmt = c.prepare(&sql)?;

                let rows = stmt
                    .query_map([], |r| {
                        let mut rows = Vec::new();
                        let mut index = 0;

                        while let Ok(val) = r.get_ref(index) {
                            let val = helpers::duckdb_value_to_json(val);
                            rows.push(val);
                            index += 1;
                        }

                        Ok(rows)
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                let columns = stmt.column_names();

                eyre::Ok((columns, rows))
            })
            .await??;

            Ok(responses::TableData { columns, rows })
        }

        async fn tables_with_columns(&self) -> color_eyre::Result<responses::TablesWithColumns> {
            let c = self.conn.clone();
            let table_name = self.table_name.clone();
            tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let sql = format!(r#"SELECT * FROM "{table_name}" WHERE false"#);
                let mut stmt = c.prepare(&sql)?;
                let _ = stmt.query_map([], |_| Ok(()))?;
                let columns = stmt.column_names();

                eyre::Ok(responses::TablesWithColumns {
                    tables: vec![responses::TableWithColumns {
                        table_name,
                        columns,
                    }],
                })
            })
            .await?
        }

        async fn query(&self, query: String) -> color_eyre::Result<responses::Query> {
            let c = self.conn.clone();

            let future = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let mut stmt = c.prepare(&query)?;

                let rows = stmt
                    .query_map([], |r| {
                        let mut rows = Vec::new();
                        let mut index = 0;

                        while let Ok(val) = r.get_ref(index) {
                            let val = helpers::duckdb_value_to_json(val);
                            rows.push(val);
                            index += 1;
                        }

                        Ok(rows)
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                let columns = stmt.column_names();

                eyre::Ok((columns, rows))
            });

            let (columns, rows) = tokio::time::timeout(self.query_timeout, future).await???;

            Ok(responses::Query { columns, rows })
        }

        async fn erd(&self) -> color_eyre::Result<responses::Erd> {
            let c = self.conn.clone();
            let table_name = self.table_name.clone();
            tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let mut col_stmt = c.prepare(&format!(
                    r#"
                    SELECT
                        column_name,
                        data_type,
                        is_nullable
                    FROM information_schema.columns
                    WHERE table_name = '{table_name}'
                    ORDER BY ordinal_position
                    "#
                ))?;

                let columns = col_stmt
                    .query_map([], |row| {
                        Ok(responses::ErdColumn {
                            name: row.get::<_, String>(0)?,
                            data_type: row.get::<_, String>(1)?,
                            nullable: row.get::<_, String>(2)? == "YES",
                            is_primary_key: false,
                        })
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                let tables = vec![responses::ErdTable {
                    name: table_name,
                    columns,
                }];

                // Parquet files don't have relationships
                eyre::Ok(responses::Erd {
                    tables,
                    relationships: Vec::new(),
                })
            })
            .await?
        }
    }
}

mod csv {
    use color_eyre::eyre;
    use color_eyre::eyre::OptionExt;
    use duckdb::Connection;
    use std::{
        path::Path,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::{
        Database, ROWS_PER_PAGE, helpers,
        responses::{self, Count},
    };

    #[derive(Clone)]
    pub struct Db {
        path: String,
        table_name: String,
        conn: Arc<Mutex<Connection>>,
        query_timeout: Duration,
    }

    impl Db {
        pub async fn open(path: String, query_timeout: Duration) -> color_eyre::Result<Self> {
            let p = path.clone();
            let table_name = Path::new(&path)
                .file_stem()
                .ok_or_eyre("failed to get file stem")?
                .to_str()
                .ok_or_eyre("file stem is not utf-8")?
                .to_owned();

            let tn = table_name.clone();
            let conn = tokio::task::spawn_blocking(move || {
                let conn = Connection::open_in_memory()?;

                // Create a view that reads from the CSV file
                conn.execute(
                    &format!(r#"CREATE VIEW "{tn}" AS SELECT * FROM read_csv('{p}', header = true, auto_detect = true)"#),
                    [],
                )?;

                eyre::Ok(conn)
            })
            .await??;

            tracing::info!("opened CSV file {path} as table '{table_name}'");
            Ok(Self {
                path,
                table_name,
                query_timeout,
                conn: Arc::new(Mutex::new(conn)),
            })
        }
    }

    impl Database for Db {
        async fn overview(&self) -> color_eyre::Result<responses::Overview> {
            let file_name = Path::new(&self.path)
                .file_name()
                .ok_or_eyre("failed to get file name")?
                .to_str()
                .ok_or_eyre("file name is not utf-8")?
                .to_owned();

            let metadata = tokio::fs::metadata(&self.path).await?;

            let db_size = helpers::format_size(metadata.len() as f64);
            let modified = Some(metadata.modified()?.into());
            let created = metadata.created().ok().map(Into::into);

            let c = self.conn.clone();
            let table_name = self.table_name.clone();
            let (row_count, column_count) = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let row_count: i32 = c.query_row(
                    &format!(r#"SELECT count(*) FROM "{table_name}""#),
                    [],
                    |row| row.get(0),
                )?;

                let mut columns_stmt =
                    c.prepare(&format!(r#"PRAGMA table_info('{table_name}')"#))?;
                let column_count = columns_stmt.query_map([], |_| Ok(()))?.count() as i32;

                eyre::Ok((row_count, column_count))
            })
            .await??;

            let row_counts = vec![Count {
                name: self.table_name.clone(),
                count: row_count,
            }];

            let column_counts = vec![Count {
                name: self.table_name.clone(),
                count: column_count,
            }];

            let index_counts = vec![Count {
                name: self.table_name.clone(),
                count: 0,
            }];

            Ok(responses::Overview {
                file_name,
                sqlite_version: None,
                db_size,
                created,
                modified,
                tables: 1,
                indexes: 0,
                triggers: 0,
                views: 0,
                row_counts,
                column_counts,
                index_counts,
            })
        }

        async fn tables(&self) -> color_eyre::Result<responses::Tables> {
            let c = self.conn.clone();
            let table_name = self.table_name.clone();
            let count = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let count: i32 = c.query_row(
                    &format!(r#"SELECT count(*) FROM "{table_name}""#),
                    [],
                    |row| row.get(0),
                )?;

                eyre::Ok(count)
            })
            .await??;

            Ok(responses::Tables {
                tables: vec![Count {
                    name: self.table_name.clone(),
                    count,
                }],
            })
        }

        async fn table(&self, name: String) -> color_eyre::Result<responses::Table> {
            let c = self.conn.clone();
            let file_size = tokio::fs::metadata(&self.path).await?.len();

            let (row_count, column_count) = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let row_count: i32 =
                    c.query_row(&format!(r#"SELECT count(*) FROM "{name}""#), [], |row| {
                        row.get(0)
                    })?;

                let mut columns_stmt = c.prepare(&format!(r#"PRAGMA table_info('{name}')"#))?;
                let column_count = columns_stmt.query_map([], |_| Ok(()))?.count() as i32;

                eyre::Ok((row_count, column_count))
            })
            .await??;

            Ok(responses::Table {
                name: self.table_name.clone(),
                sql: None,
                row_count,
                index_count: 0,
                column_count,
                table_size: helpers::format_size(file_size as f64),
            })
        }

        async fn table_data(
            &self,
            name: String,
            page: i32,
        ) -> color_eyre::Result<responses::TableData> {
            let c = self.conn.clone();

            let (columns, rows) = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let first_column: String =
                    c.query_row(&format!("PRAGMA table_info('{name}')"), [], |row| {
                        row.get(1)
                    })?;

                let offset = (page - 1) * ROWS_PER_PAGE;
                let sql = format!(
                    r#"
                SELECT * FROM "{name}"
                ORDER BY "{first_column}"
                LIMIT {ROWS_PER_PAGE}
                OFFSET {offset};
                    "#
                );
                let mut stmt = c.prepare(&sql)?;

                let rows = stmt
                    .query_map([], |r| {
                        let mut rows = Vec::new();
                        let mut index = 0;

                        while let Ok(val) = r.get_ref(index) {
                            let val = helpers::duckdb_value_to_json(val);
                            rows.push(val);
                            index += 1;
                        }

                        Ok(rows)
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                let columns = stmt.column_names();

                eyre::Ok((columns, rows))
            })
            .await??;

            Ok(responses::TableData { columns, rows })
        }

        async fn tables_with_columns(&self) -> color_eyre::Result<responses::TablesWithColumns> {
            let c = self.conn.clone();
            let table_name = self.table_name.clone();
            tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let sql = format!(r#"SELECT * FROM "{table_name}" WHERE false"#);
                let mut stmt = c.prepare(&sql)?;
                let _ = stmt.query_map([], |_| Ok(()))?;
                let columns = stmt.column_names();

                eyre::Ok(responses::TablesWithColumns {
                    tables: vec![responses::TableWithColumns {
                        table_name,
                        columns,
                    }],
                })
            })
            .await?
        }

        async fn query(&self, query: String) -> color_eyre::Result<responses::Query> {
            let c = self.conn.clone();

            let future = tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let mut stmt = c.prepare(&query)?;

                let rows = stmt
                    .query_map([], |r| {
                        let mut rows = Vec::new();
                        let mut index = 0;

                        while let Ok(val) = r.get_ref(index) {
                            let val = helpers::duckdb_value_to_json(val);
                            rows.push(val);
                            index += 1;
                        }

                        Ok(rows)
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                let columns = stmt.column_names();

                eyre::Ok((columns, rows))
            });

            let (columns, rows) = tokio::time::timeout(self.query_timeout, future).await???;

            Ok(responses::Query { columns, rows })
        }

        async fn erd(&self) -> color_eyre::Result<responses::Erd> {
            let c = self.conn.clone();
            let table_name = self.table_name.clone();
            tokio::task::spawn_blocking(move || {
                let c = c.lock().expect("could not get lock on connection");

                let mut col_stmt = c.prepare(&format!(
                    r#"
                    SELECT
                        column_name,
                        data_type,
                        is_nullable
                    FROM information_schema.columns
                    WHERE table_name = '{table_name}'
                    ORDER BY ordinal_position
                    "#
                ))?;

                let columns = col_stmt
                    .query_map([], |row| {
                        Ok(responses::ErdColumn {
                            name: row.get::<_, String>(0)?,
                            data_type: row.get::<_, String>(1)?,
                            nullable: row.get::<_, String>(2)? == "YES",
                            is_primary_key: false,
                        })
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                let tables = vec![responses::ErdTable {
                    name: table_name,
                    columns,
                }];

                // CSV files don't have relationships
                eyre::Ok(responses::Erd {
                    tables,
                    relationships: Vec::new(),
                })
            })
            .await?
        }
    }
}

mod clickhouse {
    use clickhouse::Client;
    use color_eyre::eyre::OptionExt;
    use std::time::Duration;

    use crate::{
        Database, ROWS_PER_PAGE,
        responses::{self, Count},
    };

    #[derive(Clone)]
    pub struct Db {
        conn: Client,
        database: String,
        _query_timeout: Duration,
    }

    #[derive(serde::Deserialize, clickhouse::Row, Debug)]
    pub struct ClickhouseCount {
        pub name: String,
        pub count: i64,
    }

    impl Db {
        pub async fn open(
            url: String,
            user: String,
            password: String,
            database: String,
            query_timeout: Duration,
        ) -> color_eyre::Result<Self> {
            let conn = Client::default()
                .with_url(url)
                .with_user(user)
                .with_password(password)
                .with_database(&database);

            let tables: i32 = conn
                .query(
                    r#"
            SELECT count(*)
            FROM system.tables
            WHERE database = currentDatabase()
                    "#,
                )
                .fetch_one()
                .await?;

            tracing::info!(
                "found {tables} table{} in {database}",
                if tables == 1 { "" } else { "s" }
            );

            Ok(Self {
                conn,
                database,
                _query_timeout: query_timeout,
            })
        }
    }

    impl Database for Db {
        async fn overview(&self) -> color_eyre::Result<responses::Overview> {
            let file_name = self.database.to_owned();

            let db_size = String::new();
            let modified = None;
            let created = None;

            let tables: i32 = self
                .conn
                .query(
                    r#"
            SELECT count(*)
            FROM system.tables
            WHERE database = currentDatabase()
                    "#,
                )
                .fetch_one()
                .await?;

            let indexes: i32 = self
                .conn
                .query(
                    r#"
            SELECT count(*)
            FROM system.columns
            WHERE database = currentDatabase() 
            AND (is_in_primary_key = true OR is_in_sorting_key = true)
                    "#,
                )
                .fetch_one()
                .await?;

            let triggers: i32 = 0;

            let views: i32 = self
                .conn
                .query(
                    r#"
            SELECT count(*)
            FROM system.tables
            WHERE database = currentDatabase() 
            AND engine = 'View'
                    "#,
                )
                .fetch_one()
                .await?;

            let mut row_counts = self
                .conn
                .query(
                    r#"
            SELECT name
            FROM system.tables
            WHERE database = currentDatabase()
                    "#,
                )
                .fetch_all()
                .await?
                .into_iter()
                .map(|name: String| Count { name, count: 0 })
                .collect::<Vec<_>>();

            for count in row_counts.iter_mut() {
                count.count = self
                    .conn
                    .query(&format!("SELECT count(*) FROM {}", count.name))
                    .fetch_one()
                    .await?;
            }

            row_counts.sort_by(|a, b| b.count.cmp(&a.count));

            let mut column_counts = self
                .conn
                .query(
                    r#"
            SELECT table AS name, count() AS count
            FROM system.columns
            WHERE database = currentDatabase()
            GROUP BY table
                    "#,
                )
                .fetch_all::<ClickhouseCount>()
                .await?;

            column_counts.sort_by(|a, b| b.count.cmp(&a.count));

            let mut index_counts = self
                .conn
                .query(
                    r#"
            SELECT name
            FROM system.tables
            WHERE database = currentDatabase()
                    "#,
                )
                .fetch_all()
                .await?
                .into_iter()
                .map(|name: String| Count { name, count: 0 })
                .collect::<Vec<_>>();

            for count in index_counts.iter_mut() {
                count.count = self
                    .conn
                    .query(
                        r#"
                SELECT count(*)
                FROM system.columns
                WHERE database = currentDatabase() 
                AND table = ?
                AND (is_in_primary_key = true OR is_in_sorting_key = true)
                        "#,
                    )
                    .bind(&count.name)
                    .fetch_one()
                    .await?;
            }

            index_counts.sort_by(|a, b| b.count.cmp(&a.count));

            Ok(responses::Overview {
                file_name,
                sqlite_version: None,
                db_size,
                created,
                modified,
                tables,
                indexes,
                triggers,
                views,
                row_counts,
                column_counts: column_counts
                    .into_iter()
                    .map(|ClickhouseCount { name, count }| Count {
                        name,
                        count: count as i32,
                    })
                    .collect(),
                index_counts,
            })
        }

        async fn tables(&self) -> color_eyre::Result<responses::Tables> {
            let mut tables = self
                .conn
                .query(
                    r#"
            SELECT name
            FROM system.tables
            WHERE database = currentDatabase()
                    "#,
                )
                .fetch_all()
                .await?
                .into_iter()
                .map(|name: String| Count { name, count: 0 })
                .collect::<Vec<_>>();

            for count in tables.iter_mut() {
                count.count = self
                    .conn
                    .query(&format!("SELECT count(*) FROM {}", count.name))
                    .fetch_one()
                    .await?;
            }

            tables.sort_by_key(|r| r.count);

            Ok(responses::Tables { tables })
        }

        async fn table(&self, name: String) -> color_eyre::Result<responses::Table> {
            let sql: String = self
                .conn
                .query(
                    r#"
            SELECT create_table_query
            FROM system.tables
            WHERE database = currentDatabase()
            AND table = ?
                    "#,
                )
                .bind(&name)
                .fetch_one()
                .await?;

            let row_count = self
                .conn
                .query(&format!("SELECT count(*) FROM {name}"))
                .fetch_one()
                .await?;

            let table_size = self
                .conn
                .query(
                    r#"
            SELECT
            formatReadableSize(sum(bytes)) as size
            FROM system.parts WHERE table = ?
                    "#,
                )
                .bind(&name)
                .fetch_one::<String>()
                .await?;

            let index_count: i32 = self
                .conn
                .query(
                    r#"
            SELECT count(*)
            FROM system.columns
            WHERE database = currentDatabase() 
            AND table = ?
            AND (is_in_primary_key = true OR is_in_sorting_key = true)
                    "#,
                )
                .bind(&name)
                .fetch_one()
                .await?;

            let column_count: i32 = self
                .conn
                .query(
                    r#"
            SELECT count() AS count
            FROM system.columns
            WHERE database = currentDatabase()
            AND table =  ?
                    "#,
                )
                .bind(&name)
                .fetch_one()
                .await?;

            Ok(responses::Table {
                name,
                sql: Some(sql),
                row_count,
                table_size,
                index_count,
                column_count,
            })
        }

        async fn table_data(
            &self,
            name: String,
            page: i32,
        ) -> color_eyre::Result<responses::TableData> {
            let mut columns = self
                .conn
                .query(
                    r#"
            SELECT name
            FROM system.columns
            WHERE database = currentDatabase()
            AND table = ?
                    "#,
                )
                .bind(&name)
                .fetch_all::<String>()
                .await?;
            columns.truncate(5);

            let first_column = columns.first().ok_or_eyre("no first column found")?;

            let offset = (page - 1) * ROWS_PER_PAGE;
            let _sql = format!(
                r#"
            SELECT {} FROM {name}
            ORDER BY {first_column}
            LIMIT {ROWS_PER_PAGE}
            OFFSET {offset}
                "#,
                columns.join(",")
            );

            Ok(responses::TableData {
                columns,
                rows: Vec::new(),
            })
        }

        async fn tables_with_columns(&self) -> color_eyre::Result<responses::TablesWithColumns> {
            let table_names = self
                .conn
                .query(
                    r#"
            SELECT name
            FROM system.tables
            WHERE database = currentDatabase()
                    "#,
                )
                .fetch_all::<String>()
                .await?;

            let mut tables = Vec::with_capacity(table_names.len());
            for table_name in table_names {
                let mut columns = self
                    .conn
                    .query(
                        r#"
                        SELECT name
                        FROM system.columns
                        WHERE database = currentDatabase()
                        AND table = ?
                        "#,
                    )
                    .bind(&table_name)
                    .fetch_all::<String>()
                    .await?;
                columns.truncate(5);

                tables.push(responses::TableWithColumns {
                    table_name,
                    columns,
                });
            }

            Ok(responses::TablesWithColumns { tables })
        }

        async fn query(&self, _query: String) -> color_eyre::Result<responses::Query> {
            Ok(responses::Query {
                columns: Vec::new(),
                rows: Vec::new(),
            })
        }

        async fn erd(&self) -> color_eyre::Result<responses::Erd> {
            // Get all tables with columns from system.columns
            #[derive(clickhouse::Row, serde::Deserialize)]
            struct ColumnInfo {
                table: String,
                name: String,
                #[serde(rename = "type")]
                data_type: String,
                is_in_primary_key: u8,
            }

            let column_rows = self
                .conn
                .query(
                    r#"
                    SELECT
                        table,
                        name,
                        type,
                        is_in_primary_key
                    FROM system.columns
                    WHERE database = currentDatabase()
                    ORDER BY table, position
                    "#,
                )
                .fetch_all::<ColumnInfo>()
                .await?;

            let mut table_map: std::collections::HashMap<String, Vec<responses::ErdColumn>> =
                std::collections::HashMap::new();

            for row in column_rows {
                let column = responses::ErdColumn {
                    name: row.name,
                    data_type: row.data_type,
                    nullable: false, // ClickHouse handles nullability differently via Nullable(T)
                    is_primary_key: row.is_in_primary_key == 1,
                };
                table_map.entry(row.table).or_default().push(column);
            }

            let tables: Vec<responses::ErdTable> = table_map
                .into_iter()
                .map(|(name, columns)| responses::ErdTable { name, columns })
                .collect();

            // ClickHouse doesn't support foreign keys, return empty relationships
            Ok(responses::Erd {
                tables,
                relationships: Vec::new(),
            })
        }
    }
}

mod mssql {
    use std::{sync::Arc, time::Duration};

    use color_eyre::eyre::OptionExt;
    use futures::{StreamExt, TryStreamExt};
    use tiberius::{Client, Config};
    use tokio::{net::TcpStream, sync::Mutex};

    use crate::{
        Database, ROWS_PER_PAGE,
        helpers::{self, mssql_value_to_json},
        responses::{self, Count},
    };

    #[derive(Clone)]
    pub struct Db {
        client: Arc<Mutex<Client<tokio_util::compat::Compat<TcpStream>>>>,
        query_timeout: Duration,
    }

    impl Db {
        pub async fn open(connection: String, query_timeout: Duration) -> color_eyre::Result<Self> {
            use tokio_util::compat::TokioAsyncWriteCompatExt;

            let config = Config::from_ado_string(&connection)?;
            let tcp = tokio::net::TcpStream::connect(config.get_addr()).await?;
            tcp.set_nodelay(true)?;

            let mut client = Client::connect(config, tcp.compat_write()).await?;

            let tables: i32 = client
                .query(
                    r#"
                SELECT COUNT(*) AS count
                FROM sys.tables t
                JOIN sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = SCHEMA_NAME();
                    "#,
                    &[],
                )
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get("count"))
                .ok_or_eyre("couldn't count tables")?;

            tracing::info!(
                "found {tables} table{} in {connection}",
                if tables == 1 { "" } else { "s" }
            );

            Ok(Self {
                client: Arc::new(Mutex::new(client)),
                query_timeout,
            })
        }
    }

    impl Database for Db {
        async fn overview(&self) -> color_eyre::Result<responses::Overview> {
            let mut client = self.client.lock().await;

            let file_name = client
                .query(
                    r#"
                SELECT DB_NAME() AS name;
                    "#,
                    &[],
                )
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get::<&str, &str>("name").map(ToOwned::to_owned))
                .ok_or_eyre("couldn't get database name")?;

            let db_size: i64 = client
                .query(
                    r#"
                SELECT SUM(a.total_pages * 8) AS size_kb
                FROM sys.tables t
                JOIN sys.indexes i ON t.object_id = i.object_id
                JOIN sys.partitions p ON i.object_id = p.object_id AND i.index_id = p.index_id
                JOIN sys.allocation_units a ON p.partition_id = a.container_id;
                    "#,
                    &[],
                )
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get("size_kb"))
                .ok_or_eyre("couldn't get database size")?;
            let db_size = helpers::format_size(db_size as f64);

            let modified = None;
            let created = None;

            let tables: i32 = client
                .query(
                    r#"
                SELECT COUNT(*) AS count
                FROM sys.tables t
                JOIN sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = SCHEMA_NAME();
                    "#,
                    &[],
                )
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get("count"))
                .ok_or_eyre("couldn't count tables")?;

            let indexes: i32 = client
                .query(
                    r#"
                SELECT COUNT(*) AS count
                FROM sys.stats s
                JOIN sys.tables t ON s.object_id = t.object_id
                JOIN sys.schemas sc ON t.schema_id = sc.schema_id
                WHERE sc.name = SCHEMA_NAME();
                    "#,
                    &[],
                )
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get("count"))
                .ok_or_eyre("couldn't count indexes")?;

            let triggers: i32 = client
                .query(
                    r#"
                SELECT COUNT(*) AS count
                FROM sys.triggers t
                JOIN sys.tables tbl ON t.parent_id = tbl.object_id
                JOIN sys.schemas s ON tbl.schema_id = s.schema_id
                WHERE s.name = SCHEMA_NAME();
                    "#,
                    &[],
                )
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get("count"))
                .ok_or_eyre("couldn't count triggers")?;

            let views: i32 = client
                .query(
                    r#"
                SELECT COUNT(*) AS count
                FROM sys.views v
                JOIN sys.schemas s ON v.schema_id = s.schema_id
                WHERE s.name = SCHEMA_NAME();
                    "#,
                    &[],
                )
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get("count"))
                .ok_or_eyre("couldn't count triggers")?;

            let mut row_counts = client
                .query(
                    r#"
                SELECT t.name AS name
                FROM sys.tables t
                JOIN sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = SCHEMA_NAME();
                    "#,
                    &[],
                )
                .await?
                .into_row_stream()
                .try_filter_map(|row| {
                    let out = Ok(row.get::<&str, &str>("name").map(ToOwned::to_owned));
                    async { out }
                })
                .map_ok(|name| Count { name, count: 0 })
                .filter_map(|count| async { count.ok() })
                .collect::<Vec<_>>()
                .await;

            for count in row_counts.iter_mut() {
                let sql = format!("SELECT count(*) AS count FROM {}", count.name);

                count.count = client
                    .query(sql, &[])
                    .await?
                    .into_row()
                    .await?
                    .and_then(|row| row.get("count"))
                    .ok_or_eyre("couldn't count rows")?;
            }

            row_counts.sort_by(|a, b| b.count.cmp(&a.count));

            let mut column_counts = client
                .query(
                    r#"
                SELECT t.name AS name
                FROM sys.tables t
                JOIN sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = SCHEMA_NAME();
                    "#,
                    &[],
                )
                .await?
                .into_row_stream()
                .try_filter_map(|row| {
                    let out = Ok(row.get::<&str, &str>("name").map(ToOwned::to_owned));
                    async { out }
                })
                .map_ok(|name| Count {
                    name: name.to_owned(),
                    count: 0,
                })
                .filter_map(|count| async { count.ok() })
                .collect::<Vec<_>>()
                .await;

            for count in column_counts.iter_mut() {
                count.count = client
                    .query(
                        r#"
                    SELECT COUNT(*) AS count
                    FROM sys.columns c
                    JOIN sys.tables t ON c.object_id = t.object_id
                    JOIN sys.schemas s ON t.schema_id = s.schema_id
                    WHERE s.name = SCHEMA_NAME()
                    AND t.name = @P1;
                        "#,
                        &[&count.name],
                    )
                    .await?
                    .into_row()
                    .await?
                    .and_then(|row| row.get("count"))
                    .ok_or_eyre("couldn't count columns")?;
            }

            column_counts.sort_by(|a, b| b.count.cmp(&a.count));

            let mut index_counts = client
                .query(
                    r#"
                SELECT t.name AS name
                FROM sys.tables t
                JOIN sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = SCHEMA_NAME();
                    "#,
                    &[],
                )
                .await?
                .into_row_stream()
                .try_filter_map(|row| {
                    let out = Ok(row.get::<&str, &str>("name").map(ToOwned::to_owned));
                    async { out }
                })
                .map_ok(|name| Count {
                    name: name.to_owned(),
                    count: 0,
                })
                .filter_map(|count| async { count.ok() })
                .collect::<Vec<_>>()
                .await;

            for count in index_counts.iter_mut() {
                count.count = client
                    .query(
                        r#"
                    SELECT COUNT(*) AS count
                    FROM sys.stats s
                    JOIN sys.tables t ON s.object_id = t.object_id
                    JOIN sys.schemas sc ON t.schema_id = sc.schema_id
                    WHERE sc.name = SCHEMA_NAME()
                    AND t.name = @P1;
                        "#,
                        &[&count.name],
                    )
                    .await?
                    .into_row()
                    .await?
                    .and_then(|row| row.get("count"))
                    .ok_or_eyre("couldn't count indexes")?;
            }

            index_counts.sort_by(|a, b| b.count.cmp(&a.count));

            Ok(responses::Overview {
                file_name,
                sqlite_version: None,
                db_size,
                created,
                modified,
                tables,
                indexes,
                triggers,
                views,
                row_counts,
                column_counts,
                index_counts,
            })
        }

        async fn tables(&self) -> color_eyre::Result<responses::Tables> {
            let mut client = self.client.lock().await;

            let mut tables = client
                .query(
                    r#"
                SELECT t.name AS name
                FROM sys.tables t
                JOIN sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = SCHEMA_NAME();
                    "#,
                    &[],
                )
                .await?
                .into_row_stream()
                .try_filter_map(|row| {
                    let out = Ok(row.get::<&str, &str>("name").map(ToOwned::to_owned));
                    async { out }
                })
                .map_ok(|name| Count { name, count: 0 })
                .filter_map(|count| async { count.ok() })
                .collect::<Vec<_>>()
                .await;

            for count in tables.iter_mut() {
                let sql = format!("SELECT count(*) AS count FROM {}", count.name);

                count.count = client
                    .query(sql, &[])
                    .await?
                    .into_row()
                    .await?
                    .and_then(|row| row.get("count"))
                    .ok_or_eyre("couldn't count rows")?;
            }

            tables.sort_by(|a, b| b.count.cmp(&a.count));

            Ok(responses::Tables { tables })
        }

        async fn table(&self, name: String) -> color_eyre::Result<responses::Table> {
            let mut client = self.client.lock().await;

            let row_count: i32 = client
                .query(format!("SELECT count(*) AS count FROM {name}"), &[])
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get("count"))
                .ok_or_eyre("couldn't count rows")?;

            let table_size: i64 = client
                .query(
                    r#"
                SELECT SUM(a.total_pages) * 8 AS size_kb
                FROM sys.partitions p
                JOIN sys.allocation_units a ON p.partition_id = a.container_id
                JOIN sys.tables t ON p.object_id = t.object_id
                JOIN sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = SCHEMA_NAME() AND t.name = @P1;
                    "#,
                    &[&name],
                )
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get("size_kb"))
                .ok_or_eyre("couldn't count rows")?;
            let table_size = helpers::format_size(table_size as f64);

            let index_count: i32 = client
                .query(
                    r#"
                SELECT COUNT(*) AS count
                FROM sys.stats s
                JOIN sys.tables t ON s.object_id = t.object_id
                JOIN sys.schemas sc ON t.schema_id = sc.schema_id
                WHERE sc.name = SCHEMA_NAME() AND t.name = @P1;
                    "#,
                    &[&name],
                )
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get("count"))
                .ok_or_eyre("couldn't count indexes")?;

            let column_count: i32 = client
                .query(
                    r#"
                SELECT COUNT(*) AS count
                FROM sys.columns c
                JOIN sys.tables t ON c.object_id = t.object_id
                JOIN sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = SCHEMA_NAME() AND t.name = @P1;
                    "#,
                    &[&name],
                )
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get("count"))
                .ok_or_eyre("couldn't count columns")?;

            Ok(responses::Table {
                name,
                sql: None,
                row_count,
                table_size,
                index_count,
                column_count,
            })
        }

        async fn table_data(
            &self,
            name: String,
            page: i32,
        ) -> color_eyre::Result<responses::TableData> {
            let mut client = self.client.lock().await;

            let first_column: String = client
                .query(
                    r#"
                SELECT TOP 1 column_name AS name
                FROM information_schema.columns
                WHERE table_schema = SCHEMA_NAME()
                AND table_name = @P1;
                    "#,
                    &[&name],
                )
                .await?
                .into_row()
                .await?
                .and_then(|row| row.get::<&str, &str>("name").map(ToOwned::to_owned))
                .ok_or_eyre("couldn't count columns")?;

            let offset = (page - 1) * ROWS_PER_PAGE;
            let sql = format!(
                r#"
            SELECT * FROM "{name}"
            ORDER BY {first_column}
            OFFSET {offset} ROWS FETCH NEXT {ROWS_PER_PAGE} ROWS ONLY;
                "#
            );

            let mut query = client.query(sql, &[]).await?;
            let columns: Vec<String> = query
                .columns()
                .await?
                .unwrap_or_default()
                .iter()
                .map(|c| c.name().to_owned())
                .collect();

            let rows = query
                .into_row_stream()
                .map_ok(|row| row.into_iter().map(mssql_value_to_json).collect::<Vec<_>>())
                .filter_map(|count| async { count.ok() })
                .collect::<Vec<_>>()
                .await;

            Ok(responses::TableData { columns, rows })
        }

        async fn tables_with_columns(&self) -> color_eyre::Result<responses::TablesWithColumns> {
            let mut client = self.client.lock().await;

            let table_names = client
                .query(
                    r#"
                SELECT t.name AS name
                FROM sys.tables t
                JOIN sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = SCHEMA_NAME();
                    "#,
                    &[],
                )
                .await?
                .into_row_stream()
                .try_filter_map(|row| {
                    let out = Ok(row.get::<&str, &str>("name").map(ToOwned::to_owned));
                    async { out }
                })
                .filter_map(|count| async { count.ok() })
                .collect::<Vec<_>>()
                .await;

            let mut tables = Vec::with_capacity(table_names.len());
            for table_name in table_names {
                let columns = client
                    .query(
                        r#"
                    SELECT column_name AS name
                    FROM information_schema.columns
                    WHERE table_schema = SCHEMA_NAME()
                    AND table_name = @P1;
                        "#,
                        &[&table_name],
                    )
                    .await?
                    .into_row_stream()
                    .try_filter_map(|row| {
                        let out = Ok(row.get::<&str, &str>("name").map(ToOwned::to_owned));
                        async { out }
                    })
                    .filter_map(|count| async { count.ok() })
                    .collect::<Vec<_>>()
                    .await;

                tables.push(responses::TableWithColumns {
                    table_name,
                    columns,
                });
            }

            tables.sort_by_key(|t| t.table_name.len());
            Ok(responses::TablesWithColumns { tables })
        }

        async fn query(&self, query: String) -> color_eyre::Result<responses::Query> {
            let mut client = self.client.lock().await;

            let mut query = client.query(query, &[]).await?;
            let columns: Vec<String> = query
                .columns()
                .await?
                .unwrap_or_default()
                .iter()
                .map(|c| c.name().to_owned())
                .collect();

            let rows = tokio::time::timeout(
                self.query_timeout,
                query
                    .into_row_stream()
                    .map_ok(|row| row.into_iter().map(mssql_value_to_json).collect::<Vec<_>>())
                    .filter_map(|count| async { count.ok() })
                    .collect::<Vec<_>>(),
            )
            .await?;

            Ok(responses::Query { columns, rows })
        }

        async fn erd(&self) -> color_eyre::Result<responses::Erd> {
            let mut client = self.client.lock().await;

            // Get all tables with columns
            let column_rows = client
                .query(
                    r#"
                    SELECT
                        t.name AS table_name,
                        c.name AS column_name,
                        ty.name AS data_type,
                        c.is_nullable,
                        CASE WHEN ic.object_id IS NOT NULL THEN 1 ELSE 0 END AS is_primary_key
                    FROM sys.tables t
                    JOIN sys.schemas s ON t.schema_id = s.schema_id
                    JOIN sys.columns c ON t.object_id = c.object_id
                    JOIN sys.types ty ON c.user_type_id = ty.user_type_id
                    LEFT JOIN sys.indexes i ON t.object_id = i.object_id AND i.is_primary_key = 1
                    LEFT JOIN sys.index_columns ic ON i.object_id = ic.object_id
                        AND i.index_id = ic.index_id
                        AND c.column_id = ic.column_id
                    WHERE s.name = SCHEMA_NAME()
                    ORDER BY t.name, c.column_id
                    "#,
                    &[],
                )
                .await?
                .into_row_stream()
                .try_filter_map(|row| {
                    let table_name = row.get::<&str, &str>("table_name").map(ToOwned::to_owned);
                    let column_name = row.get::<&str, &str>("column_name").map(ToOwned::to_owned);
                    let data_type = row.get::<&str, &str>("data_type").map(ToOwned::to_owned);
                    let is_nullable = row.get::<bool, &str>("is_nullable");
                    let is_pk = row.get::<i32, &str>("is_primary_key");

                    let out = match (table_name, column_name, data_type, is_nullable, is_pk) {
                        (Some(tn), Some(cn), Some(dt), Some(nullable), Some(pk)) => {
                            Ok(Some((tn, cn, dt, nullable, pk == 1)))
                        }
                        _ => Ok(None),
                    };
                    async { out }
                })
                .filter_map(|r| async { r.ok() })
                .collect::<Vec<_>>()
                .await;

            let mut table_map: std::collections::HashMap<String, Vec<responses::ErdColumn>> =
                std::collections::HashMap::new();

            for (table_name, column_name, data_type, is_nullable, is_pk) in column_rows {
                let column = responses::ErdColumn {
                    name: column_name,
                    data_type,
                    nullable: is_nullable,
                    is_primary_key: is_pk,
                };
                table_map.entry(table_name).or_default().push(column);
            }

            let tables: Vec<responses::ErdTable> = table_map
                .into_iter()
                .map(|(name, columns)| responses::ErdTable { name, columns })
                .collect();

            // Get foreign key relationships
            let fk_rows = client
                .query(
                    r#"
                    SELECT
                        OBJECT_NAME(fk.parent_object_id) AS from_table,
                        COL_NAME(fkc.parent_object_id, fkc.parent_column_id) AS from_column,
                        OBJECT_NAME(fk.referenced_object_id) AS to_table,
                        COL_NAME(fkc.referenced_object_id, fkc.referenced_column_id) AS to_column
                    FROM sys.foreign_keys fk
                    JOIN sys.foreign_key_columns fkc ON fk.object_id = fkc.constraint_object_id
                    JOIN sys.tables t ON fk.parent_object_id = t.object_id
                    JOIN sys.schemas s ON t.schema_id = s.schema_id
                    WHERE s.name = SCHEMA_NAME()
                    "#,
                    &[],
                )
                .await?
                .into_row_stream()
                .try_filter_map(|row| {
                    let from_table = row.get::<&str, &str>("from_table").map(ToOwned::to_owned);
                    let from_column = row.get::<&str, &str>("from_column").map(ToOwned::to_owned);
                    let to_table = row.get::<&str, &str>("to_table").map(ToOwned::to_owned);
                    let to_column = row.get::<&str, &str>("to_column").map(ToOwned::to_owned);

                    let out = match (from_table, from_column, to_table, to_column) {
                        (Some(ft), Some(fc), Some(tt), Some(tc)) => {
                            Ok(Some(responses::ErdRelationship {
                                from_table: ft,
                                from_column: fc,
                                to_table: tt,
                                to_column: tc,
                            }))
                        }
                        _ => Ok(None),
                    };
                    async { out }
                })
                .filter_map(|r| async { r.ok() })
                .collect::<Vec<_>>()
                .await;

            Ok(responses::Erd {
                tables,
                relationships: fk_rows,
            })
        }
    }
}

mod helpers {
    use duckdb::types::ValueRef as DuckdbValue;
    use libsql::Value as LibsqlValue;
    use tiberius::ColumnData;
    use tokio_rusqlite::types::ValueRef as SqliteValue;

    pub fn format_size(mut size: f64) -> String {
        const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut unit = 0;

        while size >= 1024.0 && unit < UNITS.len() - 1 {
            size /= 1024.0;
            unit += 1;
        }

        format!("{:.2} {}", size, UNITS[unit])
    }

    pub fn rusqlite_value_to_json(v: SqliteValue) -> serde_json::Value {
        use SqliteValue::*;
        match v {
            Null => serde_json::Value::Null,
            Integer(x) => serde_json::json!(x),
            Real(x) => serde_json::json!(x),
            Text(s) => serde_json::Value::String(String::from_utf8_lossy(s).into_owned()),
            Blob(s) => serde_json::json!(s),
        }
    }

    pub fn libsql_value_to_json(v: LibsqlValue) -> serde_json::Value {
        use LibsqlValue::*;
        match v {
            Null => serde_json::Value::Null,
            Integer(x) => serde_json::json!(x),
            Real(x) => serde_json::json!(x),
            Text(s) => serde_json::Value::String(s),
            Blob(s) => serde_json::json!(s),
        }
    }

    pub fn duckdb_value_to_json(v: DuckdbValue) -> serde_json::Value {
        use DuckdbValue::*;
        match v {
            Null => serde_json::Value::Null,
            Boolean(b) => serde_json::Value::Bool(b),
            TinyInt(x) => serde_json::json!(x),
            SmallInt(x) => serde_json::json!(x),
            Int(x) => serde_json::json!(x),
            BigInt(x) => serde_json::json!(x),
            HugeInt(x) => serde_json::json!(x),
            UTinyInt(x) => serde_json::json!(x),
            USmallInt(x) => serde_json::json!(x),
            UInt(x) => serde_json::json!(x),
            UBigInt(x) => serde_json::json!(x),
            Float(x) => serde_json::json!(x),
            Double(x) => serde_json::json!(x),
            Decimal(x) => serde_json::json!(x),
            Text(_) => serde_json::Value::String(v.as_str().unwrap().to_owned()),
            v => serde_json::Value::String(format!("{v:?}")),
        }
    }

    pub fn mssql_value_to_json(v: ColumnData<'static>) -> serde_json::Value {
        use ColumnData::*;
        match v {
            U8(x) => serde_json::json!(x),
            I16(x) => serde_json::json!(x),
            I32(x) => serde_json::json!(x),
            I64(x) => serde_json::json!(x),
            F32(x) => serde_json::json!(x),
            F64(x) => serde_json::json!(x),
            Bit(x) => serde_json::json!(x),
            String(x) => serde_json::json!(x),
            Guid(x) => serde_json::json!(x),
            Binary(x) => serde_json::json!(x),
            Numeric(x) => serde_json::json!(x.map(|x| x.value())),
            Xml(x) => serde_json::json!(x.map(|x| x.to_string())),
            DateTime(x) => serde_json::json!(x.map(|x| format!(
                "{} days and {} second fragments",
                x.days(),
                x.seconds_fragments()
            ))),
            SmallDateTime(x) => serde_json::json!(x.map(|x| format!(
                "{} days and {} second fragments",
                x.days(),
                x.seconds_fragments()
            ))),
            Time(x) => serde_json::json!(x.map(|x| format!(
                "{} increments and {} scale",
                x.increments(),
                x.scale()
            ))),
            Date(x) => serde_json::json!(x.map(|x| format!("{} days", x.days()))),
            DateTime2(x) => serde_json::json!(x.map(|x| format!(
                "{} days, {} increments and {} scale",
                x.date().days(),
                x.time().increments(),
                x.time().scale()
            ))),
            DateTimeOffset(x) => serde_json::json!(x.map(|x| format!(
                "{} days, {} increments, {} scale and {} offset",
                x.datetime2().date().days(),
                x.datetime2().time().increments(),
                x.datetime2().time().scale(),
                x.offset()
            ))),
        }
    }
}

mod responses {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    pub struct Overview {
        pub file_name: String,
        pub db_size: String,
        pub sqlite_version: Option<String>,
        pub created: Option<DateTime<Utc>>,
        pub modified: Option<DateTime<Utc>>,
        pub tables: i32,
        pub indexes: i32,
        pub triggers: i32,
        pub views: i32,
        pub row_counts: Vec<Count>,
        pub column_counts: Vec<Count>,
        pub index_counts: Vec<Count>,
    }

    #[derive(Serialize, Deserialize, clickhouse::Row, Debug)]
    pub struct Count {
        pub name: String,
        pub count: i32,
    }

    #[derive(Serialize)]
    pub struct Tables {
        pub tables: Vec<Count>,
    }

    #[derive(Serialize)]
    pub struct Table {
        pub name: String,
        pub sql: Option<String>,
        pub row_count: i32,
        pub index_count: i32,
        pub column_count: i32,
        pub table_size: String,
    }

    #[derive(Serialize)]
    pub struct TableData {
        pub columns: Vec<String>,
        pub rows: Vec<Vec<serde_json::Value>>,
    }

    #[derive(Serialize)]
    pub struct TablesWithColumns {
        pub tables: Vec<TableWithColumns>,
    }

    #[derive(Serialize)]
    pub struct TableWithColumns {
        pub table_name: String,
        pub columns: Vec<String>,
    }

    #[derive(Serialize)]
    pub struct Query {
        pub columns: Vec<String>,
        pub rows: Vec<Vec<serde_json::Value>>,
    }

    #[derive(Serialize)]
    pub struct Metadata {
        pub version: String,
        pub can_shutdown: bool,
    }

    #[derive(Serialize)]
    pub struct Erd {
        pub tables: Vec<ErdTable>,
        pub relationships: Vec<ErdRelationship>,
    }

    #[derive(Serialize)]
    pub struct ErdTable {
        pub name: String,
        pub columns: Vec<ErdColumn>,
    }

    #[derive(Serialize)]
    pub struct ErdColumn {
        pub name: String,
        pub data_type: String,
        pub nullable: bool,
        pub is_primary_key: bool,
    }

    #[derive(Serialize)]
    pub struct ErdRelationship {
        pub from_table: String,
        pub from_column: String,
        pub to_table: String,
        pub to_column: String,
    }
}

mod handlers {
    use serde::Deserialize;
    use tokio::sync::mpsc;
    use warp::Filter;

    use crate::{Database, rejections, responses::Metadata};

    fn with_state<T: Clone + Send>(
        state: &T,
    ) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone + use<T> {
        let state = state.to_owned();
        warp::any().map(move || state.clone())
    }

    pub fn routes(
        db: impl Database + 'static,
        no_shutdown: bool,
        shutdown_signal: mpsc::Sender<()>,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        let overview = warp::path::end()
            .and(warp::get())
            .and(with_state(&db))
            .and_then(overview);
        let tables = warp::path!("tables")
            .and(warp::get())
            .and(with_state(&db))
            .and_then(tables);
        let table = warp::get()
            .and(with_state(&db))
            .and(warp::path!("tables" / String))
            .and_then(table);
        let data = warp::get()
            .and(with_state(&db))
            .and(warp::path!("tables" / String / "data"))
            .and(warp::query::<PageQuery>())
            .and_then(table_data);
        let autocomplete = warp::path!("autocomplete")
            .and(warp::get())
            .and(with_state(&db))
            .and_then(autocomplete);
        let query = warp::post()
            .and(with_state(&db))
            .and(warp::path!("query"))
            .and(warp::body::json::<QueryBody>())
            .and_then(query);
        let metadata = warp::get()
            .and(warp::path!("metadata"))
            .and(warp::any().map(move || no_shutdown))
            .and_then(metadata);
        let shutdown = warp::post()
            .and(warp::path!("shutdown"))
            .and(with_state(&shutdown_signal))
            .and(warp::any().map(move || no_shutdown))
            .and_then(shutdown);
        let erd = warp::path!("erd")
            .and(warp::get())
            .and(with_state(&db))
            .and_then(erd);

        overview
            .or(tables)
            .or(table)
            .or(autocomplete)
            .or(query)
            .or(data)
            .or(metadata)
            .or(shutdown)
            .or(erd)
    }

    #[derive(Deserialize)]
    pub struct QueryBody {
        pub query: String,
    }

    #[derive(Deserialize)]
    pub struct PageQuery {
        pub page: Option<i32>,
    }

    async fn overview(db: impl Database) -> Result<impl warp::Reply, warp::Rejection> {
        let overview = db.overview().await.map_err(|e| {
            tracing::error!("error while getting database overview: {e}");
            warp::reject::custom(rejections::InternalServerError)
        })?;
        Ok(warp::reply::json(&overview))
    }

    async fn tables(db: impl Database) -> Result<impl warp::Reply, warp::Rejection> {
        let tables = db.tables().await.map_err(|e| {
            tracing::error!("error while getting tables: {e}");
            warp::reject::custom(rejections::InternalServerError)
        })?;
        Ok(warp::reply::json(&tables))
    }

    async fn table(db: impl Database, name: String) -> Result<impl warp::Reply, warp::Rejection> {
        let tables = db.table(name).await.map_err(|e| {
            tracing::error!("error while getting table: {e}");
            warp::reject::custom(rejections::InternalServerError)
        })?;
        Ok(warp::reply::json(&tables))
    }

    async fn table_data(
        db: impl Database,
        name: String,
        data: PageQuery,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let data = db
            .table_data(name, data.page.unwrap_or(1))
            .await
            .map_err(|e| {
                tracing::error!("error while getting table: {e}");
                warp::reject::custom(rejections::InternalServerError)
            })?;
        Ok(warp::reply::json(&data))
    }

    async fn autocomplete(db: impl Database) -> Result<impl warp::Reply, warp::Rejection> {
        let data = db.tables_with_columns().await.map_err(|e| {
            tracing::error!("error while getting autocomplete data: {e}");
            warp::reject::custom(rejections::InternalServerError)
        })?;
        Ok(warp::reply::json(&data))
    }

    async fn query(
        db: impl Database,
        query: QueryBody,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let tables = db
            .query(query.query)
            .await
            .map_err(|_| warp::reject::custom(rejections::InternalServerError))?;
        Ok(warp::reply::json(&tables))
    }

    async fn metadata(no_shutdown: bool) -> Result<impl warp::Reply, warp::Rejection> {
        let version = Metadata {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            can_shutdown: !no_shutdown,
        };

        Ok(warp::reply::json(&version))
    }

    async fn shutdown(
        shutdown_signal: mpsc::Sender<()>,
        no_shutdown: bool,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        if !no_shutdown {
            let res = shutdown_signal.send(()).await;
            tracing::info!("sent shutdown signal: {res:?}");
        }
        Ok("")
    }

    async fn erd(db: impl Database) -> Result<impl warp::Reply, warp::Rejection> {
        let erd = db.erd().await.map_err(|e| {
            tracing::error!("error while getting ERD data: {e}");
            warp::reject::custom(rejections::InternalServerError)
        })?;
        Ok(warp::reply::json(&erd))
    }
}

mod rejections {
    use std::convert::Infallible;

    use warp::{
        http::StatusCode,
        reject::{Reject, Rejection},
        reply::Reply,
    };

    macro_rules! rejects {
        ($($name:ident),*) => {
            $(
                #[derive(Debug)]
                pub struct $name;

                impl Reject for $name {}
            )*
        };
    }

    rejects!(InternalServerError);

    pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
        let code;
        let message;

        if err.is_not_found() {
            code = StatusCode::NOT_FOUND;
            message = "NOT_FOUND";
        } else if err
            .find::<warp::filters::body::BodyDeserializeError>()
            .is_some()
        {
            code = StatusCode::BAD_REQUEST;
            message = "BAD_REQUEST";
        } else if let Some(InternalServerError) = err.find() {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            message = "INTERNAL_SERVER_ERROR";
        } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
            code = StatusCode::METHOD_NOT_ALLOWED;
            message = "METHOD_NOT_ALLOWED";
        } else if err
            .find::<warp::reject::InvalidHeader>()
            .is_some_and(|e| e.name() == warp::http::header::COOKIE)
        {
            code = StatusCode::BAD_REQUEST;
            message = "COOKIE_NOT_AVAILABLE";
        } else {
            tracing::error!("unhandled rejection: {:?}", err);
            code = StatusCode::INTERNAL_SERVER_ERROR;
            message = "UNHANDLED_REJECTION";
        }

        Ok(warp::reply::with_status(message, code))
    }
}
