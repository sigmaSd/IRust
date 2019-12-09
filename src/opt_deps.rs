use crate::options;
use std::io;
use std::process;

struct Dep {
    name: &'static str,
    cmd: &'static str,
    function: &'static str,
    install: &'static dyn Fn() -> io::Result<Vec<process::ExitStatus>>,
}
impl Dep {
    fn new(
        name: &'static str,
        cmd: &'static str,
        function: &'static str,
        install: &'static dyn Fn() -> io::Result<Vec<process::ExitStatus>>,
    ) -> Self {
        Dep {
            name,
            cmd,
            function,
            install,
        }
    }
}

pub fn warn_about_opt_deb(options: &mut options::Options) {
    let opt_deps: [Dep; 3] = [
        Dep::new("racer", "racer", "auto_completion", &|| {
            let mut exit_status = vec![];
            exit_status.push(
                process::Command::new("cargo")
                    .args(&["+nightly", "install", "racer"])
                    .status()?,
            );
            exit_status.push(
                process::Command::new("rustup")
                    .args(&["component", "add", "rust-src"])
                    .status()?,
            );
            Ok(exit_status)
        }),
        Dep::new("cargo-fmt", "cargo-fmt", "beautifying repl code", &|| {
            Ok(vec![process::Command::new("cargo")
                .args(&["install", "cargo-fmt"])
                .status()?])
        }),
        Dep::new("cargo-edit", "cargo-add", "adding depedencies", &|| {
            Ok(vec![process::Command::new("cargo")
                .args(&["install", "cargo-edit"])
                .status()?])
        }),
    ];

    // only warn when irust is first used
    if !options.first_irust_run {
        return;
    }

    println!(
        "Hi and Welcome to IRust!\n\
         This is a one time message\n\
         IRust will check now for optional dependencies and offer to install them\n\
         --------------------------------------------------\n--------------------------------------------------"
    );

    for dep in &opt_deps {
        if !dep_installed(dep.cmd) {
            println!();
            println!(
                "{} is not installed, it's required for {}\n\
                 Do you want IRust to install it (using cargo install)? [Y/n]: ",
                dep.name, dep.function
            );
            let answer = {
                let mut a = String::new();
                let _ = std::io::stdin().read_line(&mut a);
                a
            };
            let answer = answer.trim();
            if answer.is_empty() || answer == "y" || answer == "Y" {
                match (dep.install)() {
                    Ok(status) if status.iter().all(process::ExitStatus::success) => {
                        println!("{} sucessfully installed!\n", dep.name)
                    }
                    _ => println!("error while installing {}", dep.name),
                };
            }
        }
    }
    options.first_irust_run = false;
}

fn dep_installed(d: &str) -> bool {
    if let Err(e) = std::process::Command::new(d)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        if e.kind() == std::io::ErrorKind::NotFound {
            return false;
        }
    }
    true
}
