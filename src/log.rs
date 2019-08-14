#[macro_export]
macro_rules! log {
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
