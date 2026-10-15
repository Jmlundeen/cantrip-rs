//! The game log as the search tree (docs/engine-design.md, "The game log is the search tree").
//!
//! A node is what one answer did: the answer, the journal entries its run wrote and the events it recorded. The real
//! game is one path from the root; a search branch is a sibling path that shares everything above it. Nodes are only
//! ever added.
//!
//! The game holds one working state and moves it through the tree as a checkout does: undo up to the common ancestor,
//! redo down to the target. For the nodes on the current path, the writes are the working state's own journal and the
//! events its own history, so nothing about them is copied; a node keeps its writes and events itself only while the
//! working state is off it. Going up undoes each write and keeps the entry that redoes it; going down applies those and
//! puts the original entries back on the journal. Answering a question a node already has an answer for moves to that
//! child instead of playing it out again.

use cantrip_api::*;
use cantrip_state::{Mark, Undo};

/// A node of the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u32);

impl NodeId {
    pub const ROOT: NodeId = NodeId(0);
}

/// What a node holds while the working state is not on it: its writes, the entries that redo them, its events.
#[derive(Clone, Debug, Default)]
struct Away {
    undos: Vec<Undo>,
    redos: Vec<Undo>,
    events: Vec<Record>,
}

#[derive(Clone, Debug)]
struct Node {
    parent: Option<NodeId>,
    /// The answer that made it, to the request with this id, from this player (none for the root).
    answer: Option<(u32, PlayerId, Answer)>,
    children: Vec<NodeId>,
    /// Where its writes and events begin: the journal mark and history length at its start, on the path.
    start: (Mark, usize),
    /// Where they end, once it has been left; none while the game is still running in it.
    end: Option<(Mark, usize)>,
    away: Option<Away>,
}

/// The tree of every position the game has been in and every branch a search has looked at.
#[derive(Clone, Debug)]
pub struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            nodes: vec![Node { parent: None, answer: None, children: Vec::new(), start: (Mark(0), 0), end: None, away: None }],
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.nodes[node.0 as usize].parent
    }

    pub fn children(&self, node: NodeId) -> &[NodeId] {
        &self.nodes[node.0 as usize].children
    }

    /// The answer that led to a node (the request's id, who answered, the answer).
    pub fn answer(&self, node: NodeId) -> Option<&(u32, PlayerId, Answer)> {
        self.nodes[node.0 as usize].answer.as_ref()
    }

    /// The child of `node` made by this answer, if it has been given before.
    pub fn child(&self, node: NodeId, request: u32, player: PlayerId, answer: &Answer) -> Option<NodeId> {
        self.children(node).iter().copied().find(|&c| {
            matches!(&self.nodes[c.0 as usize].answer, Some((r, p, a)) if *r == request && *p == player && a == answer)
        })
    }

    /// The path from the root to `node`, root first.
    pub fn path(&self, node: NodeId) -> Vec<NodeId> {
        let mut path = vec![node];
        let mut at = node;
        while let Some(p) = self.parent(at) {
            path.push(p);
            at = p;
        }
        path.reverse();
        path
    }

    pub(crate) fn add(&mut self, parent: NodeId, answer: (u32, PlayerId, Answer), start: (Mark, usize)) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(Node { parent: Some(parent), answer: Some(answer), children: Vec::new(), start, end: None, away: None });
        self.nodes[parent.0 as usize].children.push(id);
        id
    }

    pub(crate) fn start(&self, node: NodeId) -> (Mark, usize) {
        self.nodes[node.0 as usize].start
    }

    pub fn end(&self, node: NodeId) -> Option<(Mark, usize)> {
        self.nodes[node.0 as usize].end
    }

    pub(crate) fn close(&mut self, node: NodeId, end: (Mark, usize)) {
        let n = &mut self.nodes[node.0 as usize];
        if n.end.is_none() {
            n.end = Some(end);
        }
    }

    pub(crate) fn put_away(&mut self, node: NodeId, undos: Vec<Undo>, redos: Vec<Undo>, events: Vec<Record>) {
        self.nodes[node.0 as usize].away = Some(Away { undos, redos, events });
    }

    pub(crate) fn take_back(&mut self, node: NodeId) -> (Vec<Undo>, Vec<Undo>, Vec<Record>) {
        let away = self.nodes[node.0 as usize].away.take().expect("a node off the path holds its writes");
        (away.undos, away.redos, away.events)
    }
}

impl Default for Tree {
    fn default() -> Tree {
        Tree::new()
    }
}
