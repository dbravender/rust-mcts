
use std::fmt;
use rand::{Rng, XorShiftRng, SeedableRng};

use mcts::{GameAction, Game};

pub const WIDTH: usize = 4;
pub const HEIGHT: usize = 4;

#[derive(Clone)]
/// Implementation of the 2048 game mechanics.
///
/// This game needs a random source to perform moves -- in order to fully derteminize it
/// we need to store our own random number generator.
pub struct TwoFortyEight {
    rng:   XorShiftRng,
    board: [u16; WIDTH*HEIGHT],
    pub score: f32,
    pub moves: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
/// Possible moves for the 2048 game.
///
/// One of Up, Down. Left or Right.
pub enum Action {
    Up, Down, Left, Right
}
impl GameAction for Action {}


impl TwoFortyEight {
    /// Create a new empty game
    pub fn new_empty() -> TwoFortyEight {
        // XXX What about the seed?
        TwoFortyEight {
            rng: XorShiftRng::from_seed([1,2,3,4]),
            score: 0.0,
            moves: 0,
            board: [0; WIDTH*HEIGHT]
        }
    }

    // Create a new game with two random two's in it.
    pub fn new() -> TwoFortyEight {
        let mut game = TwoFortyEight::new_empty();
        game.random_spawn();
        game.random_spawn();
        game
    }

    /// Static method
    fn merge_vec(vec: &Vec<u16>) -> (Vec<u16>, f32, bool) {
        let mut points = 0.0;

        // first, remove zeros
        let orig_len = vec.len();
        let filtered_vec = vec.iter().map(|t| *t).filter(|&t| t > 0).collect::<Vec<u16>>();

        // Remove duplicates
        let mut merged = Vec::new();
        let mut next = 0;
        for t in filtered_vec {
            if t == next {
                merged.push(2*t);
                next = 0;
                points += 2.* (t as f32);
            } else {
                if next != 0 {
                    merged.push(next);
                }
                next = t;
            }
        }
        if next != 0 {
            merged.push(next);
        }

        // Make sure we keep the original length and notice any changes
        let changed = orig_len != merged.len();
        for _ in 0..(orig_len-merged.len()) {
            merged.push(0);
        }
        (merged, points, changed)
    }

    /// Shift and merge in the given direction
    fn shift_and_merge(board: [u16; WIDTH*HEIGHT], action: &Action) -> ([u16; WIDTH*HEIGHT], Option<f32>) {
        let (start, ostride, istride) = match *action {
            Action::Up    => ( 0,  1,  4),
            Action::Down  => (12,  1, -4),
            Action::Left  => ( 0,  4,  1),
            Action::Right => (15, -4, -1),
        };

        let start = start as isize;
        let ostride = ostride as isize;
        let istride = istride as isize;
        assert!(HEIGHT == WIDTH);

        let mut new_board = [0; WIDTH*HEIGHT];
        let mut all_points = 0.0;    //  points we accumulate
        let mut any_changed = false;  // did any of the vectors change?

        for outer in 0..(HEIGHT as isize) {
            let mut vec = Vec::with_capacity(HEIGHT);
            for inner in 0..(HEIGHT as isize) {
                let idx = start + outer*ostride + inner*istride;
                vec.push(board[idx as usize]);
            }

            let (merged_vec, points, changed) = TwoFortyEight::merge_vec(&vec);
            all_points += points;
            any_changed |= changed;

            for inner in 0..(HEIGHT as isize) {
                let idx = start + outer*ostride + inner*istride;
                new_board[idx as usize] = merged_vec[inner as usize];
            }
        }
        if any_changed {
            (new_board, Some(all_points))
        } else {
            (new_board, None)
        }
    }

    ///
    pub fn get_tile(&self, row: usize, col: usize) -> u16 {
        let idx = row * WIDTH + col;
        self.board[idx]
    }

    ///
    pub fn set_tile(&mut self, row: usize, col: usize, num: u16) {
        let idx = row * WIDTH + col;
        self.board[idx] = num;
    }

