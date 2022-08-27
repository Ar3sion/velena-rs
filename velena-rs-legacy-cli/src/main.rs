/*
 connect4.c, cmdline.c
*/

mod play_game;

use std::io;
use crate::play_game::CLIError;

fn main() {
    let mut line = String::new();
    let mut quit = false;

    while !quit {
        io::stdin().read_line(&mut line).unwrap();

        if line.starts_with("q") {
            quit = true; //We quit
        } else {
            match play_game::play_game(&line) {
                Ok(answer) => println!("Velena answers in {}", answer),
                Err(error) => match error {
                    CLIError::SyntaxError => println!("?Syntax error"),
                    CLIError::PositionalError => println!("?Positional error")
                }
            }
        }
    }
    
    println!("Program terminated");
}
