# 預定系統的核心

- Feature Name: core-reservation
- Start Date: 2023-07-26 22:12:59

## Summary

預留系統核心，解決資源預留一段時間的問題。我們利用 postgres EXCLUDE 約束來確保在給定時間只能為給定資源進行一次預留。

## Motivation

讓我可以練習這個概念以及我該如何處理這個類型的問題。在這個項目中我可以學習gRPC接口、數據連接池、預訂問題等。

事實上，我們需要一個通用的解決方案來處理各種預訂：1）日曆預訂； 2）酒店/房間預訂； 3）會議室預訂；等等，我們應該有一個可以被這些系統使用的通用解決方案。

## Guide-level explanation

首先，我們分為兩個部分，
- Networking Layer： 我需要讓用戶可以對Calender service進行基本的「預定」、「保留」、「刪除」，而同時將這些資訊包裝成request對Reservation service互動並且回傳結果，而這裡我們採用gRPC interface。

- Core Logic： 使用db connection pool對sqlx搭建的資料庫進行連接和確認。

### Service interface

I want try to use gRPC interface to make connection with real world. let me familiar with this interface.

```proto
note: 假設有兩個不同語言體系的人需要一起完成一篇作文，所以兩個人決定使用同一張A4稿紙(gRPC)作為他們的共同紙張，並且透過英語(protobuf)作為陳述方式。


enum ReservationStatus {
    UNKNOWN = 0;
    PENDING = 1;
    CONFIRMED = 2;
    BLOCKED = 3;
}

enum ReservationUpdateType {
    UNKNOWN = 0;
    CREATE = 1;
    UPDATE = 2;
    DELETE = 3;
}

message Reservation {
    string id = 1;
    string user_id = 2;
    ReservationStatus status = 3;

    // resource reservation window
    string resource_id = 4;
    google.protobuf.Timestamp start = 5;
    google.protobuf.Timestamp end = 6;

    // extra note
    string note = 7;
}

message ReserveRequest {
    Reservation reservation = 1;
}

message ReserveResponse {
    Reservation reservation = 1;
}

message UpdateRequest {
    string note = 2;
}

message UpdateResponse {
    Reservation reservation = 1;
}

message ConfirmRequest {
    string id = 1;
}

message ConfirmResponse {
    Reservation reservation = 1;
}

message CancelRequest {
    string id = 1;
}

message CancelResponse {
    Reservation reservation = 1;
}

message GetRequest {
    string id = 1;
}

message GetResponse {
    Reservation reservation = 1;
}

message QueryRequest {
    string resource_id = 1;
    string user_id = 2;
    // use status to filter result. If UNKNOWN, return all reservations
    ReservationStatus status = 3;
    google.protobuf.Timestamp start = 4;
    google.protobuf.Timestamp end = 5;
}

message ListenRequest {}
message ListenResponse {
    int8 op = 1;
    Reservation reservation = 2;
}

service ReservationService {
    rpc reserve(ReserveRequest) returns (ReserveResponse);
    rpc confirm(ConfirmRequest) returns (ConfirmResponse);
    rpc update(UpdateRequest) returns (UpdateResponse);
    rpc cancel(CancelRequest) returns (CancelResponse);
    rpc get(GetRequest) returns (GetResponse);
    rpc query(QueryRequest) returns (stream Reservation);
    // another system could monitor newly added/confirmed/cancelled reservations
    rpc listen(ListenRequest) returns (stream Reservation);
}
```

### Database Schema

note: 在這部分，我學會了如何避免同時預約相同的資源。我可以使用 PostgreSQL 的 EXCLUDE 約束來確保預約之間不會產生衝突。

We use postgres as the database. Below is the schema:

```sql
CREATE SCHEMA rsvp;
CREATE TYPE rsvp.reservation_status AS ENUM ('unknown', 'pending', 'confirmed', 'blocked');
CREATE TYPE rsvp.reservation_update_type AS ENUM ('unknown', 'create', 'update', 'delete');

CREATE TABLE rsvp.reservations (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id VARCHAR(64) NOT NULL,
    status rsvp.reservation_status NOT NULL DEFAULT 'pending',

    resource_id VARCHAR(64) NOT NULL,
    timespan TSTZRANGE NOT NULL,

    note TEXT,

    CONSTRAINT reservations_pkey PRIMARY KEY (id),
    CONSTRAINT reservations_conflict EXCLUDE USING gist (resource_id WITH =, timespan WITH &&)
    -- 當resource_id是一樣的也就是說同一個預定者的時間區段，就要去比對timespan是否是有"Overlap"的狀況發生
);
CREATE INDEX reservations_resource_id_idx ON rsvp.reservations (resource_id);
CREATE INDEX reservations_user_id_idx ON rsvp.reservations (user_id);

-- 如果 uid 和 rid 都是 null，那麼函數將返回在指定時間範圍內的所有預約。
-- 如果只有 uid 是 null，那麼函數將返回指定資源在指定時間範圍內的所有預約。
-- 如果只有 rid 是 null，那麼函數將返回指定用戶在指定時間範圍內的所有預約。
-- 如果 uid 和 rid 都不是 null，那麼函數將返回指定用戶在指定資源和時間範圍內的所有預約。
CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during TSTZRANGE) RETURNS TABLE (LIKE rsvp.reservations) AS $$ $$ LANGUAGE plpgsql;

-- 以防萬一說一下，這是一個函數我們可以在資料庫端寫上述的需求，也可以在資料庫端寫這樣的需求，在這邊寫就是會比較高效一點，但我還是保留。

-- resevation change queue
CREATE TABLE rsvp.reservation_changes (
    id SERIAL NOT NULL,
    reservation_id uuid NOT NULL,
    op rsvp.reservation_update_type NOT NULL
);

-- trigger for add/update/delete a reservation
CREATE OR REPLACE FUNCTION rsvp.reservations_trigger() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- update reservation_changes
        INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (NEW.id, 'create');
    ELSIF TG_OP = 'UPDATE' THEN
        -- if status changed, update reservation_changes
        IF OLD.status <> NEW.status THEN
            INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (NEW.id, 'update');
        END IF;
    ELSIF TG_OP = 'DELETE' THEN
        -- update reservation_changes
        INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (OLD.id, 'delete');
    END IF;
    -- notify a channel called reservation_update
    NOTIFY reservation_update;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservations_trigger
    AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservations
    FOR EACH ROW EXECUTE PROCEDURE rsvp.reservations_trigger();
```

## Reference-level explanation

TBD

## Drawbacks

N/A

## Rationale and alternatives

N/A

## Prior art

TODO: refactor this zone

- pgcli
- database schema
- sqlx
- thiserror
- tonic
- tokei
- async-trait
-

## Unresolved questions

- Triggers, while powerful, can occasionally impose performance challenges, such as potentially creating a bottleneck through frequent writes to the 'reservation_changes' table during periods of high concurrency.

## Future possibilities

TBD

## mock questions

- [為何使用protobuffer](mock_questions_notes/why_protobuf.md)