    /// Check whether the currend board is full.
    pub fn board_full(&self) -> bool {
        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                if self.get_tile(row, col) == 0 {
                    return false;
                }
            }
        }
        true
    }

    /// Place a 2 into some random empty tile
    pub fn random_spawn(&mut self) {
        assert!(!self.board_full());

        loop {
            let row = self.rng.gen::<usize>() % HEIGHT;
            let col = self.rng.gen::<usize>() % WIDTH;
            if self.get_tile(row, col) == 0 {
                self.set_tile(row, col, 2);
                break;
            }
        }

        // This is much slower... even for nearly full borads.
        // And not correct, because it's not useing self.rng!
        // let candidates = self.board.iter()
        //     .enumerate()
        //     .filter(|&(_, &n)| n == 0)
        //     .map(|(i, &_)| i)
        //     .collect::<Vec<_>>();
        //
        // let idx = choose_random(&candidates);
        // self.board[*idx as usize] = 2;
    }
}

impl Game<Action> for TwoFortyEight {

    /// Return a list with all allowed actions given the current game state.
    fn allowed_actions(&self) -> Vec<Action> {
        let actions = vec![Action::Up, Action::Down, Action::Left, Action::Right];

        actions.iter().map(|t| *t).filter(|&a| {
                let (_, points) = TwoFortyEight::shift_and_merge(self.board, &a);
                match points {
                    Some(_) => true,
                    None => false
                }
            }).collect()
    }

    /// Change the current game state according to the given action.
    fn make_move(&mut self, action: &Action) {
        let (new_board, points) = TwoFortyEight::shift_and_merge(self.board, action);
        self.score += points.expect("Illegal move");
        self.moves += 1;
        self.board = new_board;
        self.random_spawn()
    }

    /// Reward for the player when reaching the current game state.
    fn reward(&self) -> f32 {
        self.score
    }

    /// Derterminize the game
    fn set_rng_seed(&mut self, seed: u32) {
        self.rng = XorShiftRng::from_seed([seed+0, seed+1, seed+2, seed+3]);
    }
}


