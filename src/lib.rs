pub mod axumlib;

pub const CHAT_ID: i64 = -1_001_864_190_705;

pub mod handlers {
    pub mod post_webhook;
}

pub mod models {
    pub mod error;
    pub mod event;
    pub mod message;
    pub mod report;
}

pub mod services {
    pub mod post_webhook;
    pub mod telegram;
}
