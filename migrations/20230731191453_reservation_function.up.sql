-- 這是一個使用者查詢的function

CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during TSTZRANGE) RETURNS TABLE (LIKE rsvp.reservations) AS $$
BEGIN
    IF uid IS NULL AND rid IS NULL THEN
    -- 如果 uid 和 rid 都是 null，那麼函數將返回在指定時間範圍內的所有預約。
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE timespan && during;
    ELSIF uid IS NULL THEN
    -- 如果只有 uid 是 null，那麼函數將返回指定資源在指定時間範圍內的所有預約。
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND during @> timespan;
    ELSIF rid IS NULL THEN
    -- 如果只有 rid 是 null，那麼函數將返回指定用戶在指定時間範圍內的所有預約。
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND during @> timespan;
    ELSE
    -- 如果 uid 和 rid 都不是 null，那麼函數將返回指定用戶在指定資源和時間範圍內的所有預約。
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND user_id = uid AND during @> timespan;
    END IF;
END;
$$ LANGUAGE plpgsql;
