use irust_repl::*;

#[test]
fn repl() {
    let mut repl = Repl::default();
    repl.insert("let a = 4;");
    repl.insert("let b = 6;");
    assert_eq!(repl.eval("a+b").unwrap().output, "10");

    repl.insert(r#"let c = "hello";"#);
    assert_eq!(repl.eval("c.chars().count() < a+b").unwrap().output, "true");

    repl.set_executor(Executor::AsyncStd).unwrap();
    repl.insert("async fn d() -> usize {4}");
    assert_eq!(repl.eval("d().await").unwrap().output, "4");
}

#[test]
fn two_repls_at_the_same_time() {
    let mut repl1 = Repl::default();
    let mut repl2 = Repl::default();
    repl1.insert("let a = 4;");
    repl2.insert("let a = 5;");

    let a1_thread =
        std::thread::spawn(move || repl1.eval("a").unwrap().output.parse::<u8>().unwrap());
    let a2_thread =
        std::thread::spawn(move || repl2.eval("a").unwrap().output.parse::<u8>().unwrap());

    assert_eq!(a1_thread.join().unwrap() + a2_thread.join().unwrap(), 9)
}
