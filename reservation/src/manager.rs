use crate::Rsvp;
use abi::{DbConfig, Error, FilterPager, ReservationId, Validator};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{
    postgres::{types::PgRange, PgPoolOptions},
    PgPool, Row,
};

pub struct ReservationManager {
    pool: PgPool, // sqlx 裡面 postgres pool database connection 使用Arc將各種database connection 分開
}

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: abi::Reservation) -> Result<abi::Reservation, Error> {
        rsvp.validate()?;

        let status = abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timestamp();

        let id:i64 = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id"
        )
        .bind(rsvp.user_id.clone())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(rsvp.note.clone())
        .bind(status.to_string())
        .fetch_one(&self.pool)
        .await?.get(0);

        rsvp.id = id;

        Ok(rsvp)
    }

    // change reservation status
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        // error: code: "42883", message: "operator does not exist: uuid = text"，所以轉Uuid進去查詢語句。
        // let id: Uuid = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;

        id.validate()?;

        // if current status is pending, change status into confirmed, otherwise do nothing
        let rsvp = sqlx::query_as("UPDATE rsvp.reservations SET status = 'confirmed' WHERE id = $1 AND status = 'pending' RETURNING *
        ")
            .bind(id)
            .fetch_one(&self.pool)
            .await.map_err(|_|abi::Error::NotFound)?;

        Ok(rsvp)
    }

    // update note
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, Error> {
        // let id = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;

        id.validate()?;

        let rsvp =
            sqlx::query_as("UPDATE rsvp.reservations SET note = $1 WHERE id = $2 RETURNING *")
                .bind(note)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(rsvp)
    }

    // delete reservation
    async fn delete(&self, id: ReservationId) -> Result<(), Error> {
        // let id = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;

        id.validate()?;

        sqlx::query("DELETE FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // get reservation
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        // let id = Uuid::from_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;

        id.validate()?;

        let rsvp = sqlx::query_as("SELECT * FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(rsvp)
    }

    async fn query(&self, query: abi::ReservationQuery) -> Result<Vec<abi::Reservation>, Error> {
        let user_id = str_to_option(&query.user_id);
        let resource_id = str_to_option(&query.resource_id);
        let range = query.get_timespan();
        let status = abi::ReservationStatus::from_i32(query.status)
            .unwrap_or(abi::ReservationStatus::Pending);
        let rsvps = sqlx::query_as(
            "select * from rsvp.query($1, $2, $3, $4::rsvp.reservation_status, $5, $6, $7)",
        )
        .bind(user_id)
        .bind(resource_id)
        .bind(range)
        .bind(status.to_string())
        .bind(query.page)
        .bind(query.desc)
        .bind(query.page_size)
        .fetch_all(&self.pool)
        .await?;
        Ok(rsvps)
    }

    async fn keyset_query(
        &self,
        filter: abi::FilterById,
    ) -> Result<(FilterPager, Vec<abi::Reservation>), Error> {
        let id = str_to_option(&filter.user_id);
        let resource_id = str_to_option(&filter.resource_id);
        let status = abi::ReservationStatus::from_i32(filter.status)
            .unwrap_or(abi::ReservationStatus::Pending);
        let page_size = if filter.page_size < 10 || filter.page_size > 100 {
            10
        } else {
            filter.page_size
        };

        let rsvps: Vec<abi::Reservation> = sqlx::query_as(
            "select * from rsvp.filter($1, $2, $3::rsvp.reservation_status, $4, $5, $6)",
        )
        .bind(id)
        .bind(resource_id)
        .bind(status.to_string())
        .bind(filter.cursor)
        .bind(filter.desc)
        .bind(page_size)
        .fetch_all(&self.pool)
        .await?;

        // if the first id is current cursor, then we have prev, we start from 1.
        // for example 100 data, cursor is 50, is_desc is false, if the first id is current cursor, which means 1~49 must exist
        // if len - start > page_size, then we have next, we end at len -1

        // ----------------------------------------------------------------------------------------------

        // 依照如果資料的第一筆id是目前的cursor，因為如果沒有那肯定小於cursor。
        let has_prev_page = !rsvps.is_empty() && rsvps[0].id == filter.cursor;
        // 並且如果有前一頁就好比11~20、21~30，那麼start就是1，只有0~10的時候初始值才是0。
        let start = if has_prev_page { 1 } else { 0 };

        // 假設start為0也就是第一頁資料，如果他有下一頁我們的LIMIT是會抓取11筆資料這樣就是11-0>10，那麼就有下一頁。
        // 如果start為1也就是第一頁以後的，
        let has_next_page = (rsvps.len() - start) as i32 > page_size;
        // 因為當初有LIMIT有多取1(為了確定是否還有下一頁)，但是我們並不需要+1的值，所以設定end時就是為長度-1
        // 如果當初沒取到也就是沒有下一頁，那麼end就是長度。
        let end = if has_next_page {
            rsvps.len() - 1
        } else {
            rsvps.len()
        };

        // set the FilterPager，-1是代表沒有下一頁或前一頁。
        let prev = if has_prev_page {
            rsvps[start - 1].id
        } else {
            -1
        };
        let next = if has_next_page { rsvps[end - 1].id } else { -1 };

        // choose the section data
        // TODO: optimize this to avoid use clone
        let result = rsvps[start..end].to_vec();

        let pager = FilterPager {
            next,
            prev,

            // TODO: set the total count
            total: 0,
        };

        Ok((pager, result))

        // ----------------------------------------------------------------------------------------------
    }
}

impl ReservationManager {
    // 創建一個新的 ReservationManager 實例，並將傳入的 pool 綁定到這個實例上。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 從 config 裡面取得資料庫的連線資訊，並且建立一個新的 ReservationManager 實例。
    pub async fn from_config(config: &DbConfig) -> Result<Self, abi::Error> {
        let url = config.url();
        let pool = PgPoolOptions::default()
            .max_connections(config.max_connections)
            .connect(&url)
            .await?;
        Ok(Self::new(pool))
    }
}

fn str_to_option(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
