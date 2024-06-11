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
        .recover(rejections::handle_rejection)
        .with(warp::cors().allow_any_origin());

    let address = args.address.parse::<std::net::SocketAddr>()?;
    warp::serve(api).run(address).await;

    Ok(())
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
            let table_names = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;

            let mut table_counts = HashMap::with_capacity(tables as usize);
            for name in table_names {
                let name = name?;
                let count = conn.query_row(&format!("SELECT count(*) FROM {name}"), (), |r| {
                    r.get::<_, i32>(0)
                })?;

                table_counts.insert(name, count);
            }

            let counts = table_counts
                .into_iter()
                .map(|(name, count)| responses::RowCount { name, count })
                .collect::<Vec<_>>();

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
}

mod helpers {
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
}

mod handlers {
    use warp::Filter;

    use crate::{rejections, TheDB};

    fn with_state<T: Clone + Send>(
        state: T,
    ) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || state.clone())
    }

    pub fn routes(
        db: TheDB,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path::end()
            .and(warp::get())
            .and(with_state(db))
            .and_then(overview)
    }

    async fn overview(db: TheDB) -> Result<impl warp::Reply, warp::Rejection> {
        let overview = db.overview().await.map_err(|e| {
            tracing::error!("error while getting database overview: {e}");
            warp::reject::custom(rejections::InternalServerError)
        })?;
        Ok(warp::reply::json(&overview))
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
