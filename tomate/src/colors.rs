pub use owo_colors::OwoColorize;

#[macro_export]
macro_rules! show {
    ($expression:expr $(,)?) => {
        ::std::print!("{}", $expression);
        ::std::io::Write::flush(&mut ::std::io::stdout()).expect("Failed to flush STDOUT");
    };
    (@dont_flush $expression:expr $(,)?) => {
        ::std::print!("{}", $expression);
    };
    ($($expression:expr),* $(,)?) => {
        $( $crate::show!(@dont_flush $expression); )*
        ::std::io::Write::flush(&mut ::std::io::stdout()).expect("Failed to flush STDOUT");
    };
}

#[macro_export]
macro_rules! showln {
    ($expression:expr $(,)?) => {
        ::std::println!("{}", $expression);
    };
    ($($expression:expr),* $(,)?) => {
        $($crate::show!($expression);)*
        ::std::println!();
    };
}
