#[derive(Debug)]
struct Hello {
    field: Value
}

/// Here's typo a that I wouldn't have noticed
// rust-analyser

fn function() -> Bye { Bye }

fn foo() -> Bar {
    println!("Hello, World!")
}
