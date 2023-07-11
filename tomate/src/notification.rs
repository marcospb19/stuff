use crate::error::UnwrapOrExplode;

pub fn send_notification(message: impl AsRef<str>) {
    notify_rust::Notification::new()
        .summary("Tomate")
        .body(message.as_ref())
        .show()
        .unwrap_or_explode(format!("Failed to send notification with body: \"{}\"\n", message.as_ref()).as_str());
}
