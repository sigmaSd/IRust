use crossterm::style::Stylize;
use std::io;
use std::process;

use crate::irust::options::Options;

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

pub fn check_required_deps() -> bool {
    const REQUIRED_DEPS: &[&str] = &["cargo"];
    for dep in REQUIRED_DEPS {
        if !dep_installed(dep) {
            eprintln!(
                "{0} is not insalled!\n{0} is required for IRust to work.",
                dep
            );
            return false;
        }
    }
    true
}

pub fn warn_about_opt_deps(options: &mut Options) {
    let opt_deps: [Dep; 4] = [
        Dep::new("racer", "racer", "auto_completion", &|| {
            let mut exit_status = vec![];
            let mut run_cmd = |cmd: &[&str]| -> io::Result<()> {
                println!("{}", format!("Running: {:?}", cmd).magenta());
                exit_status.push(process::Command::new(cmd[0]).args(&cmd[1..]).status()?);
                Ok(())
            };

            if !dep_installed("rustup") {
                println!(
                    "{}",
                    "rustup is not installed.\nrustup is required to install and configure racer"
                        .red()
                );
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "rustup is not installed",
                ));
            }

            let cmd = ["rustup", "install", "nightly"];
            run_cmd(&cmd)?;

            let cmd = ["cargo", "+nightly", "install", "racer"];
            run_cmd(&cmd)?;

            let cmd = ["rustup", "component", "add", "rust-src"];
            run_cmd(&cmd)?;

            Ok(exit_status)
        }),
        Dep::new("rustfmt", "rustfmt", "beautifying repl code", &|| {
            if !dep_installed("rustup") {
                println!(
                    "{}",
                    "rustup is not installed.\nrustup is required to install rustfmt".red()
                );
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "rustup is not installed",
                ));
            }
            let cmd = ["rustup", "component", "add", "rustfmt"];
            println!("{}", format!("Running: {:?}", cmd).magenta());

            Ok(vec![process::Command::new(cmd[0])
                .args(&cmd[1..])
                .status()?])
        }),
        Dep::new("cargo-edit", "cargo-add", "adding depedencies", &|| {
            let cmd = ["cargo", "install", "cargo-edit"];
            println!("{}", format!("Running: {:?}", cmd).magenta());

            Ok(vec![process::Command::new(cmd[0])
                .args(&cmd[1..])
                .status()?])
        }),
        Dep::new(
            "cargo-asm",
            "cargo-asm",
            "viewing functions assembly",
            &|| {
                let cmd = ["cargo", "install", "cargo-asm"];
                println!("{}", format!("Running: {:?}", cmd).magenta());

                Ok(vec![process::Command::new(cmd[0])
                    .args(&cmd[1..])
                    .status()?])
            },
        ),
    ];

    // only warn when irust is first used
    if !options.first_irust_run {
        return;
    }

    println!(
        "{}",
        "Hi and Welcome to IRust!\n\
         This is a one time message\n\
         IRust will check now for optional dependencies and offer to install them\n\
         "
        .dark_blue()
    );

    let mut installed_something = false;
    for dep in &opt_deps {
        if !dep_installed(dep.cmd) {
            println!();
            println!(
                "{}",
                format!(
                    "{} is not installed, it's required for {}\n\
                 Do you want IRust to install it? [Y/n]: ",
                    dep.name, dep.function
                )
                .yellow()
            );
            let answer = {
                let mut a = String::new();
                std::io::stdin()
                    .read_line(&mut a)
                    .expect("failed to read stdin");
                a.trim().to_string()
            };

            if answer.is_empty() || answer == "y" || answer == "Y" {
                match (dep.install)() {
                    Ok(status) if status.iter().all(process::ExitStatus::success) => {
                        println!(
                            "{}",
                            format!("{} sucessfully installed!\n", dep.name).green()
                        );
                        installed_something = true;
                    }
                    _ => println!("{}", format!("error while installing {}", dep.name).red()),
                };
            }
        }
    }
    options.first_irust_run = false;

    if installed_something {
        println!(
            "{}",
            "You might need to reload the shell inorder to update $PATH".yellow()
        );
    }
    println!("{}", "Everthing is set!".green());
}

fn dep_installed(d: &str) -> bool {
    if let Err(e) = std::process::Command::new(d)
        .arg("-h")
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
