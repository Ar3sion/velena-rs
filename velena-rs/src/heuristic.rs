/*
 heurist.c, pnsearch.c
*/

use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
use crate::board::Board;

#[derive(Copy, Clone)]
enum NodeType {
    And,
    Or
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum NodeState {
    NotEvaluated,
    Evaluated,
    Expanded
}

enum NodeValue {
    Disproved,
    Unknown,
    Proved
}

struct Node {
    board: Board,
    child: [Option<Rc<RefCell<Node>>>; Board::WIDTH],
    parent: [Option<Weak<RefCell<Node>>>; Board::HEIGHT],
    state: NodeState,
    value: NodeValue,
    proof: usize,
    disproof: usize,
    node_type: NodeType
}

impl Node {
    fn new(board: Board, node_type: NodeType) -> Self {
        Self {
            board,
            child: Default::default(),
            parent: Default::default(),
            state: NodeState::NotEvaluated,
            value: NodeValue::Unknown,
            proof: 0,
            disproof: 0,
            node_type
        }
    }
    
    fn evaluate(&mut self, root_node_type: NodeType, fight: bool) -> Option<usize> {
        self.state = NodeState::Evaluated;

        if self.board.is_full() {
            self.value = if fight {
                match root_node_type {
                    NodeType::And => NodeValue::Disproved,
                    NodeType::Or => NodeValue::Proved
                }
            } else {
                match root_node_type {
                    NodeType::And => NodeValue::Proved,
                    NodeType::Or => NodeValue::Disproved
                }
            };

            return None;
        }

        if let Some(win_immediately) = self.board.get_winning_move() {
            self.value = match self.node_type {
                NodeType::And => NodeValue::Disproved,
                NodeType::Or => NodeValue::Proved
            };
            return Some(win_immediately);
        }

        self.value = NodeValue::Unknown;
        None
    }

    fn set_proof_and_disproof_numbers(&mut self) {
        const MAX_VALUE: usize = 200000000;

        match self.state {
            NodeState::Expanded => {
                match self.node_type {
                    NodeType::And => {
                        self.proof = 0;
                        self.disproof = MAX_VALUE;

                        for column in 0..Board::WIDTH {
                            if let Some(child) = self.child[column].as_ref() {
                                let child_borrow = child.borrow();
                                //We need to prove every child
                                self.proof += child_borrow.proof;
                                //We only need to disprove one child
                                self.disproof = Ord::min(self.disproof, child_borrow.disproof);
                            }
                        }

                        if self.disproof == 0 {
                            //The node is disproved, it is impossible to prove
                            self.proof = MAX_VALUE;
                        }
                    }
                    NodeType::Or => {
                        self.proof = MAX_VALUE;
                        self.disproof = 0;

                        for column in 0..Board::WIDTH {
                            if let Some(child) = self.child[column].as_ref() {
                                let child_borrow = child.borrow();
                                //We only need to prove one child
                                self.proof = Ord::min(self.proof, child_borrow.proof);
                                //We only need to disprove every child
                                self.disproof += child_borrow.disproof;
                            }
                        }

                        if self.proof == 0 {
                            //The node is proved, it is impossible to disprove
                            self.disproof = MAX_VALUE;
                        }
                    }
                }
            },
            NodeState::Evaluated => {
                match self.value {
                    NodeValue::Disproved => {
                        self.proof = MAX_VALUE;
                        self.disproof = 0;
                    }
                    NodeValue::Unknown => {
                        let mut child_count = 0;
                        for column in 0..Board::WIDTH {
                            if self.board.can_play(column) {
                                child_count += 1;
                            }
                        }
                        assert_ne!(child_count, 0); //Leaf with unknown evaluated value
                        match self.node_type {
                            NodeType::And => {
                                self.proof = child_count;
                                self.disproof = 1;
                            },
                            NodeType::Or => {
                                self.proof = 1;
                                self.disproof = child_count;
                            }
                        }
                    }
                    NodeValue::Proved => {
                        self.proof = 0;
                        self.disproof = MAX_VALUE;
                    }
                }
            },
            NodeState::NotEvaluated => panic!()
        }
    }
    
    fn update_ancestors(&mut self) {
        self.set_proof_and_disproof_numbers();
        for parent in &self.parent {
            if let Some(parent) = parent {
                parent.upgrade().unwrap().borrow_mut().update_ancestors();
            }
        }
    }
}

fn select_most_proving_node<'a>(mut node: Rc<RefCell<Node>>, best_move: &mut Option<usize>) -> Rc<RefCell<Node>> {
    const NODE_SEQUENCE_ORDER: [usize; Board::WIDTH] = [3, 2, 4, 5, 1, 0, 6];

