use chrono::{DateTime, Days, Utc};
use fake::faker::chrono::zh_cn::DateTimeBetween;
use fake::faker::lorem::zh_cn::Sentence;
use fake::{faker::name::zh_cn::Name, Fake};
use rand::Rng;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::Response;

use crate::{
    pb::metadata::{Content, MaterializeRequest, Publisher},
    MetadataService, ResponseStream, ServiceResult,
};
const CHANNEL_SIZE: usize = 1024;
impl MetadataService {
    pub async fn materialize(
        &self,
        mut requests: impl Stream<Item = Result<MaterializeRequest, tonic::Status>> + Unpin + Send,
    ) -> ServiceResult<ResponseStream> {
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
        while let Some(Ok(request)) = requests.next().await {
            let id = request.id;
            tx.send(Ok(Content::materialize(id))).await.unwrap();
        }
        let stream = Box::pin(ReceiverStream::new(rx));
        Ok(Response::new(stream))
    }
}

impl Content {
    fn materialize(id: u32) -> Self {
        let mut rng = rand::thread_rng();
        let created_at = DateTimeBetween(before(30), Utc::now()).fake::<DateTime<Utc>>();
        let created_at = prost_types::Timestamp {
            seconds: created_at.timestamp(),
            nanos: created_at.timestamp_subsec_nanos() as i32,
        };
        Self {
            id,
            name: Name().fake(),
            description: Sentence(3..20).fake(),
            publishers: (0..rng.gen_range(2..10))
                .map(|_| Publisher::new())
                .collect(),
            url: "https://placehold.co/1600x900".to_string(),
            image: "https://placehold.co/1600x900".to_string(),
            content_type: (0..=4).fake(),
            created_at: Some(created_at),
            views: (0..10000).fake(),
            likes: (0..10000).fake(),
            dislikes: (0..10000).fake(),
        }
    }
}

impl Publisher {
    fn new() -> Self {
        Self {
            id: (10000..200000).fake(),
            name: Name().fake(),
            avatar: "https://placehold.co/400x400".to_string(),
        }
    }
}

fn before(n: u64) -> DateTime<Utc> {
    let now = Utc::now();
    now.checked_sub_days(Days::new(n)).unwrap()
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::config::AppConfig;

    #[tokio::test]
    async fn test_content_materialize() {
        let content = Content::materialize(1);
        println!("{:?}", content);
        assert_eq!(content.id, 1);
    }

    #[tokio::test]
    async fn test_service_materialize() {
        let config = AppConfig::load().unwrap();
        let svc = MetadataService::new(config);
        let stream = tokio_stream::iter(vec![Ok(MaterializeRequest { id: 1 })]);
        let response = svc.materialize(stream).await.unwrap();
        let mut stream = response.into_inner();
        while let Some(content) = stream.next().await {
            println!("{:?}", content.unwrap());
        }
    }
}
