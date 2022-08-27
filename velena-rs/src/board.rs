/*
 connect4.c, ia_main.c
*/

use crate::groups;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Player {
    White, //Yellow in the GUI
    Black //Red in the GUI
}

impl Player {
    fn opponent(self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Square {
    Empty,
    Taken(Player)
}

pub struct Board {
    level: usize, //AI difficulty
    squares: [[Square; Board::HEIGHT]; Board::WIDTH],
    stack_heights: [usize; Board::WIDTH],
    move_count: usize //Number of non-empty squares
}

impl Board {
    pub const WIDTH: usize = 7;
    pub const HEIGHT: usize = 6;
    
    pub fn new(level: usize) -> Self {
        Self {
            level,
            squares: [[Square::Empty; Board::HEIGHT]; Board::WIDTH],
            stack_heights: [0; Board::WIDTH],
            move_count: 0
        }
    }
    
    fn player_to_play(&self) -> Player {
        if self.move_count & 1 == 0 {
            Player::White //Move count is even: white to play
        } else {
            Player::Black //Move count is odd: black to play
        }
    }
    
    fn is_winning(&self) -> bool {
        let target_player = self.player_to_play().opponent(); //Last player to have played
        for group in groups::GROUPS {
            let mut group_complete = true;
            for (x, y) in group {
                if self.squares[x][y] != Square::Taken(target_player) {
                    group_complete = false;
                }
            }
            if group_complete {
                return true;
            }
        }
        false
    }
    
    fn is_draw(&self) -> bool {
        self.move_count == Self::WIDTH * Self::HEIGHT
    }
    
    fn is_endgame(&self) -> bool {
        self.is_winning() || self.is_draw()
    }
    
    pub fn make_move(&mut self, column: usize) -> Result<(), ()> {
        if column >= Self::WIDTH {
            return Err(()); //Invalid column index
        }
        let stack_height = self.stack_heights[column];
        if stack_height >= Self::HEIGHT {
            return Err(()); //Impossible move: column is full
        }
        self.squares[column][stack_height] = Square::Taken(self.player_to_play());
        self.stack_heights[column] += 1;
        self.move_count += 1;
        Ok(())
    }
}