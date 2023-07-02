#![allow(unused)]

enum Notification {
    Started,
    TimeToRest,
    Resumed,
    Paused,
}

pub fn send_notification(message: impl AsRef<str>) {
    notify_rust::Notification::new()
        .summary("Tomate")
        .body(message.as_ref())
        .show()
        .unwrap();
}
