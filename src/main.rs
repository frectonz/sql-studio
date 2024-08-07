use async_trait::async_trait;
use clap::{Parser, Subcommand};
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
    #[arg(short, long, default_value = "127.0.0.1:3030")]
    address: String,

    /// Timeout duration for queries sent from the query page.
    #[clap(short, long, default_value = "5secs")]
    timeout: humantime::Duration,

    /// Base path to be provided to the UI. [e.g /sql-studio]
    #[clap(short, long)]
    base_path: Option<String>,

    /// Don't open URL in the system browser.
    #[clap(long)]
    no_browser: bool,

    /// Don't show the shutdown button in the UI.
    #[clap(long)]
    no_shutdown: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// A local SQLite database.
    Sqlite {
        /// Path to the sqlite database file. [use the path "preview" if you don't have an sqlite db at
        /// hand, a sample db will be created for you]
        database: String,
    },

    /// A remote SQLite database via libSQL.
    Libsql {
        /// libSQL server address
        url: String,

        /// libSQL authentication token.
        auth_token: String,
    },

    /// A PostgreSQL database.
    Postgres {
        /// PostgreSQL connection url [postgresql://postgres:postgres@127.0.0.1/sample]
        url: String,

        /// PostgreSQL schema
        #[arg(short, long, default_value = "public")]
        schema: String,
    },

    /// A MySQL/MariaDB database.
    Mysql {
        /// mysql connection url [mysql://user:password@localhost/sample]
        url: String,
    },

    /// A local DuckDB database.
    Duckdb {
        /// Path to the the duckdb file.
        database: String,
    },

    /// A ClickHouse database.
    Clickhouse {
        /// Address to the clickhouse server.
        #[arg(default_value = "http://localhost:8123")]
        url: String,

        /// User we want to authentticate as.
        #[arg(default_value = "default")]
        user: String,

        /// Password we want to authentticate with.
        #[arg(default_value = "")]
        password: String,

        /// Name of the database.
        #[arg(default_value = "default")]
        database: String,
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
        Command::Postgres { url, schema } => {
            AllDbs::Postgres(postgres::Db::open(url, schema, args.timeout.into()).await?)
        }
        Command::Mysql { url } => AllDbs::Mysql(mysql::Db::open(url, args.timeout.into()).await?),
        Command::Duckdb { database } => {
            AllDbs::Duckdb(duckdb::Db::open(database, args.timeout.into()).await?)
        }
        Command::Clickhouse {
            url,
            user,
            password,
            database,
        } => AllDbs::Clickhouse(Box::new(
            clickhouse::Db::open(url, user, password, database, args.timeout.into()).await?,
        )),
    };

    let mut index_html = statics::get_index_html()?;
    if let Some(ref base_path) = args.base_path {
        let base = format!(r#"<meta name="BASE_PATH" content="{base_path}" />"#);
        index_html = index_html.replace(r#"<!-- __BASE__ -->"#, &base);
    }

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["Content-Length", "Content-Type"]);

    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

    let api = warp::path("api").and(handlers::routes(db, args.no_shutdown, shutdown_tx));
    let homepage = statics::homepage(index_html.clone());
    let statics = statics::routes();

    let routes = api
        .or(statics)
        .or(homepage)
        .recover(rejections::handle_rejection)
        .with(cors);

    if args.base_path.is_none() && !args.no_browser {
        let res = open::that(format!("http://{}", args.address));
        tracing::info!("tried to open in browser: {res:?}");
    }

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
    use include_dir::{include_dir, Dir};
    use warp::{
        http::{
            header::{CACHE_CONTROL, CONTENT_TYPE},
            Response,
        },
        Filter,
    };

    static STATIC_DIR: Dir = include_dir!("ui/dist");

    async fn send_file(path: warp::path::Tail) -> Result<impl warp::Reply, warp::Rejection> {
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
            .body(file.contents())
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

    pub fn routes() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path::tail().and_then(send_file)
    }
}

#[async_trait]
trait Database: Sized + Clone + Send {
    async fn overview(&self) -> color_eyre::Result<responses::Overview>;

    async fn tables(&self) -> color_eyre::Result<responses::Tables>;

    async fn table(&self, name: String) -> color_eyre::Result<responses::Table>;

    async fn table_data(&self, name: String, page: i32)
        -> color_eyre::Result<responses::TableData>;

    async fn query(&self, query: String) -> color_eyre::Result<responses::Query>;
}

#[derive(Clone)]
enum AllDbs {
    Sqlite(sqlite::Db),
    Libsql(libsql::Db),
    Postgres(postgres::Db),
    Mysql(mysql::Db),
    Duckdb(duckdb::Db),
    Clickhouse(Box<clickhouse::Db>),
}

