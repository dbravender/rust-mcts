
extern crate test;

use std::fmt;
use std::f32;
use std::fmt::Debug;

use utils::{choose_random};


/// A `Game` represets a game state.
pub trait Game<A: GameAction> : Clone {

    /// Return a list with all allowed actions given the current game state.
    fn allowed_actions(&self) -> Vec<A>;

    /// Change the current game state according to the given action.
    fn make_move(&mut self, action: &A);

    /// Reward for the player when reaching the current game state.
    fn reward(&self) -> f32;
}

/// A `GameAction` represents a move in a game.
pub trait GameAction: Debug+Clone+Copy+PartialEq {}


/// Perform a random playout.
///
/// Start with an initial game state and perform random actions from 
/// `game.allowed_actions` until a game-stateis reached that does not have
/// any `allowed_actions`.
pub fn playout<G: Game<A>, A: GameAction>(initial: &G) -> G {
    let mut game = initial.clone();

    let mut potential_moves = game.allowed_actions();
    while potential_moves.len() > 0 {
        let action = choose_random(&potential_moves).clone();
        game.make_move(&action);
        potential_moves = game.allowed_actions();
    }
    game
}

/// Calculate the expected reward based on random playouts.
pub fn expected_reward<G: Game<A>, A: GameAction>(game: &G, n_samples: usize) -> f32 {
    let mut score_sum: f32 = 0.0;

    for _ in 0..n_samples {
        score_sum += playout(game).reward();
    }
    (score_sum as f32) / (n_samples as f32)
}


//////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct TreeNode<A: GameAction> {
    action: Option<A>,                  // how did we get here
    children: Vec<TreeNode<A>>,         // next steps we investigated
    terminal_state: bool,               // is this a leaf of the tree?
    fully_expanded: bool,               // are there unexplored actions?
    n: f32, q: f32                      // statistics for this game state
}


impl<A> TreeNode<A> where A: GameAction {

    /// Create and initialize a new TreeNode
    pub fn new(action: Option<A>) -> TreeNode<A> {
        TreeNode::<A> {
            action: action,
            children: Vec::new(),
            terminal_state: false,
            fully_expanded: false,
            n: 0., q: 0. }
    }

    /// Find the best child accoring to UCT1
    pub fn best_child(&mut self, c: f32) -> Option<&mut TreeNode<A>> {
        let mut best_value :f32 = f32::NEG_INFINITY;
        let mut best_child :Option<&mut TreeNode<A>> = None;

        for child in &mut self.children {
            let value = child.q / child.n + c*(2.*self.n.ln()/child.n).sqrt();
            if value > best_value {
                best_value = value;
                best_child = Some(child);
            }
        }
        best_child
    }

    /// Add a child to the current node with an previously
    /// unexplored action.
    /// XXX Use HashSet? Use iterators? XXX
    pub fn expand<G>(&mut self, game: &G) -> Option<&mut TreeNode<A>>
        where G: Game<A> {
        let allowed_actions = game.allowed_actions();

        if allowed_actions.len() == 0 {
            self.fully_expanded = true;
            self.terminal_state = true;
            return None;
        }

        let mut child_actions : Vec<A> = Vec::new();
        for child in &self.children {
            match child.action {
                Some(a) => child_actions.push(a),
                None    => panic!("Child node without action"),
            }
        }

        // Find untried actions
        let mut candidate_actions = Vec::new();
        for action in &allowed_actions {
            if !child_actions.contains(action) {
                candidate_actions.push(action);
            }
        }

        if candidate_actions.len() == 1 {
            self.fully_expanded = true;
        }

        // XXX Select random one XXX
        //let action = candidate_actions[0].clone();
        let action = *choose_random(&candidate_actions).clone();

        self.children.push(TreeNode::new(Some(action)));
        self.children.last_mut()
    }

