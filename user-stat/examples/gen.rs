use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
};

use anyhow::Result;
use chrono::{DateTime, Days, Utc};
use fake::{
    faker::{chrono::en::DateTimeBetween, internet::en::SafeEmail, name::zh_cn::Name},
    Dummy, Fake, Faker,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgHasArrayType, PgPool};

// generate 10000 users and run them in a tx, repeat 500 times

#[derive(Debug, Serialize, Dummy, Deserialize, Clone, PartialEq, Eq)]
struct UserStat {
    #[dummy(faker = "UniqueEmail")]
    email: String,
    #[dummy(faker = "Name()")]
    name: String,
    // enum自动识别
    gender: Gender,
    #[dummy(faker = "DateTimeBetween(before(365*5), before(90))")]
    created_at: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(30), end())")]
    last_visited_at: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(90), end())")]
    last_watched_at: DateTime<Utc>,
    #[dummy(faker = "IntList(50, 100000, 100000)")]
    recent_watched: Vec<i32>,
    #[dummy(faker = "IntList(50, 200000, 100000)")]
    viewed_but_not_started: Vec<i32>,
    #[dummy(faker = "IntList(50, 300000, 100000)")]
    started_but_not_finished: Vec<i32>,
    #[dummy(faker = "IntList(50, 400000, 100000)")]
    finished: Vec<i32>,
    #[dummy(faker = "DateTimeBetween(before(45), end())")]
    last_email_notification: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(15), end())")]
    last_in_app_notification: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(90), end())")]
    last_sms_notification: DateTime<Utc>,
}

impl Hash for UserStat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.email.hash(state)
    }
}

struct UniqueEmail;
impl Dummy<UniqueEmail> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &UniqueEmail, rng: &mut R) -> String {
        let safe: [char; 36] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
            'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
            'y', 'z',
        ];

        let email: String = SafeEmail().fake_with_rng(rng);
        let suffix = nanoid::nanoid!(8, &safe);
        let at = email.find('@').unwrap();
        format!("{}.{}{}", &email[..at], suffix, &email[at..])
    }
}

#[derive(Debug, Serialize, Dummy, Deserialize, Clone, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
enum Gender {
    Female,
    Male,
    Unknown,
}

impl PgHasArrayType for Gender {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        // PgTypeInfo::with_name("gender")
        <&str as PgHasArrayType>::array_type_info()
    }
}