#[async_trait]
impl Database for AllDbs {
    async fn overview(&self) -> color_eyre::Result<responses::Overview> {
        match self {
            AllDbs::Sqlite(x) => x.overview().await,
            AllDbs::Libsql(x) => x.overview().await,
            AllDbs::Postgres(x) => x.overview().await,
            AllDbs::Mysql(x) => x.overview().await,
            AllDbs::Duckdb(x) => x.overview().await,
            AllDbs::Clickhouse(x) => x.overview().await,
        }
    }

    async fn tables(&self) -> color_eyre::Result<responses::Tables> {
        match self {
            AllDbs::Sqlite(x) => x.tables().await,
            AllDbs::Libsql(x) => x.tables().await,
            AllDbs::Postgres(x) => x.tables().await,
            AllDbs::Mysql(x) => x.tables().await,
            AllDbs::Duckdb(x) => x.tables().await,
            AllDbs::Clickhouse(x) => x.tables().await,
        }
    }

    async fn table(&self, name: String) -> color_eyre::Result<responses::Table> {
        match self {
            AllDbs::Sqlite(x) => x.table(name).await,
            AllDbs::Libsql(x) => x.table(name).await,
            AllDbs::Postgres(x) => x.table(name).await,
            AllDbs::Mysql(x) => x.table(name).await,
            AllDbs::Duckdb(x) => x.table(name).await,
            AllDbs::Clickhouse(x) => x.table(name).await,
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
            AllDbs::Clickhouse(x) => x.table_data(name, page).await,
        }
    }

    async fn query(&self, query: String) -> color_eyre::Result<responses::Query> {
        match self {
            AllDbs::Sqlite(x) => x.query(query).await,
            AllDbs::Libsql(x) => x.query(query).await,
            AllDbs::Postgres(x) => x.query(query).await,
            AllDbs::Mysql(x) => x.query(query).await,
            AllDbs::Duckdb(x) => x.query(query).await,
            AllDbs::Clickhouse(x) => x.query(query).await,
        }
    }
}

mod sqlite {
    use async_trait::async_trait;
    use color_eyre::eyre::OptionExt;
    use std::{
        collections::HashMap,
        path::Path,
        sync::Arc,
        time::{Duration, SystemTime},
    };
    use tokio_rusqlite::{Connection, OpenFlags};

    use crate::{helpers, responses, Database, ROWS_PER_PAGE, SAMPLE_DB};

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

    #[async_trait]
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
                        let count =
                            conn.query_row(&format!("SELECT count(*) FROM '{name}'"), (), |r| {
                                r.get::<_, i32>(0)
                            })?;

                        row_counts.insert(name.to_owned(), count);
                    }

                    let mut row_counts = row_counts
                        .into_iter()
                        .map(|(name, count)| responses::Count { name, count })
                        .collect::<Vec<_>>();

                    row_counts.sort_by(|a, b| b.count.cmp(&a.count));

                    let mut column_counts = HashMap::with_capacity(tables as usize);
                    for name in table_names.iter() {
                        let mut columns = conn.prepare(&format!("PRAGMA table_info('{name}')"))?;
                        let count =
                            columns.query_map((), |r| r.get::<_, String>(1))?.count() as i32;

                        column_counts.insert(name.to_owned(), count);
                    }

                    let mut column_counts = column_counts
                        .into_iter()
                        .map(|(name, count)| responses::Count { name, count })
                        .collect::<Vec<_>>();

                    column_counts.sort_by(|a, b| b.count.cmp(&a.count));

                    let mut index_counts = HashMap::with_capacity(tables as usize);
                    for name in table_names.iter() {
                        let count = conn.query_row(
                            "SELECT count(*) FROM sqlite_master WHERE type='index' AND tbl_name=?1",
                            [name],
                            |r| r.get::<_, i32>(0),
                        )?;

                        let has_primary_key =
                            conn.query_row(&format!("PRAGMA table_info('{name}')"), [], |r| {
                                r.get::<_, i32>(5)
                            })? == 1;

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
                        let count =
                            conn.query_row(&format!("SELECT count(*) FROM '{name}'"), (), |r| {
                                r.get::<_, i32>(0)
                            })?;

                        table_counts.insert(name, count);
                    }

                    let mut counts = table_counts
                        .into_iter()
                        .map(|(name, count)| responses::Count { name, count })
                        .collect::<Vec<_>>();

                    counts.sort_by_key(|r| r.count);

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

                    let row_count =
                        conn.query_row(&format!("SELECT count(*) FROM '{name}'"), (), |r| {
                            r.get::<_, i32>(0)
                        })?;

                    let table_size = if more_than_five {
                        "> 5GB".to_owned()
                    } else {
                        let table_size = conn.query_row(
                            "SELECT SUM(pgsize) FROM dbstat WHERE name = ?1",
                            [&name],
                            |r| r.get::<_, i64>(0),
                        )?;
                        helpers::format_size(table_size as f64)
                    };

                    let index_count = conn.query_row(
                        "SELECT count(*) FROM sqlite_master WHERE type='index' AND tbl_name=?1",
                        [&name],
                        |r| r.get::<_, i32>(0),
                    )?;

                    let has_primary_key =
                        conn.query_row(&format!("PRAGMA table_info('{name}')"), [], |r| {
                            r.get::<_, i32>(5)
                        })? == 1;
                    let index_count = if has_primary_key {
                        index_count + 1
                    } else {
                        index_count
                    };

                    let mut columns = conn.prepare(&format!("PRAGMA table_info('{name}')"))?;
                    let column_count =
                        columns.query_map((), |r| r.get::<_, String>(1))?.count() as i32;

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
                        conn.query_row(&format!("PRAGMA table_info('{name}')"), [], |r| {
                            r.get::<_, String>(1)
                        })?;

                    let offset = (page - 1) * ROWS_PER_PAGE;
                    let mut stmt = conn.prepare(&format!(
                        r#"
                        SELECT *
                        FROM '{name}'
                        ORDER BY {first_column}
                        LIMIT {ROWS_PER_PAGE}
                        OFFSET {offset}
                        "#
                    ))?;
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

        async fn query(&self, query: String) -> color_eyre::Result<responses::Query> {
            let start = SystemTime::now();
            let timeout = self.query_timeout;

            let res = self
                .conn
                .call(move |conn| {
                    let mut stmt = conn.prepare(&query)?;
                    let columns = stmt
                        .column_names()
                        .into_iter()
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>();

                    let columns_len = columns.len();
                    let rows: Result<Vec<_>, _> = stmt
                        .query_map((), |r| {
                            let now = SystemTime::now();
                            if now - timeout >= start {
                                // just used a random error, we just want to bail out
                                return Err(rusqlite::Error::InvalidQuery);
                            }

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
                })
                .await?;

            Ok(res)
        }
    }
}

