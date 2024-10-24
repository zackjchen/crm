pub mod abi;
pub mod config;
pub mod pb;
use std::{ops::Deref, pin::Pin, sync::Arc};

pub use config::AppConfig;
use pb::user_stats::{
    user_stats_server::{UserStats, UserStatsServer},
    QueryRequest, RawQueryRequest, User,
};
use tokio_stream::Stream;

use tonic::{Request, Response, Status};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct UserStatsService {
    inner: Arc<UserStatsServiceInner>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct UserStatsServiceInner {
    config: AppConfig,
    pool: sqlx::PgPool,
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send>>;
type ServiceResult<T> = Result<Response<T>, Status>;

#[tonic::async_trait]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;
    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        let query = request.into_inner();
        self.query(query).await
    }

    type RawQueryStream = ResponseStream;
    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        let query = request.into_inner();
        self.raw_query(query).await
    }
}

impl UserStatsService {
    pub async fn new(config: AppConfig) -> Self {
        let pool = sqlx::PgPool::connect(&config.server.db_url)
            .await
            .expect("Failed to connect to db");
        let inner = UserStatsServiceInner { config, pool };
        UserStatsService {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> UserStatsServer<UserStatsService> {
        UserStatsServer::new(self)
    }
}

#[cfg(feature = "test-utils")]
pub mod test_utils {
    use std::{path::Path, sync::Arc};

    use chrono::{TimeZone, Utc};
    use prost_types::Timestamp;
    use sqlx::Executor;
    use sqlx_db_tester::TestPg;

    use crate::{
        pb::user_stats::{IdQuery, TimeQuery},
        AppConfig, UserStatsService, UserStatsServiceInner,
    };

    impl UserStatsService {
        pub async fn new_for_test() -> (TestPg, Self) {
            let config = AppConfig::load().expect("Failed to load config");
            let (tdb, pool) = get_test_pool(&config.server.db_url).await;

            let inner = UserStatsServiceInner {
                config: AppConfig::load().expect("Failed to load config"),
                pool,
            };
            (
                tdb,
                Self {
                    inner: Arc::new(inner),
                },
            )
        }
    }

    pub async fn get_test_pool(db_url: &str) -> (TestPg, sqlx::PgPool) {
        let index = db_url.rfind('/');
        let url = match index {
            None => "postgre://zackjchen:postgres@localhost:5432",
            _ => &db_url[..index.unwrap()],
        };
        println!("url:{}", url);

        let tdb = TestPg::new(url.to_string(), Path::new("migrations"));
        println!("pool ---------");

        let pool = tdb.get_pool().await;
        let mut transaction = pool
            .begin()
            .await
            .expect("get_test_pool:Failed to begin transaction");
        let data = include_str!("../fixtures/data.sql").split(';');
        for sql in data {
            transaction
                .execute(sql)
                .await
                .expect("get_test_pool:Failed to execute sql:{}");
        }
        transaction
            .commit()
            .await
            .expect("get_test_pool:Failed to commit transaction");

        (tdb, pool)
    }

    pub fn to_timestamp(days: i64) -> Timestamp {
        let dt = Utc
            .with_ymd_and_hms(2024, 5, 7, 0, 0, 0)
            .unwrap()
            .checked_sub_signed(chrono::Duration::days(days))
            .unwrap();
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
    pub fn new_timequery(after: Timestamp, before: Timestamp) -> TimeQuery {
        TimeQuery {
            after: Some(after),
            before: Some(before),
        }
    }

    pub fn new_id_query(ids: &[u32]) -> IdQuery {
        IdQuery { ids: ids.to_vec() }
    }
}