impl fmt::Display for TwoFortyEight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // XXX could be much nicer XXX
        try!(writeln!(f, "Moves={} Score={}:", self.moves, self.score));
        for _ in 0..WIDTH {
            try!(write!(f, "|{: ^5}", "-----"));
        }
        try!(f.write_str("|"));
        for row in 0..HEIGHT {
            try!(f.write_str("\n"));
            for _ in 0..WIDTH {
                try!(write!(f, "|{: ^5}", ""));
            }
            try!(f.write_str("|\n"));
            for col in 0..WIDTH {
                let tile =  self.get_tile(row, col);
                if tile == 0 {
                    try!(write!(f, "|{: ^5}", ""));
                } else {
                    try!(write!(f, "|{: ^5}", tile));
                }
            }
            try!(f.write_str("|\n"));
            for _ in 0..WIDTH {
                try!(write!(f, "|{: ^5}", ""));
            }
            try!(f.write_str("|\n"));
            for _ in 0..WIDTH {
                try!(write!(f, "|{: ^5}", "-----"));
            }
            try!(f.write_str("|"));
        }
        f.write_str("")
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use test::Bencher;

    use mcts::*;
    use twofortyeight::*;

    #[test]
    fn test_new() {
        let game = TwoFortyEight::new();

        assert_eq!(game.reward(), 0.);
    }

    #[test]
    fn test_display() {
        let coords = vec![(0, 1, 2), (2, 2, 4), (3, 1, 2048)];

        // Set given tiles
        let mut game = TwoFortyEight::new();
        for (row, col, num) in coords.clone() {
            game.set_tile(row, col, num);
        }

        println!("{}", game);
    }

    #[test]
    fn test_setget_tile() {
        let mut game = TwoFortyEight::new();

        let coords = vec![(0, 1, 2), (2, 2, 4), (3, 1, 16)];

        // Set given tiles
        for (row, col, num) in coords.clone() {
            game.set_tile(row, col, num);
        }

        // Check given tiles
        for (row, col, num) in coords.clone() {
            assert_eq!(game.get_tile(row, col), num);
        }
    }

    #[test]
    fn test_random_spawn() {
        let mut game = TwoFortyEight::new_empty();

        for _ in 0..WIDTH*HEIGHT {
            assert!(!game.board_full());
            game.random_spawn();
        }
        assert!(game.board_full());
    }

    #[test]
    fn test_merge_vec() {
        let test_cases = vec![
            (vec![0]               , vec![0]),
            (vec![2]               , vec![2]),
            (vec![0, 2]            , vec![2, 0]),
            (vec![2, 2]            , vec![4, 0]),
            (vec![2, 8, 2]         , vec![2, 8, 2]),
            (vec![2, 0, 4, 4]      , vec![2, 8, 0, 0]),
            (vec![2, 4, 2, 2]      , vec![2, 4, 4, 0]),
            (vec![2, 2, 2, 0]      , vec![4, 2, 0, 0]),
            (vec![1, 2, 0, 0, 4]   , vec![1, 2, 4, 0, 0]),
            (vec![1, 2, 2, 0, 4]   , vec![1, 4, 4, 0, 0]),
            (vec![1, 2, 2, 2, 4]   , vec![1, 4, 2, 4, 0]),
            (vec![0, 2, 0, 2, 0]   , vec![4, 0, 0, 0, 0]),
            (vec![0, 0, 0, 0, 0]   , vec![0, 0, 0, 0, 0]),
            (vec![2, 2, 2, 2, 2]   , vec![4, 4, 2, 0, 0]),
            (vec![2, 0, 2, 0, 4]   , vec![4, 4, 0, 0, 0]),
            (vec![2, 2, 0, 4, 4]   , vec![4, 8, 0, 0, 0]),
            (vec![2, 2, 4, 4, 4, 4], vec![4, 8, 8, 0, 0]),
            (vec![4, 0, 0, 0, 0, 4], vec![8, 0, 0, 0, 0, 0]),
        ];

        /*
        let test_cases = (
            ((2, 0, 4, 4), (2, 8, 0, 0)),
            ((2, 4, 2, 2), (2, 4, 4, 0)),
            ((2, 2, 2, 0), (4, 2, 0, 0)),
            ((0, 2, 2, 2), (4, 2, 0, 0)),
            ((2, 4, 2, 0), (2, 4, 2, 0)),
            ((0, 0, 2, 0), (2, 0, 0, 0)),
            ((0, 0, 0, 2), (2, 0, 0, 0)),
            ((4, 2, 2, 2), (4, 4, 2, 0)),
            ((0, 4, 2, 0), (4, 2, 0, 0)),
            ((4, 0, 0, 4), (8, 0, 0, 0)),
            ((4, 4, 4, 2), (8, 4, 2, 0)),
            ((2, 2, 4, 8), (4, 4, 8, 0)),
        );*/

        for (input, should) in test_cases {
            let  output = TwoFortyEight::merge_vec(&input);
            println!("merge_vec({:?}) => {:?}  (should be {:?})", input, output, should);
        }
    }

    #[test]
    fn test_shift_and_merge() {
        let mut game = TwoFortyEight::new_empty();
        game.set_tile(2, 2, 4);

        let actions = vec![Action::Down, Action::Right, Action::Up, Action::Left];
        for a in &actions {
            let (board, points) = TwoFortyEight::shift_and_merge(game.board, a);
            assert!(points.unwrap() == 0.0);
            game.board = board;
            println!("{}", game);
        }
        assert!(game.get_tile(0, 0) == 4);
    }

    #[test]
    fn test_playout() {
        let game = TwoFortyEight::new();
        let final_game = playout(&game);
        println!("{}", final_game);
    }

    #[test]
    fn test_mcts() {
        let game = TwoFortyEight::new();
        let mut mcts = MCTS::new(&game, 5);

        mcts.search(25, 1.);
        let action = mcts.best_action();
        action.expect("should give some action");
    }

    #[bench]
    fn bench_playout(b: &mut Bencher) {
        let game = TwoFortyEight::new();
        b.iter(|| playout(&game));
    }

    #[bench]
    fn bench_allowed_actions(b: &mut Bencher) {
        let game = TwoFortyEight::new();
        b.iter(|| game.allowed_actions());
    }

    #[bench]
    fn random_spawn_until_full(b: &mut Bencher) {
        b.iter(|| {
            let mut game = TwoFortyEight::new();
            while !game.board_full() {
                game.random_spawn()
            }
        })
    }

    #[bench]
    fn board_full(b: &mut Bencher) {
        let mut game = TwoFortyEight::new();

        for _ in 0..(WIDTH*HEIGHT/2) {
            game.random_spawn()
        }

        b.iter(|| game.board_full())
    }
}
