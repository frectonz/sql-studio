use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use clap::Parser;
use color_eyre::eyre::OptionExt;
use rusqlite::{Connection, OpenFlags};
use warp::Filter;

/// Web based SQLite database browser.
#[derive(Parser, Debug)]
struct Args {
    /// Path to the sqlite database file.
    database: String,

    /// The address to bind to.
    #[arg(short, long, default_value = "127.0.0.1:3030")]
    address: String,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "tracing=info,warp=debug,sqlite_studio=debug".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let args = Args::parse();
    let db = TheDB::open(args.database)?;

    let api = warp::path("api")
        .and(handlers::routes(db))
        .with(warp::cors().allow_any_origin());
    let homepage = warp::get().and_then(statics::homepage);
    let statics = statics::routes();

    let routes = api
        .or(statics)
        .or(homepage)
        .recover(rejections::handle_rejection);

    let address = args.address.parse::<std::net::SocketAddr>()?;
    warp::serve(routes).run(address).await;

    Ok(())
}

mod statics {
    use std::path::Path;

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

    pub async fn homepage() -> Result<impl warp::Reply, warp::Rejection> {
        let file = STATIC_DIR
            .get_file("index.html")
            .ok_or_else(warp::reject::not_found)?;

        let resp = Response::builder()
            .header(CONTENT_TYPE, "text/html")
            .header(CACHE_CONTROL, "max-age=3600, must-revalidate")
            .body(file.contents())
            .unwrap();

        Ok(resp)
    }

    pub fn routes() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        let index = warp::path::end().and_then(homepage);
        let other = warp::path::tail().and_then(send_file);

        index.or(other)
    }
}

#[derive(Clone)]
struct TheDB {
    path: String,
    conn: Arc<Mutex<Connection>>,
}

