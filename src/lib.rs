pub mod axumlib;
mod telegram;

pub const CURRENT_VERSION: &str = "v1";

pub mod handlers {
    pub mod post_webhook;
}

pub mod models {
    pub mod error;
    pub mod event;
    pub mod report;
}

pub mod services {
    pub mod post_webhook;
}
