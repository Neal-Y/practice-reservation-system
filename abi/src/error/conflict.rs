// 我想返回一個很“清晰”衝突問題給用戶而不是這樣
/*
ConflictReservation("Key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\"))
conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\")).")
test manager::tests::reserve_conflict_should_reject ... ok
*/

// 那由於PgDatabaseError也沒有其他方法提供一個更好的錯誤信息，我們只能自己implement function從 PgDatabaseError中的 "get_raw()"取得一大坨原始資料並解析它。

// TODO: write a parser
