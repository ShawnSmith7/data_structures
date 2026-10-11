pub mod cursor;
mod firebase_array;

use std::{fmt, ops};
use fmt::{Display, Formatter};
use ops::Index;
use serde::{Serialize, Deserialize};

use NodeId::*;
use ArenaTreeError::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeId {
    Root,
    Id(usize),
}

impl Default for NodeId {
    fn default() -> Self {
        Root
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, bound(deserialize = "T: Deserialize<'de> + Default"))]
pub struct Node<T, const N: usize> {
    pub val: T,
    pub parent_id: NodeId,
    #[serde(with = "firebase_array")]
    pub branches: [Option<usize>; N],
}

impl<T, const N: usize> Default for Node<T, N>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            val: T::default(),
            parent_id: Root,
            branches: [None; N],
        }
    }
}

impl<T, const N: usize> Node<T, N> {
    pub fn new(val: T, parent_id: NodeId) -> Self {
        Self {
            val,
            parent_id,
            branches: [None; N],
        }
    }

    pub fn get_branch(&self, branch: usize) -> Result<&Option<usize>, ArenaTreeError<N>> {
        self.branches.get(branch).ok_or(BranchOutOfBounds(branch))
    }

    pub fn get_branch_mut(&mut self, branch: usize) -> Result<&mut Option<usize>, ArenaTreeError<N>> {
        self.branches.get_mut(branch).ok_or(BranchOutOfBounds(branch))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, bound(deserialize = "T: Deserialize<'de> + Default"))]
pub struct ArenaTree<T, const N: usize> {
    root: Node<T, N>,
    nodes: Vec<Node<T, N>>,
    free_ids: Vec<usize>,
}

impl<T, const N: usize> Index<NodeId> for ArenaTree<T, N> {
    type Output = Node<T, N>;

    fn index(&self, id: NodeId) -> &Self::Output {
        self.get(id)
    }
}

impl<T, const N: usize> ArenaTree<T, N> {
    pub fn new(val: T) -> Self {
        Self {
            root: Node::new(val, Root),
            nodes: Vec::default(),
            free_ids: Vec::default(),
        }
    }

    pub fn get_checked(&self, id: NodeId) -> Result<&Node<T, N>, ArenaTreeError<N>> {
        match id {
            Root => Ok(&self.root),
            Id(id) => if self.free_ids.contains(&id) {
                Err(NodeDoesNotExist(id))
            } else {
                self.nodes.get(id).ok_or(NodeDoesNotExist(id))
            }
        }
    }

    pub fn get(&self, id: NodeId) -> &Node<T, N> {
        self.get_checked(id).expect("Invalid NodeId passed to ArenaTree::get")
    }

    pub unsafe fn get_unchecked(&self, id: NodeId) -> &Node<T, N> {
        match id {
            Root => &self.root,
            Id(id) => unsafe {
                self.nodes.get_unchecked(id)
            },
        }
    }

    fn get_mut(&mut self, id: NodeId) -> Result<&mut Node<T, N>, ArenaTreeError<N>> {
        match id {
            Root => Ok(&mut self.root),
            Id(id) => if self.free_ids.contains(&id) {
                Err(NodeDoesNotExist(id))
            } else {
                self.nodes.get_mut(id).ok_or(NodeDoesNotExist(id))
            }
        }
    }

    unsafe fn get_unchecked_mut(&mut self, id: NodeId) -> &mut Node<T, N> {
        match id {
            Root => &mut self.root,
            Id(id) => unsafe {
                self.nodes.get_unchecked_mut(id)
            },
        }
    }

    pub fn set_checked(&mut self, id: NodeId, val: T) -> Result<(), ArenaTreeError<N>> {
        self.get_mut(id)?.val = val;

        Ok(())
    }

    pub fn set(&mut self, id: NodeId, val: T) {
        self.set_checked(id, val).expect("Invalid NodeId passed to ArenaTree::set");
    }

