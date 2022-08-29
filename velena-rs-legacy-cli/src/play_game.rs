/*
 playgame.c
*/

use velena_rs::ai;
use velena_rs::board::Board;

pub enum CLIError {
    //Error while parsing the string, for example 
    //when we send a 'g' as a level or we send a 8 as a column number.
    SyntaxError,
    
    //Position not reachable through a series of legal moves, for
    //example when we keep playing after four men have been connected.
    PositionalError
}

fn parse_input_string(input_string: &str) -> Result<(Board, usize), CLIError> {
    let mut iterator = input_string.chars();
    
    //The first char of the input string represents the difficulty of the AI
    let level = match iterator.next() {
        Some('a') => 1,
        Some('b') => 2,
        Some('c') => 3,
        _ => return Err(CLIError::SyntaxError)
    };
    
    //Fill the board
    let mut board = Board::new();
    
    for move_character in iterator { 
        let column = match String::from(move_character).parse() {
            Ok(0) => return Ok((board, level)),
            Ok(column @ 1..=Board::WIDTH)  => column - 1, //AI uses range 0..7 instead of 1..=7
            _ => return Err(CLIError::SyntaxError)
        };
        if let Ok(new_board) = board.make_move(column) {
            board = new_board;
            if board.is_endgame() {
                return Err(CLIError::PositionalError);
            }
        } else {
            return Err(CLIError::PositionalError);
        }
    }
    
    Err(CLIError::SyntaxError) //No 0 at the end
}

pub fn play_game(input_string: &str) -> Result<usize, CLIError> {
    //Parses the input string and returns Velena's answer
    let (board, level) = parse_input_string(input_string)?;
    Ok(ai::compute_ai_move(board, level) + 1) //Answer in the range 1..=7
}