    /// Recursively perform an MCTS iteration.
    pub fn iteration<G>(&mut self, game: &mut G, c: f32) -> f32
        where G: Game<A>+Clone {

        if self.terminal_state {
            let delta = game.reward();
            self.n += 1.;
            self.q += delta;
            return delta;
        };

        if self.fully_expanded {
            // Choose child
            let mut delta;
            {
                let child = self.best_child(c).unwrap();

                // Recurse into chosen one...
                game.make_move(&child.action.unwrap());
                delta = child.iteration(game, c);
            }

            // Update my statistics
            self.n += 1.;
            self.q += delta;
            return delta;
        } else {
            let mut delta :f32;
            {
                let child = self.expand(game);
                match child {
                    Some(child) => {
                            game.make_move(&child.action.unwrap());
                            let game = playout(game);
                            delta = game.reward();
                            child.n += 1.;
                            child.q += delta },
                    None => delta = game.reward()
                }
            }
            self.n += 1.;
            self.q += delta;
            return delta;
        };
    }
}


//////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MCTS<G: Game<A>, A: GameAction> {
    root: TreeNode<A>,
    game: G
}

impl <G: Game<A>, A: GameAction> MCTS<G, A> {

    /// Create a new MCTS solver
    pub fn new(game: &G) -> MCTS<G, A> {
        let game = game.clone();
        let root = TreeNode::new(None);
        MCTS {root: root, game: game}
    }

    pub fn search(&mut self, game: &G, n_samples: usize, c: f32) -> Vec<A> {
        let root = &mut self.root;

        // Perform MCTS iterations
        for _ in 0..n_samples {
            root.iteration(&mut game.clone(), c);
        }

        // Find best path
        let mut best_actions = Vec::new();
        let mut node = root.best_child(0.);
        while let Some(child) = node {
            best_actions.push(child.action.unwrap());
            node = child.best_child(0.)
        }

        best_actions
    }
}

impl<G: Game<A>, A: GameAction> fmt::Display for MCTS<G, A> {

    /// Output a nicely indented tree
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        // Nested definition for recursive formatting
        fn fmt_subtree<M>(f: &mut fmt::Formatter, node: &TreeNode<M>, indent_level :i32) -> fmt::Result
            where M: GameAction {
            for _ in (0..indent_level) {
                try!(f.write_str("    "));
            }
            match node.action {
                Some(a)  => try!(writeln!(f, "{:?} q={} n={}", a, node.q, node.n)),
                None     => try!(writeln!(f, "Root q={} n={}", node.q, node.n))
            }
            for child in &node.children {
                try!(fmt_subtree(f, child, indent_level+1));
            }
            write!(f, "")
        }

        fmt_subtree(f, &self.root, 0)
    }
}


/////////////////////////////////////////////////////////////////////////////
// Unittests

#[cfg(test)]
mod tests {
    use test::Bencher;

    use mcts::*;
    use minigame::MiniGame;
    // use tictactoe::{TicTacToe, Action, Player, GameStatus};

    #[test]
    fn test_playout() {
        let game = MiniGame::new();
        let game = playout(&game);
        println!("Final: {:?}", game);
    }

    #[test]
    fn test_expand() {
        let game = MiniGame::new();
        let mut mcts = MCTS::new(&game);

        mcts.root.expand(&game);
        mcts.root.expand(&game);
        {
            let v = mcts.root.expand(&game).unwrap();
            v.expand(&game);
        }

        println!("MCTS some expands:\n{}", &mcts);
    }

    #[test]
    fn test_mcts() {
        let game = MiniGame::new();
        let mut mcts = MCTS::new(&game);

        println!("MCTS on new game: {:?}", mcts);

        for i in 0..5 {
            mcts.root.iteration(&mut game.clone(), 1.0);
            println!("After {} iteration(s):\n{}", i, mcts);
        }
    }

    #[bench]
    fn bench_playout(b: &mut Bencher) {
        let game = MiniGame::new();
        b.iter(|| playout(&game))
    }

    #[bench]
    fn bench_expected(b: &mut Bencher) {
        let game = MiniGame::new();
        b.iter(|| expected_reward(&game, 100))
    }

    #[test]
    fn test_search() {
        let game = MiniGame::new();
        let mut mcts = MCTS::new(&game);

        let actions = mcts.search(&game.clone(), 100, 1.);
        println!("Search result: {:?}", actions);
    }

    #[bench]
    fn bench_iterations(b: &mut Bencher) {
        let game = MiniGame::new();
        let mut mcts = MCTS::new(&game);

        b.iter(|| mcts.root.iteration(&mut game.clone(), 1.0))
    }

}
