// 我想返回一個很“清晰”衝突問題給用戶而不是這樣
/*
ConflictReservation("Key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\"))
conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\")).")
test manager::tests::reserve_conflict_should_reject ... ok
*/

// 那由於PgDatabaseError也沒有其他方法提供一個更好的錯誤信息，我們只能自己implement function從 PgDatabaseError中的 "get_raw()"取得一大坨原始資料並解析它。

// TODO: write a parser

use chrono::{DateTime, Utc};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str::FromStr};

//? target: to parse the info to below struct

#[derive(Debug, Clone)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

#[derive(Debug, Clone)]
pub struct ReservationConflict {
    _a: ReservationWindow,
    _b: ReservationWindow,
}

#[derive(Debug, Clone)]
pub struct ReservationWindow {
    _rid: String,
    _start: DateTime<Utc>,
    _end: DateTime<Utc>,
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

impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

#[warn(unused_attributes)]
#[allow(dead_code)]
pub struct ParsedInfo {
    a: HashMap<String, String>,
    b: HashMap<String, String>,
}

impl FromStr for ParsedInfo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // use regex to parse the string
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

        // TODO: how can I convert vec into result

        let result_parsed_info = regex
            .captures_iter(s)
            .try_fold(Vec::new(), |mut vec_maps, cap_item| {
                let mut item = HashMap::new();
                item.insert(cap_item["k1"].to_string(), cap_item["v1"].to_string());
                item.insert(cap_item["k2"].to_string(), cap_item["v2"].to_string());
                vec_maps.push(Some(item));
                Ok(vec_maps)
            })
            .and_then(|mut collected_maps| {
                if collected_maps.len() == 2 {
                    Ok(ParsedInfo {
                        a: collected_maps[0].take().unwrap(), // //! 這邊會使用take()來取出item並把vec_maps中的item設為None
                        b: collected_maps[1].take().unwrap(), // //! 原本使用clone()，但複製HashMap是不必要的，所以改用take()
                    })
                } else {
                    Err(())
                }
            });

        result_parsed_info
    }
}
