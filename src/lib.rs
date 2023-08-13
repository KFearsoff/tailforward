pub mod axumlib;
pub mod config;

pub mod handlers {
    mod post_webhook;
    pub use post_webhook::webhook_handler;
}

pub mod models {
    pub mod error;
    pub use error::TailscaleWebhook;

    pub mod event;
    pub use event::Event;

    pub mod message;
    pub use message::Message;

    pub mod report;
}

mod services {
    pub mod post_webhook;
    pub mod telegram;
}
