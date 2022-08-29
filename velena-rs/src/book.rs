/*
 ia_main.c, connect4.c, buildob.c, database.c
*/

use std::cmp::Ordering;
use crate::board::{Board, Player, Square};

fn encode_square(board: Board, column: usize, row: usize) -> u8 {
    match board.get_square(column, row).unwrap() {
        Square::Empty => 0,
        Square::Taken(Player::White) => 1,
        Square::Taken(Player::Black) => 2
    }
}

fn collapse_position(board: Board) -> [u8; 14] {
    let mut output = [0; 14];
    for row in 0..Board::HEIGHT {
        output[row * 2 + 0] =
            encode_square(board, 0, row) << 6 |
            encode_square(board, 1, row) << 4 |
            encode_square(board, 2, row) << 2 |
            encode_square(board, 3, row) << 0;
        output[row * 2 + 1] =
            encode_square(board, 4, row) << 6 |
            encode_square(board, 5, row) << 4 |
            encode_square(board, 6, row) << 2 | 0b11;
    }
    output
}

fn check_book(mut collapsed: [u8; 14], player: Player) -> bool {
    //I will probably get rid of precomputed moves later on, we're not in the 90s anymore

    const WHITE_BOOK: &[u8] = include_bytes!("openbook.cn4");
    const BOOK_SIZE: usize = WHITE_BOOK.len() / 14;

    match player {
        Player::White => { //1
            collapsed[12] = 0x01;
            collapsed[13] = 0x00;
        } 
        Player::Black => { //-1
            collapsed[12] = 0xff;
            collapsed[13] = 0xff;
        } 
    };

    //Dichotomy
    let mut left = 0;
    let mut right = BOOK_SIZE;

    while left != right {
        let middle = (left + right) / 2;
        let value = &WHITE_BOOK[middle * 14..(middle + 1) * 14];
        match value.cmp(&collapsed) {
            Ordering::Less => {
                left = middle + 1;
            }
            Ordering::Equal => {
                return true;
            }
            Ordering::Greater => {
                right = middle;
            }
        }
    }

    false
}

pub fn use_opening_book(board: Board) -> Option<usize> {
    let mut possible_moves = [0; Board::WIDTH];
    let mut count = 0;

    let player = board.player_to_play();
    for column in 0..Board::WIDTH {
        if let Ok(new_board) = board.make_move(column) {
            let collapsed = collapse_position(new_board.min(new_board.symmetric_board()));
            if check_book(collapsed, player) {
                possible_moves[count] = column;
                count += 1;
            }
        }
    }

    if count == 0 {
        None
    } else {
        Some(possible_moves[fastrand::usize(0..count)])
    }
}