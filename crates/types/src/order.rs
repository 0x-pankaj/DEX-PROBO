use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum OptionType {
    Yes,
    No,
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub market_id: String,
    pub option: OptionType,
    pub order_type: OrderType,
    pub price: f64,
    pub quantity: u32,
    pub timestamp: u64,
}
