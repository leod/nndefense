pub const HOW_TO_WIN: &'static [[(usize, usize); 3]] = &[
    [(0,0),(0,1),(0,2)],
    [(1,0),(1,1),(1,2)],
    [(2,0),(2,1),(2,2)],

    [(0,0),(1,0),(2,0)],
    [(0,1),(1,1),(2,1)],
    [(0,2),(1,2),(2,2)],

    [(0,0),(1,1),(2,2)],
    [(2,0),(1,1),(0,2)]
];

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Player {
    X,
    O
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct GameState {
    pub field: [[Option<Player>; 3]; 3]
}

impl GameState {
    pub fn move_copy(&self, player: Player, (x, y): (usize, usize)) -> GameState {
        assert!(self.field[x][y].is_none());

        let mut new_state = *self;
        new_state.field[x][y] = Some(player);

        return new_state;
    }
}

pub trait Strategy {
    fn get_move(&mut self, me: Player, state: &GameState) -> (usize, usize);
}

pub fn initial_state() -> GameState {
    GameState {
        field: [
            [None, None, None],
            [None, None, None],
            [None, None, None]
        ]
    }
}

pub fn print_state(state: &GameState) {
    //println!("---");

    for y in 0..3 {
        for x in 0..3 {
            print!("{}", match state.field[x][y] {
                Some(Player::X) => "X",
                Some(Player::O) => "O",
                None => " ",
            });
        }

        println!("");
    }

    println!("---");
}

pub fn play(strat_x: &mut Strategy, strat_o: &mut Strategy, print: bool) -> Option<Player> {
    let mut state = initial_state();
    let mut turn = Player::X;

    for _ in 0..9 {
        let (move_x, move_y) = match turn {
            Player::X => strat_x.get_move(turn, &state),
            Player::O => strat_o.get_move(turn, &state)
        };
        assert!(move_x < 3 && move_y < 3);
        assert!(state.field[move_x][move_y].is_none());

        state.field[move_x][move_y] = Some(turn);

        if print {
            print_state(&state);
        }

        if HOW_TO_WIN.iter().any(|positions| positions.iter().all(|&(x,y)| state.field[x][y] == Some(turn))) {
            if print {
                println!("Player {} wins", match turn {
                    Player::X => "X",
                    Player::O => "O",
                });
            }

            return Some(turn);
        }

        turn = match turn {
            Player::X => Player::O,
            Player::O => Player::X
        };
    }

    if print {
        println!("Draw");
    }

    None
}
