use chrono::{TimeZone, Utc};
use itertools::Itertools;
use tonic::{Response, Status};
use tracing::info;

use crate::{
    pb::user_stats::{QueryRequest, RawQueryRequest, TimeQuery, User},
    ResponseStream, ServiceResult, UserStatsService,
};

impl UserStatsService {
    pub async fn query(&self, req: QueryRequest) -> ServiceResult<ResponseStream> {
        let time_conditions = req
            .timestamps
            .iter()
            .map(|(col_name, query)| cast_timequery(col_name, query));

        let ids_conditions = req
            .ids
            .iter()
            .map(|(col_name, query)| cast_idquery(col_name, query.ids.as_slice()));

        let conditions = Itertools::merge(time_conditions, ids_conditions).join(" AND ");

        let sql = format!("select email, name from user_stats where {}", conditions);
        info!("UserStatsService SQL: {}", &sql);

        self.raw_query(RawQueryRequest { query: sql }).await
    }

    pub async fn raw_query(&self, req: RawQueryRequest) -> ServiceResult<ResponseStream> {
        info!("sql: {}", &req.query);
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.pool)
            .await
        else {
            let err_msg = format!("Failed to fetch data, Query: {}", &req.query);
            return ServiceResult::Err(Status::internal(err_msg));
        };

        ServiceResult::Ok(Response::new(Box::pin(tokio_stream::iter(
            ret.into_iter().map(Ok),
        ))))
    }
}

fn cast_timequery(col_name: &str, time_query: &TimeQuery) -> String {
    match (time_query.after.as_ref(), time_query.before.as_ref()) {
        (Some(t_after), Some(t_before)) => {
            let after = timestamp_to_utc(t_after).to_rfc3339();
            let before = timestamp_to_utc(t_before).to_rfc3339();
            format!("{} between '{}' and '{}'", col_name, after, before)
        }
        (Some(t_after), None) => {
            format!("{} >= {}", col_name, timestamp_to_utc(t_after).to_rfc3339())
        }
        (None, Some(t_before)) => {
            format!(
                "{} <= {}",
                col_name,
                timestamp_to_utc(t_before).to_rfc3339()
            )
        }
        _ => {
            panic!("Invalid TimeQuery");
        }
    }
}

fn cast_idquery(col_name: &str, ids: &[u32]) -> String {
    if ids.is_empty() {
        return "True".to_string();
    }
    format!("array{:?} <@ {}", ids, col_name)
}

/// Cast prost_type::TimeStamp to chrono::UTC
fn timestamp_to_utc(timestamp: &prost_types::Timestamp) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(timestamp.seconds, timestamp.nanos as u32)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use tokio_stream::StreamExt;

    use crate::{
        pb::user_stats::QueryRequestBuilder,
        test_utils::{new_id_query, new_timequery, to_timestamp},
    };

    use super::*;

    #[tokio::test]
    async fn raw_query_should_work() {
        let (_tdb, svc) = UserStatsService::new_for_test().await;
        let req = RawQueryRequest {
            query: "select email, name from user_stats where name = '高菲霞'".to_string(),
        };
        let res = svc.raw_query(req).await.unwrap();
        let res = res.into_inner().collect::<Vec<_>>().await;
        assert_eq!(res.len(), 1);
        let user = res[0].as_ref().unwrap();
        assert_eq!(user.name, "高菲霞");
        assert_eq!(user.email, "brenna.elx4os2u@example.net")
    }

    #[tokio::test]
    async fn query_should_work() {
        let (_tdb, svc) = UserStatsService::new_for_test().await;
        let query = QueryRequestBuilder::default()
            .timestamp((
                "created_at".to_string(),
                new_timequery(to_timestamp(120), to_timestamp(0)),
            ))
            .timestamp((
                "last_visited_at".to_string(),
                new_timequery(to_timestamp(30), to_timestamp(0)),
            ))
            .id((
                "viewed_but_not_started".to_string(),
                new_id_query(&[252790]),
            ))
            .build()
            .unwrap();

        let res = svc.query(query).await.unwrap();
        let res = res.into_inner().collect::<Vec<_>>().await;
        assert_eq!(res.len(), 16);
        println!("res{:?}", res);
    }
}
