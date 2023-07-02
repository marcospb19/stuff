use owo_colors::OwoColorize;

pub trait UnwrapOrExplode<T> {
    fn unwrap_or_explode(self, message: &str) -> T;
}

impl<T> UnwrapOrExplode<T> for Option<T> {
    fn unwrap_or_explode(self, message: &str) -> T {
        match self {
            Some(inner) => inner,
            None => explode_error(message),
        }
    }
}

impl<T, E> UnwrapOrExplode<T> for Result<T, E> {
    fn unwrap_or_explode(self, message: &str) -> T {
        match self {
            Ok(inner) => inner,
            Err(_) => explode_error(message),
        }
    }
}

fn explode_error(message: &str) -> ! {
    crate::show!("Error".red(), ":", format_args!("{message}"));
    std::process::exit(1)
}
