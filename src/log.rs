#[macro_export]
macro_rules! log {
    ($msg:literal $(,$arg:expr)*) => {
        println!(
            "{} [\x1b[34mMESSAGE\x1b[0m]: {}",
            chrono::Local::now().format("%d-%m-%y %H:%M:%S"),
            format!($msg, $($arg,)*)
        )
    };
}

#[macro_export]
macro_rules! log_error {
    ($msg:literal $(,$arg:expr)*) => {
        eprintln!(
            "{} [\x1b[31mERROR\x1b[0m]: {}",
            chrono::Local::now().format("%d-%m-%y %H:%M:%S"),
            format!($msg, $($arg,)*)
        )
    };
}
