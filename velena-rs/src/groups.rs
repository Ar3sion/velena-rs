/*
 connect4.c
 */

use crate::board::Board;

pub const GROUP_COUNT: usize = 69;

const fn generate_groups() -> [[(usize, usize); 4]; GROUP_COUNT] {
    let mut output = [[(0, 0); 4]; GROUP_COUNT];
    let mut i = 0;
    
    // Horizontal lines
    let mut y = 0;
    while y < Board::HEIGHT {
        let mut x = 0;
        while x < Board::WIDTH - 3{
            output[i][0] = (x + 0, y);
            output[i][1] = (x + 1, y);
            output[i][2] = (x + 2, y);
            output[i][3] = (x + 3, y);
            i += 1;
            
            x += 1;
        }
        y += 1;
    }

    //Vertical lines
    let mut y = 0;
    while y < Board::HEIGHT - 3 {
        let mut x = 0;
        while x < Board::WIDTH {
            output[i][0] = (x, y + 0);
            output[i][1] = (x, y + 1);
            output[i][2] = (x, y + 2);
            output[i][3] = (x, y + 3);
            i += 1;

            x += 1;
        }
        y += 1;
    }
    
    //Diagonal (north-east) lines
    let mut y = 0;
    while y < Board::HEIGHT - 3 {
        let mut x = 0;
        while x < Board::WIDTH - 3 {
            output[i][0] = (x + 0, y + 0);
            output[i][1] = (x + 1, y + 1);
            output[i][2] = (x + 2, y + 2);
            output[i][3] = (x + 3, y + 3);
            i += 1;

            x += 1;
        }
        y += 1;
    }

    //Diagonal (south-east) lines
    let mut y = 3;
    while y < Board::HEIGHT {
        let mut x = 0;
        while x < Board::WIDTH - 3 {
            output[i][0] = (x + 0, y - 0);
            output[i][1] = (x + 1, y - 1);
            output[i][2] = (x + 2, y - 2);
            output[i][3] = (x + 3, y - 3);
            i += 1;

            x += 1;
        }
        y += 1;
    }
    
    output
}

pub const GROUPS: [[(usize, usize); 4]; GROUP_COUNT] = generate_groups();