mod libsql {
    use std::{collections::HashMap, sync::Arc, time::Duration};

    use async_trait::async_trait;
    use color_eyre::eyre::OptionExt;
    use futures::{StreamExt, TryStreamExt};
    use libsql::Builder;

    use crate::{helpers, responses, Database, ROWS_PER_PAGE};

    #[derive(Clone)]
    pub struct Db {
        url: String,
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
                url,
                query_timeout,
                db: Arc::new(db),
            })
        }
    }

    #[async_trait]
    impl Database for Db {
        async fn overview(&self) -> color_eyre::Result<responses::Overview> {
            let file_name = self.url.to_owned();

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
    }
}

mod postgres {
    use std::{sync::Arc, time::Duration};

    use async_trait::async_trait;
    use tokio_postgres::{Client, NoTls};

    use crate::{
        helpers,
        responses::{self, Count},
        Database, ROWS_PER_PAGE,
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
            let (client, connection) = tokio_postgres::connect(&url, NoTls).await?;

            // The connection object performs the actual communication with the database,
            // so spawn it off to run on its own.
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("postgres connection error: {}", e);
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

    #[async_trait]
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
    }
}

mod mysql {
    use std::time::Duration;

    use async_trait::async_trait;
    use color_eyre::eyre::OptionExt;
    use mysql_async::{prelude::*, Pool};

    use crate::{
        helpers,
        responses::{self, Count},
        Database, ROWS_PER_PAGE,
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

    #[async_trait]
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
    }
}

mod duckdb {
    use async_trait::async_trait;
    use color_eyre::eyre;
    use color_eyre::eyre::OptionExt;
    use duckdb::{Config, Connection};
    use std::{
        path::Path,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::{
        helpers,
        responses::{self, Count},
        Database, ROWS_PER_PAGE,
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

    #[async_trait]
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
    }
}

mod clickhouse {
    use async_trait::async_trait;
    use clickhouse::Client;
    use color_eyre::eyre::OptionExt;
    use std::time::Duration;

    use crate::{
        responses::{self, Count},
        Database, ROWS_PER_PAGE,
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

    #[async_trait]
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

        async fn query(&self, _query: String) -> color_eyre::Result<responses::Query> {
            Ok(responses::Query {
                columns: Vec::new(),
                rows: Vec::new(),
            })
        }
    }
}

mod helpers {
    use duckdb::types::ValueRef as DuckdbValue;
    use libsql::Value as LibsqlValue;
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
    pub struct Query {
        pub columns: Vec<String>,
        pub rows: Vec<Vec<serde_json::Value>>,
    }

    #[derive(Serialize)]
    pub struct Metadata {
        pub version: String,
        pub can_shutdown: bool,
    }
}

mod handlers {
    use serde::Deserialize;
    use tokio::sync::mpsc;
    use warp::Filter;

    use crate::{rejections, responses::Metadata, Database};

    fn with_state<T: Clone + Send>(
        state: &T,
    ) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
        let state = state.to_owned();
        warp::any().map(move || state.clone())
    }

    pub fn routes(
        db: impl Database,
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

        overview
            .or(tables)
            .or(table)
            .or(query)
            .or(data)
            .or(metadata)
            .or(shutdown)
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
