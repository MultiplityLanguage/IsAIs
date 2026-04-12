use isais::{Interpreter, models::MockLLM};
use std::io::{self, Write, BufRead};

fn main() {
    println!("IsAIs Interpreter v0.1.0");
    println!("A language for LLMs, by LLMs\n");
    println!("Type IsAIs code or 'quit' to exit\n");

    let model = Box::new(MockLLM);
    let mut interpreter = Interpreter::new(model);

    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        match line {
            Ok(input) => {
                let input = input.trim();
                
                if input.is_empty() {
                    print!("isais> ");
                    io::stdout().flush().unwrap();
                    continue;
                }

                if input == "quit" || input == "exit" {
                    println!("Goodbye!");
                    break;
                }

                print!("isais> ");
                match interpreter.evaluate(input) {
                    Ok(value) => println!("=> {}\n", value),
                    Err(e) => eprintln!("Error: {}\n", e),
                }
            }
            Err(_) => {
                break;
            }
        }
    }
}