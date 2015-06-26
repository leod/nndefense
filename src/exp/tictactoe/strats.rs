use std::io;
use std::io::Write;
use rand;
use rand::Rng;

use exp::tictactoe::game::*;

// Strategies as in http://anji.sourceforge.net/docs/james_gecco04.pdf
pub struct InputStrategy;
pub struct RandomStrategy;
pub struct CenterStrategy;
pub struct BadStrategy;
pub struct BestStrategy {
    // If `forkable` is true, the strategy can be tricked by using a fork
    pub forkable: bool 
}

impl Strategy for InputStrategy {
    fn get_move(&mut self, me: Player, state: &GameState) -> (usize, usize) {
        loop {
            print!("Move {}: ", match me { Player::X => "X", Player::O => "O" });
            io::stdout().flush();

            let mut line = String::new();
            io::stdin().read_line(&mut line);

            if line.len() < 2 {
                continue;
            }

            let new_len = line.len() - 1;
            line.truncate(new_len);

            let n = match line.parse::<usize>() { 
                Ok(n) => n,
                _ => continue
            };

            if n > 9 {
                continue;
            }

            let x = n % 3;
            let y = n / 3;

            if state.field[x][y].is_some() {
                continue;
            }

            return (x, y)
        }
    }
}

impl Strategy for RandomStrategy {
    fn get_move(&mut self, me: Player, state: &GameState) -> (usize, usize) {
        let mut legal_moves = vec![];

        for x in 0..3 {
            for y in 0..3 {
                if state.field[x][y].is_none() {
                    legal_moves.push((x,y));
                }
            }
        }

        let mut rng = rand::thread_rng();
        *rng.choose(&legal_moves).unwrap()
    }
}

impl Strategy for CenterStrategy {
    fn get_move(&mut self, me: Player, state: &GameState) -> (usize, usize) {
        if state.field[1][1].is_none() {
            (1, 1)
        } else {
            RandomStrategy.get_move(me, state)
        }
    }
}

impl Strategy for BadStrategy {
    fn get_move(&mut self, me: Player, state: &GameState) -> (usize, usize) {
        let mut rng = rand::thread_rng();

        let sides = vec![(0, 1), (1, 0), (2, 1), (1, 2)];
        let free_sides = sides 
                             .into_iter()
                             .filter(|p| state.field[p.0][p.1].is_none())
                             .collect::<Vec<(usize, usize)>>();
        if free_sides.len() > 0 {
            loop {
                match rng.choose(&free_sides) {
                    Some(p) => return *p,
                    None => ()
                };
            }
        }

        let corners = vec![(0, 0), (2, 2), (2, 0), (0, 2)];
        let free_corners = corners
                             .into_iter()
                             .filter(|p| state.field[p.0][p.1].is_none())
                             .collect::<Vec<(usize, usize)>>();
        if free_corners.len() > 0 {
            loop {
                match rng.choose(&free_corners) {
                    Some(p) => return *p,
                    None => ()
                };
            }
        }

        assert!(state.field[1][1].is_none());
        (1, 1)
    }
}