    pub unsafe fn set_unchecked(&mut self, id: NodeId, val: T) {
        unsafe {
            self.get_unchecked_mut(id).val = val;
        }
    }

    pub fn add_node_checked(&mut self, val: T, parent_id: NodeId, branch: usize) -> Result<usize, ArenaTreeError<N>> {
        if self.get_checked(parent_id)?.get_branch(branch)?.is_some() {
            return Err(BranchOverwrite { id: parent_id, branch });
        }

        unsafe {
            Ok(self.add_node_unchecked(val, parent_id, branch))
        }
    }

    pub fn add_node(&mut self, val: T, parent_id: NodeId, branch: usize) -> usize {
        self.add_node_checked(val, parent_id, branch).expect("Invalid input passed to ArenaTree::add_node")
    }

    pub unsafe fn add_node_unchecked(&mut self, val: T, parent_id: NodeId, branch: usize) -> usize {
        let node = Node::new(val, parent_id);
        let id = match self.free_ids.pop() {
            None => {
                let id = self.nodes.len();
                self.nodes.push(node);
                id
            },
            Some(id) => {
                unsafe {
                    *self.nodes.get_unchecked_mut(id) = node;
                }
                id
            }
        };

        unsafe {
            *self.get_unchecked_mut(parent_id).branches.get_unchecked_mut(branch) = Some(id);
        }

        id
    }

    unsafe fn delete_branch_inner(&mut self, child_id: usize, id: NodeId, branch: usize) {
        unsafe {
            for branch in 0..N {
                self.delete_branch_recursive(child_id, branch);
            }

            *self.get_unchecked_mut(id).branches.get_unchecked_mut(branch) = None;
        }
        self.free_ids.push(child_id);
    }

    pub fn delete_branch_checked(&mut self, id: NodeId, branch: usize) -> Result<(), ArenaTreeError<N>> {
        match *self.get_checked(id)?.get_branch(branch)? {
            None => Err(BranchDoesNotExist { branch, id }),
            Some(child_id) => unsafe {
                self.delete_branch_inner(child_id, id, branch);
                Ok(())
            },
        }
    }

    pub fn delete_branch(&mut self, id: NodeId, branch: usize) {
        self.delete_branch_checked(id, branch).expect("Invalid state passed for ArenaTree::delete_branch");
    }

    pub unsafe fn delete_branch_unchecked(&mut self, id: NodeId, branch: usize) {
        unsafe {
            let child_id = self.get_unchecked(id).branches.get_unchecked(branch).unwrap();
            self.delete_branch_inner(child_id, id, branch);
        }
    }

    unsafe fn delete_branch_recursive(&mut self, id: usize, branch: usize) {
        unsafe {
            if let Some(id) = *self.nodes.get_unchecked(id).branches.get_unchecked(branch) {
                for branch in 0..N {
                    self.delete_branch_recursive(id, branch);
                }

                self.free_ids.push(id);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaTreeError<const N: usize> {
    NodeDoesNotExist(usize),
    BranchOutOfBounds(usize),
    BranchDoesNotExist {
        branch: usize,
        id: NodeId,
    },
    BranchOverwrite {
        id: NodeId,
        branch: usize,
    },
    RootHasNoParent,
}

impl<const N: usize> Display for ArenaTreeError<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NodeDoesNotExist(id) =>
                write!(f, "Node does not exist at id {id}"),
            BranchOutOfBounds(branch) =>
                write!(f, "Branch {branch} is out of bounds {N}"),
            BranchDoesNotExist { branch, id } =>
                write!(f, "Branch {branch} does not exist at {id:?}"),
            BranchOverwrite { id, branch } =>
                write!(f, "Cannot overwrite branch {branch} at {id:?}"),
            RootHasNoParent =>
                write!(f, "The root node does not have a parent"),
        }
    }
}

impl<const N: usize> std::error::Error for ArenaTreeError<N> {}