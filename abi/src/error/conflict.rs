// 我想返回一個很“清晰”衝突問題給用戶而不是這樣
/*
ConflictReservation("Key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\"))
conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\")).")
test manager::tests::reserve_conflict_should_reject ... ok
*/

// 那由於PgDatabaseError也沒有其他方法提供一個更好的錯誤信息，我們只能自己implement function從 PgDatabaseError中的 "get_raw()"取得一大坨原始資料並解析它。

// // TODO: write a parser

use crate::Error;
use chrono::{DateTime, Utc};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str::FromStr, vec};

//? target: to parse the info into below struct

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(conflict) = s.parse::<ReservationConflict>() {
            Ok(ReservationConflictInfo::Parsed(conflict))
        } else {
            Ok(ReservationConflictInfo::Unparsed(s.to_string()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationConflict {
    pub new: ReservationWindow,
    pub old: ReservationWindow,
}

impl FromStr for ReservationConflict {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ParsedInfo::from_str(s)?.try_into()
    }
}

pub struct ParsedInfo {
    new: HashMap<String, String>,
    old: HashMap<String, String>,
}

impl FromStr for ParsedInfo {
    type Err = Error;

    /*
    (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\"))
    conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\")).
     */

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // use regex to parse the string
        // TODO: how to write regular expressions
        let regex = Regex::new(r#"\((?P<k1>[a-zA-Z0-9_-]+)\s*,\s*(?P<k2>[a-zA-Z0-9_-]+)\)=\((?P<v1>[a-zA-Z0-9_-]+)\s*,\s*\[(?P<v2>[^\)\]]+)"#).unwrap();

        // let maps = regex
        //     .captures_iter(s)
        //     .map(|cap| {
        //         let mut map = HashMap::new();
        //         map.insert(cap["k1"].to_string(), cap["v1"].to_string());
        //         map.insert(cap["k2"].to_string(), cap["v2"].to_string());
        //         map
        //     })
        //     .collect::<Vec<HashMap<String, String>>>();

        // // TODO: how can I convert vec into result

        let result_parsed_info = regex
            .captures_iter(s)
            .try_fold(vec![], |mut vec_maps, cap_item| {
                let mut item = HashMap::new();
                item.insert(cap_item["k1"].to_string(), cap_item["v1"].to_string());
                item.insert(cap_item["k2"].to_string(), cap_item["v2"].to_string());
                vec_maps.push(Some(item));
                Ok(vec_maps)
            })
            .and_then(|mut collected_maps| {
                if collected_maps.len() == 2 {
                    Ok(ParsedInfo {
                        new: collected_maps[0].take().unwrap(), // //! 這邊會使用take()來取出item並把vec_maps中的item設為None
                        old: collected_maps[1].take().unwrap(), // //! 原本使用clone()，但複製HashMap是不必要的，所以改用take()
                    })
                } else {
                    Err(Error::ParsedFailed)
                }
            });

        result_parsed_info
    }
}

impl TryFrom<ParsedInfo> for ReservationConflict {
    type Error = Error;

    fn try_from(info: ParsedInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            new: info.new.try_into()?,
            old: info.old.try_into()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TryFrom<HashMap<String, String>> for ReservationWindow {
    type Error = Error;

    fn try_from(map: HashMap<String, String>) -> Result<Self, Self::Error> {
        let replaced_map = map
            .get("timespan")
            .ok_or(Error::ParsedFailed)?
            .replace('"', ""); //? 把timespan中的"去掉

        let mut split = replaced_map.splitn(2, ',');

        let start = parse_time_into_utc(split.next().ok_or(Error::ParsedFailed)?)?;

        let end = parse_time_into_utc(split.next().ok_or(Error::ParsedFailed)?)?;

        Ok(Self {
            rid: map
                .get("resource_id")
                .ok_or(Error::ParsedFailed)?
                .to_string(),
            start,
            end,
        })
    }
}

fn parse_time_into_utc(s: &str) -> Result<DateTime<Utc>, Error> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|_| Error::InvalidTime)?
        .with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ERR_MSG: &str = "Key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\")).";

    #[test]
    fn test_parse_function_to_take_datetime_utc() {
        let dt = parse_time_into_utc("2022-12-26 22:00:00+00").unwrap();
        assert_eq!(dt.to_rfc3339(), "2022-12-26T22:00:00+00:00");
    }

    #[test]
    fn parsed_info_should_work() {
        let info: ParsedInfo = ERR_MSG.parse().unwrap();
        assert_eq!(info.new["resource_id"], "ocean-view-room-713");
        assert_eq!(
            info.new["timespan"],
            "\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\""
        );
        assert_eq!(info.old["resource_id"], "ocean-view-room-713");
        assert_eq!(
            info.old["timespan"],
            "\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\""
        );
    }

    #[test]
    fn hash_map_to_reservation_window_should_work() {
        let mut map = HashMap::new();
        map.insert("resource_id".to_string(), "ocean-view-room-713".to_string());
        map.insert(
            "timespan".to_string(),
            "\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\"".to_string(),
        );
        let window: ReservationWindow = map.try_into().unwrap();
        assert_eq!(window.rid, "ocean-view-room-713");
        assert_eq!(window.start.to_rfc3339(), "2022-12-26T22:00:00+00:00");
        assert_eq!(window.end.to_rfc3339(), "2022-12-30T19:00:00+00:00");
    }

    #[test]
    fn conflict_error_message_should_parse() {
        let info: ReservationConflictInfo = ERR_MSG.parse().unwrap();
        match info {
            ReservationConflictInfo::Parsed(conflict) => {
                assert_eq!(conflict.new.rid, "ocean-view-room-713");
                assert_eq!(conflict.new.start.to_rfc3339(), "2022-12-26T22:00:00+00:00");
                assert_eq!(conflict.new.end.to_rfc3339(), "2022-12-30T19:00:00+00:00");
                assert_eq!(conflict.old.rid, "ocean-view-room-713");
                assert_eq!(conflict.old.start.to_rfc3339(), "2022-12-25T22:00:00+00:00");
                assert_eq!(conflict.old.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
            }
            ReservationConflictInfo::Unparsed(_) => panic!("should be parsed"),
        }
    }
}