struct IntList(pub i32, pub i32, pub i32);
impl Dummy<IntList> for Vec<i32> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &IntList, rng: &mut R) -> Vec<i32> {
        let (max_size, start, len) = (config.0, config.1, config.2);
        let size = rng.gen_range(0..max_size);
        (0..size)
            .map(|_| rng.gen_range(start..start + len))
            .collect()
    }
}
#[allow(unused)]
async fn bulk_insert(users: HashSet<UserStat>, pool: PgPool) -> Result<()> {
    let mut tx = pool.begin().await?;
    for user in users {
        let _affect_rows = sqlx::query(
            r#"
            INSERT INTO user_stats(email,name,gender,created_at,last_visited_at,last_watched_at,
            recent_watched,viewed_but_not_started,started_but_not_finished,
            finished,last_email_notification,last_in_app_notification,last_sms_notification
            )values($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            "#,
        )
        .bind(user.email)
        .bind(user.name)
        .bind(user.gender)
        .bind(user.created_at)
        .bind(user.last_visited_at)
        .bind(user.last_watched_at)
        .bind(user.recent_watched)
        .bind(user.viewed_but_not_started)
        .bind(user.started_but_not_finished)
        .bind(user.finished)
        .bind(user.last_email_notification)
        .bind(user.last_in_app_notification)
        .bind(user.last_sms_notification)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    }
    tx.commit().await?;
    Ok(())
}

async fn efficient_bulk_insert(users: HashSet<UserStat>, pool: PgPool) -> Result<()> {
    fn to_string_list(list: Vec<Vec<i32>>) -> Vec<String> {
        let mut ret = vec![];
        for vec in list {
            let s = vec
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            ret.push(format!("{{{}}}", s));
        }
        ret
    }
    let mut tx = pool.begin().await?;
    let emails = users.iter().map(|u| u.email.clone()).collect::<Vec<_>>();
    let names = users.iter().map(|u| u.name.clone()).collect::<Vec<_>>();
    let genders = users.iter().map(|u| u.gender.clone()).collect::<Vec<_>>();
    let created_ats = users.iter().map(|u| u.created_at).collect::<Vec<_>>();
    let last_visited_ats = users.iter().map(|u| u.last_visited_at).collect::<Vec<_>>();
    let last_watched_ats = users.iter().map(|u| u.last_watched_at).collect::<Vec<_>>();
    let recent_watcheds = users
        .iter()
        .map(|u| u.recent_watched.clone())
        .collect::<Vec<_>>();
    let viewed_but_not_starteds = users
        .iter()
        .map(|u| u.viewed_but_not_started.clone())
        .collect::<Vec<_>>();
    let started_but_not_finisheds = users
        .iter()
        .map(|u| u.started_but_not_finished.clone())
        .collect::<Vec<_>>();
    let finisheds = users.iter().map(|u| u.finished.clone()).collect::<Vec<_>>();
    let last_email_notifications = users
        .iter()
        .map(|u| u.last_email_notification)
        .collect::<Vec<_>>();
    let last_in_app_notifications = users
        .iter()
        .map(|u| u.last_in_app_notification)
        .collect::<Vec<_>>();
    let last_sms_notifications = users
        .iter()
        .map(|u| u.last_sms_notification)
        .collect::<Vec<_>>();

    let recent_watcheds = to_string_list(recent_watcheds);
    let viewed_but_not_starteds = to_string_list(viewed_but_not_starteds);
    let started_but_not_finisheds = to_string_list(started_but_not_finisheds);
    let finisheds = to_string_list(finisheds);
    let affect_rows = sqlx::query(
        r#"
        INSERT INTO user_stats(email,name,gender,created_at,last_visited_at,last_watched_at,recent_watched
        ,viewed_but_not_started,started_but_not_finished,finished,last_email_notification,last_in_app_notification,last_sms_notification
        )select unnest($1::text[]),unnest($2::text[]),unnest($3::gender[]),
        unnest($4::timestamptz[]),unnest($5::timestamptz[]),unnest($6::timestamptz[]),unnest($7::text[])::int[],
        unnest($8::text[])::int[],unnest($9::text[])::int[],unnest($10::text[])::int[],unnest($11::timestamptz[]),unnest($12::timestamptz[]),unnest($13::timestamptz[])
        "#).bind(emails)
        .bind(names)
        .bind(genders)
        .bind(created_ats)
        .bind(last_visited_ats)
        .bind(last_watched_ats)
        .bind(recent_watcheds)
        .bind(viewed_but_not_starteds)
        .bind(started_but_not_finisheds)
        .bind(finisheds)
        .bind(last_email_notifications)
        .bind(last_in_app_notifications)
        .bind(last_sms_notifications)
        .execute(&mut *tx).await?.rows_affected()
        ;
    tx.commit().await?;
    println!("{} rows affected", affect_rows);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    for i in 1..=2 {
        let users: HashSet<_> = (0..10000).map(|_| Faker.fake::<UserStat>()).collect();
        let pool = PgPool::connect("postgres://zackjchen:postgres@localhost:5432/stats").await?;
        println!("Insert Batch {}", i);
        efficient_bulk_insert(users, pool).await?;
        // bulk_insert(users, pool).await?;
        // println!("the {} users inserted", i * 10000);
    }
    let dummy: UserStat = Faker.fake();
    println!("{:?}", dummy);
    Ok(())
}

fn before(days: u64) -> DateTime<Utc> {
    Utc::now().checked_sub_days(Days::new(days)).unwrap()
}
fn end() -> DateTime<Utc> {
    Utc::now()
}
