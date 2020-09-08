use crossterm::style::Colorize;
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

pub fn check_required_deps() {
    const REQUIRED_DEPS: &[&str] = &["cargo"];
    for dep in REQUIRED_DEPS {
        if !dep_installed(&dep) {
            println!(
                "{}",
                format!(
                    "{0} is not insalled!\n{0} is required for IRust to work.",
                    dep
                )
                .red()
            );
            std::process::exit(1);
        }
    }
}

pub fn warn_about_opt_deps(irust: &mut crate::IRust) {
    let opt_deps: [Dep; 3] = [
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
                std::process::exit(1);
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
                std::process::exit(1);
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
    ];

    // only warn when irust is first used
    if !irust.options.first_irust_run {
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
                let _ = std::io::stdin().read_line(&mut a);
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
    irust.options.first_irust_run = false;

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