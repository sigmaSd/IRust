#[macro_export]
macro_rules! log_inner {
    ($($arg:tt)*) => (
        std::sync::Once::new().call_once(|| {
            use std::io::Write;

            let mut irust_log = if std::path::Path::exists(std::path::Path::new("irust.log")) {
                std::fs::OpenOptions::new().append(true).open("irust.log").unwrap()
            } else {
                std::fs::File::create("irust.log").unwrap()
            };

        	writeln!(irust_log, "{} line {}: {}", file!(), line!(), format_args!($($arg)*)).unwrap();
        });

    );
}

#[macro_export]
macro_rules! log {
    () => {
        $crate::log_inner!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::log_inner!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    // Trailing comma with single argument is ignored
    ($val:expr,) => { $crate::log!($val) };
    ($($val:expr),+ $(,)?) => {
        ($($crate::log!($val)),+,)
    };
}
