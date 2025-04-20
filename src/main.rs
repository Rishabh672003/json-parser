mod lexer;
mod parser;

fn timeit<T, F: FnOnce() -> T>(label: &str, f: F) -> T {
    use std::time::Instant;
    let start = Instant::now();
    let result = f();
    let end = start.elapsed();
    println!("{label}: {:?}", end);
    result
}

fn main() {
    let mut argv = std::env::args();
    _ = argv.next();
    let file = argv.next().expect("No file was provided");

    let file_content = std::fs::read_to_string(file).unwrap();
    let toks = timeit("Tokenization", || lexer::tokenize(&file_content).unwrap());
    let ans = timeit("Parsing", || parser::parse(&toks)).inspect_err(|x| {
        println!("{}", x);
    });
    let value = match ans {
        Ok(val) => val,
        Err(_) => {
            panic!();
        }
    };

    if toks.len() < 50 {
        println!("{value:#?}");
    }
}