impl TheDB {
    fn open(path: String) -> color_eyre::Result<Self> {
        let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        Ok(Self {
            path,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    async fn overview(&self) -> color_eyre::Result<responses::Overview> {
        let file_name = Path::new(&self.path)
            .file_name()
            .ok_or_eyre("failed to get file name overview")?
            .to_str()
            .ok_or_eyre("file name is not utf-8")?
            .to_owned();

        let metadata = tokio::fs::metadata(&self.path).await?;

        let sqlite_version = rusqlite::version().to_owned();
        let file_size = helpers::format_size(metadata.len());
        let modified = metadata.modified()?.into();
        let created = metadata.created()?.into();

        let conn = self.conn.clone();
        let (tables, indexes, triggers, views, counts) = tokio::task::spawn_blocking(move || {
            let conn = conn.lock().or_else(|e| {
                tracing::error!("could not get lock: {e}");
                color_eyre::eyre::bail!("could not get lock: {e}")
            })?;

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

            let mut stmt = conn.prepare(r#"SELECT name FROM sqlite_master WHERE type="table""#)?;
            let table_names = stmt.query_map([], |row| row.get::<_, String>(0))?;

            let mut table_counts = HashMap::with_capacity(tables as usize);
            for name in table_names {
                let name = name?;
                let count = conn.query_row(&format!("SELECT count(*) FROM {name}"), (), |r| {
                    r.get::<_, i32>(0)
                })?;

                table_counts.insert(name, count);
            }

            let mut counts = table_counts
                .into_iter()
                .map(|(name, count)| responses::RowCount { name, count })
                .collect::<Vec<_>>();

            counts.sort_by(|a, b| b.count.cmp(&a.count));

            color_eyre::eyre::Ok((tables, indexes, triggers, views, counts))
        })
        .await??;

        Ok(responses::Overview {
            file_name,
            sqlite_version,
            file_size,
            created,
            modified,
            tables,
            indexes,
            triggers,
            views,
            counts,
        })
    }

    async fn tables(&self) -> color_eyre::Result<responses::Tables> {
        let conn = self.conn.clone();
        let tables = tokio::task::spawn_blocking(move || {
            let conn = conn.lock().or_else(|e| {
                tracing::error!("could not get lock: {e}");
                color_eyre::eyre::bail!("could not get lock: {e}")
            })?;

            let mut stmt = conn.prepare(r#"SELECT name FROM sqlite_master WHERE type="table""#)?;
            let table_names = stmt
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<Vec<_>>();

            let mut table_counts = HashMap::with_capacity(table_names.len());
            for name in table_names {
                let name = name?;
                let count = conn.query_row(&format!("SELECT count(*) FROM {name}"), (), |r| {
                    r.get::<_, i32>(0)
                })?;

                table_counts.insert(name, count);
            }

            let mut counts = table_counts
                .into_iter()
                .map(|(name, count)| responses::RowCount { name, count })
                .collect::<Vec<_>>();

            counts.sort_by_key(|r| r.count);

            color_eyre::eyre::Ok(counts)
        })
        .await??;

        Ok(responses::Tables { tables })
    }

    async fn get_table(&self, name: String) -> color_eyre::Result<responses::Table> {
        let conn = self.conn.clone();
        let (name, sql, columns, rows) = tokio::task::spawn_blocking(move || {
            let conn = conn.lock().or_else(|e| {
                tracing::error!("could not get lock: {e}");
                color_eyre::eyre::bail!("could not get lock: {e}")
            })?;

            let sql = conn.query_row(
                r#"
                SELECT sql FROM sqlite_master WHERE type="table" AND name = ?1
            "#,
                [&name],
                |r| r.get::<_, String>(0),
            )?;

            let mut stmt = conn.prepare(&format!("SELECT * FROM {name}"))?;
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
                        let val = helpers::value_to_json(r.get_ref(i)?);
                        rows.push(val);
                    }
                    Ok(rows)
                })?
                .filter_map(|x| x.ok())
                .collect::<Vec<_>>();

            color_eyre::eyre::Ok((name, sql, columns, rows))
        })
        .await??;

        Ok(responses::Table {
            name,
            sql,
            columns,
            rows,
        })
    }
}

mod helpers {
    use rusqlite::types::ValueRef;

    pub fn format_size(size: u64) -> String {
        const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit = 0;

        while size >= 1024.0 && unit < UNITS.len() - 1 {
            size /= 1024.0;
            unit += 1;
        }

        format!("{:.2} {}", size, UNITS[unit])
    }

    pub fn value_to_json(v: ValueRef) -> serde_json::Value {
        match v {
            ValueRef::Null => serde_json::Value::Null,
            ValueRef::Integer(x) => serde_json::json!(x),
            ValueRef::Real(x) => serde_json::json!(x),
            ValueRef::Text(s) => serde_json::Value::String(String::from_utf8_lossy(s).into_owned()),
            ValueRef::Blob(s) => serde_json::json!(s),
        }
    }
}

mod responses {
    use chrono::{DateTime, Utc};
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Overview {
        pub file_name: String,
        pub sqlite_version: String,
        pub file_size: String,
        pub created: DateTime<Utc>,
        pub modified: DateTime<Utc>,
        pub tables: i32,
        pub indexes: i32,
        pub triggers: i32,
        pub views: i32,
        pub counts: Vec<RowCount>,
    }

    #[derive(Serialize)]
    pub struct RowCount {
        pub name: String,
        pub count: i32,
    }

    #[derive(Serialize)]
    pub struct Tables {
        pub tables: Vec<RowCount>,
    }

    #[derive(Serialize)]
    pub struct Table {
        pub name: String,
        pub sql: String,
        pub columns: Vec<String>,
        pub rows: Vec<Vec<serde_json::Value>>,
    }
}

mod handlers {
    use warp::Filter;

    use crate::{rejections, TheDB};

    fn with_state<T: Clone + Send>(
        state: &T,
    ) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
        let state = state.to_owned();
        warp::any().map(move || state.clone())
    }

    pub fn routes(
        db: TheDB,
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

        overview.or(tables).or(table)
    }

    async fn overview(db: TheDB) -> Result<impl warp::Reply, warp::Rejection> {
        let overview = db.overview().await.map_err(|e| {
            tracing::error!("error while getting database overview: {e}");
            warp::reject::custom(rejections::InternalServerError)
        })?;
        Ok(warp::reply::json(&overview))
    }

    async fn tables(db: TheDB) -> Result<impl warp::Reply, warp::Rejection> {
        let tables = db.tables().await.map_err(|e| {
            tracing::error!("error while getting tables: {e}");
            warp::reject::custom(rejections::InternalServerError)
        })?;
        Ok(warp::reply::json(&tables))
    }

    async fn table(db: TheDB, name: String) -> Result<impl warp::Reply, warp::Rejection> {
        let tables = db.get_table(name).await.map_err(|e| {
            tracing::error!("error while getting table: {e}");
            warp::reject::custom(rejections::InternalServerError)
        })?;
        Ok(warp::reply::json(&tables))
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
