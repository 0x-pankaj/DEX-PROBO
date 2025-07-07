use serde::{Deserialize, Serialize};

use crate::order::{OptionType, OrderType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromApi {
    CreateOrder {
        user_id: String,
        marker_id: String,
        option: OptionType,
        order_type: OrderType,
        price: f64,
        quantity: u32,
    },
    CancelOrder {
        user_id: String,
        order_id: String,
    },
}
