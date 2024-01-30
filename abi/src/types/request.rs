use crate::{
    CancelRequest, ConfirmRequest, FilterById, FilterRequest, GetRequest, QueryRequest,
    Reservation, ReservationQuery, ReserveRequest,
};

// 這邊使用macro來簡化程式碼，可以看到其實macro就像是函數一樣簡化重複性的函數，有點像是函數中的prototype(原型)

macro_rules! impl_new {
     // 用於匹配單個參數且有Option<field>的結構體
     (single $name:ident, $field:ident, $type:ty) => {
        impl $name {
            pub fn new(value: $type) -> Self {
                Self {
                    $field: Some(value),
                }
            }
        }
    };
    // 用於匹配多個參數並且有id的結構體
    ($($name:ident),* $(,)?) => {
        $(
            impl $name {
                pub fn new(id: i64) -> Self {
                    Self { id }
                }
            }
        )*
    };
}

impl_new!(single ReserveRequest, reservation, Reservation);
impl_new!(ConfirmRequest, GetRequest, CancelRequest);
impl_new!(single FilterRequest, query, FilterById);
impl_new!(single QueryRequest, query, ReservationQuery);

// TODO: 這邊的macro有點複雜，需要再研究一下

// impl ReserveRequest {
//     pub fn new(rsvp: Reservation) -> Self {
//         Self {
//             reservation: Some(rsvp),
//         }
//     }
// }

// impl ConfirmRequest {
//     pub fn new(rsvp: Reservation) -> Self {
//         Self {
//             reservation: Some(rsvp),
//         }
//     }
// }

// impl FilterRequest {
//     pub fn new(filter: FilterById) -> Self {
//         Self {
//             query: Some(filter),
//         }
//     }
// }
