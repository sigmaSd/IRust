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
