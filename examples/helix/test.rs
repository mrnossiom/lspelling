#[derive(Debug)]
struct Hello {
    field: Value
}

/// Here's typo a that I wouldn't have noticed
// rust-analyser

fn function(t: impl Fn() -> bool) -> Bye { Bye }

fn foo() -> Bar {
    println!("Hello, World!");

    let bye_jello_boulo = "Bye, Jello!";
}