    let mut depth = 0;
    while node.borrow().state == NodeState::Expanded {
        let mut good_child = None;
        {
            let node_borrow = node.borrow();
            match node_borrow.node_type {
                NodeType::And => {
                    for column in NODE_SEQUENCE_ORDER {
                        if let Some(child) = node_borrow.child[column].as_ref() {
                            if child.borrow().proof == node_borrow.proof {
                                good_child = Some(column);
                                break;
                            }
                        }
                    }
                }
                NodeType::Or => {
                    for column in NODE_SEQUENCE_ORDER {
                        if let Some(child) = node_borrow.child[column].as_ref() {
                            if child.borrow().disproof == node_borrow.disproof {
                                good_child = Some(column);
                                break;
                            }
                        }
                    }
                }
            }
        }
        let good_child = good_child.unwrap();
        if depth == 0 {
            *best_move = Some(good_child);
        }
        let child = node.borrow().child[good_child].as_ref().unwrap().clone();
        node = child;
        depth += 1;
    }

    node
}

fn develop(node: Rc<RefCell<Node>>, nodes: &mut HashMap<u64, Rc<RefCell<Node>>>, 
           root_node_type: NodeType, fight: bool, nodes_expanded: &mut usize) {
    let mut node_borrow = node.borrow_mut();

    node_borrow.state = NodeState::Expanded;

    for column in 0..Board::WIDTH {
        if let Ok(mut new_board) = node_borrow.board.make_move(column) {
            let mut symmetric = new_board.symmetric_board();

            let backtrace = if symmetric < new_board {
                mem::swap(&mut new_board, &mut symmetric);
                Board::WIDTH - 1 - column
            } else {
                column
            };
            
            match nodes.entry(new_board.key()) {
                Entry::Occupied(e) => {
                    let cached_child = e.get();
                    cached_child.borrow_mut().parent[backtrace] = Some(Rc::downgrade(&node));
                    node_borrow.child[column] = Some(cached_child.clone());
                }
                Entry::Vacant(e) => {
                    *nodes_expanded += 1;
                    
                    let mut child = Node::new(new_board, match node_borrow.node_type {
                        NodeType::And => NodeType::Or,
                        NodeType::Or => NodeType::And
                    });
                    child.parent[backtrace] = Some(Rc::downgrade(&node));
                    let child = Rc::new(RefCell::new(child));
                    e.insert(child.clone());
                    node_borrow.child[column] = Some(child);
                }
            }
        }
    }

    for child in &node_borrow.child {
        if let Some(child) = child {
            let mut child_borrow = child.borrow_mut();
            child_borrow.evaluate(root_node_type, fight);
            child_borrow.set_proof_and_disproof_numbers();
        }
    }
}

fn heuristic_proof_number_search(root: Rc<RefCell<Node>>, nodes: &mut HashMap<u64, Rc<RefCell<Node>>>, fight: bool) -> Option<usize> {
    const MAX_NODE_COUNT: usize = 2800;
    
    let mut nodes_expanded = 0;
    
    let (mut best_move, root_type) = {
        let mut root_borrow = root.borrow_mut();
        let root_type = root_borrow.node_type;
        let best_move = root_borrow.evaluate(root_type, fight);
        root_borrow.set_proof_and_disproof_numbers();
        (best_move, root_type)
    };
    
    //Loop until the root is proved, disproved, or we run out of computing resources
    while {
        let root_borrow = root.borrow();
        root_borrow.proof != 0 && root_borrow.disproof != 0
    } && nodes_expanded <= MAX_NODE_COUNT {
        let most_proving_node = select_most_proving_node(root.clone(), &mut best_move);
        develop(most_proving_node.clone(), nodes, root_type, fight, &mut nodes_expanded);
        most_proving_node.borrow_mut().update_ancestors();
    }

    {
        let mut root_borrow = root.borrow_mut();
        if root_borrow.proof == 0 {
            root_borrow.deref_mut().value = NodeValue::Proved;
        } else if root_borrow.disproof == 0 {
            root_borrow.deref_mut().value = NodeValue::Disproved;
        } else {
            root_borrow.deref_mut().value = NodeValue::Unknown;
        }
    }
    
    best_move
}

pub fn heuristic_best_play(board: Board, fight: bool) -> Option<usize> {
    let symmetric = board.symmetric_board();
    let is_symmetric = symmetric < board;
    let node_board = if is_symmetric {
        symmetric
    } else {
        board
    };
    let key = node_board.key();
    let root_node = Rc::new(RefCell::new(
        Node::new(node_board, NodeType::Or)
    ));
    
    let mut nodes = HashMap::new();
    nodes.insert(key, root_node.clone());
    let best_move = heuristic_proof_number_search(root_node.clone(), &mut nodes, fight);
    
    let root_node_borrow = root_node.borrow();
    match root_node_borrow.deref().value {
        NodeValue::Disproved => None,
        NodeValue::Unknown => None,
        NodeValue::Proved => Some(if is_symmetric {
            Board::WIDTH - 1 - best_move.unwrap()
        } else {
            best_move.unwrap()
        })
    }
}