impl Strategy for BestStrategy {
    fn get_move(&mut self, me: Player, state: &GameState) -> (usize, usize) {
        // http://programmers.stackexchange.com/questions/213559/algorithm-to-create-an-tictactoe-game-ai

        let not_me = match me {
            Player::X => Player::O,
            Player::O => Player::X
        };

        let mut rng = rand::thread_rng();
 
        // 1. If there are two in a row, complete it
        let count_fields = |state: &GameState, player, fields: &[(usize, usize); 3]| fields.iter().fold(0, |count, &(x, y)| {
            if state.field[x][y] == player {
                count + 1
            } else {
                count
            }
        });

        for win_row in HOW_TO_WIN.iter() {
            // See if we are at two spots of the win_row and the other spot is free
            let num_mine = count_fields(state, Some(me), win_row);
            let num_free = count_fields(state, None, win_row);

            //let bla: &[(usize, usize)] = win_row;

            // Can we complete?
            if num_mine == 2 && num_free == 1 {
                for &(x, y) in win_row {
                    if state.field[x][y].is_none() {
                        //println!("Completing");
                        return (x, y);
                    }
                }

                //return bla.iter().find(|&p| state.field[p.0][p.1].is_none()).unwrap();
            }
        }

        // 2. If the other player has two in a row, block
        for win_row in HOW_TO_WIN.iter() {
            let num_others = count_fields(state, Some(not_me), win_row);
            let num_free = count_fields(state, None, win_row);

            if num_others == 2 && num_free == 1 {
                for &(x, y) in win_row {
                    if state.field[x][y].is_none() {
                        //println!("Preventing completion");
                        return (x, y);
                    }
                }

                //return win_row.iter().find(|&(x,y)| state.field[x][y].is_none()).unwrap();
            }
        }

        // 3. Fork: find a move that gets us two uncomplete rows where we have two fields
        let next_is_fork = |state: &GameState, next_player, (next_x, next_y)| {
            let mut num_good_rows = 0; 
            let next_state = state.move_copy(next_player, (next_x, next_y));

            for win_row in HOW_TO_WIN.iter() {
                let next_num_mine = count_fields(&next_state, Some(next_player), win_row);
                let next_num_free = count_fields(&next_state, None, win_row);

                if next_num_mine == 2 && next_num_free == 1 {
                    num_good_rows += 1;
                }
            }

            return num_good_rows >= 2;
        };

        let mut forks = Vec::new();

        for x in 0..3 {
            for y in 0..3 {
                match state.field[x][y] {
                    Some(_) => (),
                    None => {
                        if next_is_fork(&state, me, (x, y)) {
                            forks.push((x, y));
                        }
                    }
                };
            }
        }

        match rng.choose(&forks) {
            Some(p) => { 
                //println!("Taking fork");
                return *p;
            },

            None => ()
        };
        
        // 4. Block other player's fork
        if !self.forkable {
            for x in 0..3 {
                for y in 0..3 {
                    match state.field[x][y] {
                        Some(_) => (),
                        None => {
                            if next_is_fork(&state, not_me, (x, y)) {
                                // 4.1. Create two in a row, forcing a blocking move
                                let mut forcing_moves = Vec::new();

                                for win_row in HOW_TO_WIN.iter() {
                                    let num_mine = count_fields(state, Some(me), win_row);
                                    let num_free = count_fields(state, None, win_row);

                                    if num_mine == 1 && num_free == 2 {
                                        for &(x1, y1) in win_row {
                                            if state.field[x1][y1].is_none() {
                                                // Check that the reply to our blocking move is not a fork
                                                let next_state = state.move_copy(me, (x1, y1)); 

                                                for &(x2, y2) in win_row {
                                                    if (x1 != x2 || y1 != y2) && next_state.field[x2][y2].is_none() {
                                                        if !next_is_fork(&next_state, not_me, (x2, y2)) {
                                                            //println!("Creating two in a row to block fork");
                                                            //return (x1, y1);
                                                            forcing_moves.push((x1, y1));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                return match rng.choose(&forcing_moves) {
                                    Some(p) => *p,
                                    None =>
                                        // 4.2. Block fork directly
                                        //println!("Blocking fork directly");
                                        return (x, y)
                                }

                            }
                        }
                    };
                }
            }
        }

        // 5. Play in the center
        if state.field[1][1].is_none() {
            //println!("Center");
            return (1, 1);
        }

        // 6. Play in opposing corner
        let mut opposing_corners = Vec::new();

        if state.field[0][0] == Some(not_me) && state.field[2][2].is_none() {
            opposing_corners.push((2, 2));
        }
        if state.field[2][2] == Some(not_me) && state.field[0][0].is_none() {
            opposing_corners.push((0, 0));
        }
        if state.field[0][2] == Some(not_me) && state.field[2][0].is_none() {
            opposing_corners.push((2, 0));
        }
        if state.field[2][0] == Some(not_me) && state.field[0][2].is_none() {
            opposing_corners.push((0, 2));
        }

        match rng.choose(&opposing_corners) {
            Some(p) => return *p,
            None => ()
        };

        // 7. Play in an empty corner
        let mut empty_corners = Vec::new();

        if state.field[2][2].is_none() {
            empty_corners.push((2, 2));
        }
        if state.field[0][0].is_none() {
            empty_corners.push((0, 0));
        }
        if state.field[2][0].is_none() {
            empty_corners.push((2, 0));
        }
        if state.field[0][2].is_none() {
            empty_corners.push((0, 2));
        }

        match rng.choose(&empty_corners) {
            Some(p) => return *p,
            None => ()
        };

        // 8. Play in the middle of an empty side
        let mut empty_sides = Vec::new();

        if state.field[0][1].is_none() {
            empty_sides.push((0, 1));
        }
        if state.field[1][0].is_none() {
            empty_sides.push((1, 0));
        }
        if state.field[2][1].is_none() {
            empty_sides.push((2, 1));
        }
        if state.field[1][2].is_none() {
            empty_sides.push((1, 2));
        }

        match rng.choose(&empty_sides) {
            Some(p) => return *p,
            None => ()
        };

        // Now we should have covered all the possibilities
        assert!(false);

        let one_million = 1000000;
        return (one_million, one_million);
    }
}
