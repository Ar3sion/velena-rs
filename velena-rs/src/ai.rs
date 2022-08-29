/*
 ia_main.c
 */

use crate::board::{Board, Player, Square};
use crate::{book, heuristic};

fn random_winning_move(board: Board, player: Player) -> Option<usize> {
    let mut possible_moves = [0; Board::WIDTH];
    let mut count = 0;
    for column in 0..Board::WIDTH {
        if board.is_move_winning(column, player) {
            possible_moves[count] = column;
            count += 1;
        }
    }
    if count == 0 {
        None
    } else {
        Some(possible_moves[fastrand::usize(0..count)])
    }
}

fn try_to_win_immediately(board: Board) -> Option<usize> {
    random_winning_move(board, board.player_to_play())
}

fn avoid_immediate_loss(board: Board) -> Option<usize> {
    random_winning_move(board, board.player_to_play().opponent())
}

fn is_column_alternating(board: Board, column: usize, height: usize, base_player: Player) -> bool {
    let mut player = base_player;
    for i in 0..height {
        if board.get_square(column, i).unwrap() != Square::Taken(player) {
            return false;
        }
        player = player.opponent();
    }
    true
}

fn black_best_move(board: Board) -> Option<usize> {
    //While translating this, I did not understand it fully
    //It looks like a minimalistic opening table for when both players 
    //play perfectly since the beginning
    //I do not store the moves needed to achieve a position in my Board
    //structure, so I had to modify this function.
    //I also made it x-symmetrical as it was not.
    
    if board.filled_squares() >= 14 {
        return None;
    }
    
    //Verify that the central column is filled properly
    if !is_column_alternating(board, 3, board.filled_squares().min(5), Player::White) {
        return None;
    }
    
    match board.filled_squares() {
        //Play in the center first
        0..=4 => Some(3), 
        //Then play in side column 1 or 5, choosing randomly since the position is symmetrical
        5 => Some([1, 5][fastrand::usize(0..=1)]),
        //Continue to play in the same column
        6..=8 => { 
            let side_column_1 = if board.get_square(1, 0).unwrap() == Square::Taken(Player::Black) {
                1
            } else {
                5
            };
            if !is_column_alternating(board, side_column_1, board.filled_squares() - 5, Player::Black) {
                None
            } else {
                Some(side_column_1)
            }
        },
        //Then start to fill the other side column
        9..=12 => {
            let minimal_height = board.filled_squares() - 9;
            if !is_column_alternating(board, 1, minimal_height, Player::Black)
                || !is_column_alternating(board, 5, minimal_height, Player::Black) {
                None //One of the side columns is too short
            } else if is_column_alternating(board, 1, 4, Player::Black) {
                //Column 1 is full
                Some(5)
            } else if is_column_alternating(board, 5, 4, Player::Black) {
                //Column 5 is full
                Some(1)
            } else {
                //No column is full
                None
            }
        }
        //Finally play in side column 1 or 5, choosing randomly since the position is symmetrical
        13 => {
            if !is_column_alternating(board, 1, 4, Player::Black)
                || !is_column_alternating(board, 5, 4, Player::Black) {
                None
            } else {
                Some([1, 5][fastrand::usize(0..=1)])
            }
        },
        _ => panic!()
    }
}

pub fn compute_ai_move(board: Board, level: usize) -> usize {
    if board.filled_squares() == 0 { //Board is empty
        return 3; //Opening in the central column
    }
    if board.filled_squares() == 1 { //We play after White
        return if board.get_square(1, 0).unwrap() == Square::Taken(Player::White) {
            2
        } else if board.get_square(5, 0).unwrap() == Square::Taken(Player::White) {
            4
        } else {
            3
        }
    }
    if board.filled_squares() == Board::SQUARES - 1 { //Only one space left
        for column in 0..Board::WIDTH {
            if board.can_play(column) {
                return column;
            }
        }
        panic!("Invalid board! ");
    }
    if let Some(winning_move) = try_to_win_immediately(board) { //We can win immediately
        return winning_move;
    }
    if let Some(forced_move) = avoid_immediate_loss(board) { //We are forced to play
        return forced_move;
    }
    //Special opening case for Black (I think)
    if board.player_to_play() == Player::Black && (level >= 3 || board.filled_squares() == 1) {
        if let Some(best_move) = black_best_move(board) { 
            return best_move;
        }
    }
    if level >= 3 { //Let's look in the opening book
        if let Some(opening) = book::use_opening_book(board) {
            return opening;
        }
    }
    if let Some(heuristic) = heuristic::heuristic_best_play(board, false) {
        return heuristic; //The heuristic was enough to find a solution
    }
    0
}