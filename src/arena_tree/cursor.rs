use super::{NodeId, ArenaTree, Node, ArenaTreeError, Result};
use std::marker::PhantomData;

use NodeId::*;
use ArenaTreeError::*;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct ArenaTreeCursor<'a, T, const N: usize> {
    id: NodeId,
    _marker: PhantomData<&'a T>,
}

impl<'a, T, const N: usize> Clone for ArenaTreeCursor<'a, T, N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T, const N: usize> Copy for ArenaTreeCursor<'a, T, N> {}

impl<'a, T, const N: usize> ArenaTreeCursor<'a, T, N> {
    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn get_checked(&self, tree: &'a ArenaTree<T, N>) -> Result<&'a Node<T, N>, N> {
        tree.get_checked(self.id)
    }

    pub fn get(&self, tree: &'a ArenaTree<T, N>) -> &'a Node<T, N> {
        tree.get(self.id)
    }

    pub fn set_checked(&self, val: T, tree: &mut ArenaTree<T, N>) -> Result<(), N> {
        tree.set_checked(self.id, val)
    }

    pub fn set(&self, val: T, tree: &mut ArenaTree<T, N>) {
        tree.set(self.id, val)
    }

    pub fn add_node_checked(&self, val: T, branch: usize, tree: &mut ArenaTree<T, N>) -> Result<usize, N> {
        tree.add_node_checked(val, self.id, branch)
    }

    pub fn add_node(&self, val: T, branch: usize, tree: &mut ArenaTree<T, N>) -> usize {
        tree.add_node(val, self.id, branch)
    }

    pub fn delete_branch_checked(&self, branch: usize, tree: &mut ArenaTree<T, N>) -> Result<(), N> {
        tree.delete_branch_checked(self.id, branch)
    }

    pub fn delete_branch(&self, branch: usize, tree: &mut ArenaTree<T, N>) {
        tree.delete_branch(self.id, branch)
    }

    pub fn move_to_child_checked(&mut self, branch: usize, tree: &ArenaTree<T, N>) -> Result<NodeId, N> {
        self.id = Id(self.get_checked(tree)?
            .get_branch_checked(branch)?
            .ok_or(BranchDoesNotExist { branch, id: self.id })?
        );
        Ok(self.id)
    }

    pub fn move_to_child(&mut self, branch: usize, tree: &ArenaTree<T, N>) -> NodeId {
        self.id = Id(self.get(tree).get_branch(branch).unwrap());
        self.id
    }

    pub fn move_to_parent_checked(&mut self, tree: &ArenaTree<T, N>) -> Result<NodeId, N> {
        if self.id == Root {
            return Err(RootHasNoParent);
        }
        let id = self.get_checked(tree)?.parent_id;

        self.id = id;
        Ok(id)
    }

    pub fn move_to_parent(&mut self, tree: &ArenaTree<T, N>) -> NodeId {
        #[cfg(debug_assertions)]
        {
            self.move_to_parent_checked(tree).unwrap()
        }
        #[cfg(not(debug_assertions))]
        {
            self.id = self.get(tree).parent_id;
            self.id
        }
    }
}

macro_rules! define_cursor {
    ($(#[$attr:meta])*, $name:ident $(, $m:ident)?) => {
        $(#[$attr])*
        pub struct $name<'a, T, const N: usize> {
            inner: ArenaTreeCursor<'a, T, N>,
            tree: &'a $($m)? ArenaTree<T, N>,
        }

        impl<'a, T: Default, const N: usize> From<&'a $($m)? ArenaTree<T, N>> for $name<'a, T, N> {
            fn from(tree: &'a $($m)? ArenaTree<T, N>) -> Self {
                Self {
                    inner: Default::default(),
                    tree,
                }
            }
        }

        impl<'a, T, const N: usize> $name<'a, T, N> {
            pub fn id(&self) -> NodeId {
                self.inner.id
            }

            pub fn tree(&self) -> &ArenaTree<T, N> {
                self.tree
            }

            pub fn get(&self) -> &Node<T, N> {
                self.inner.get(self.tree)
            }

            pub fn move_to_child_checked(&mut self, branch: usize) -> Result<NodeId, N> {
                self.inner.id = Id(self.get()
                    .get_branch_checked(branch)?
                    .ok_or(BranchDoesNotExist { branch, id: self.id() })?
                );
                Ok(self.inner.id)
            }

            pub fn move_to_child(&mut self, branch: usize) -> NodeId {
                self.inner.id = Id(self.get().get_branch(branch).unwrap());
                self.inner.id
            }

            pub fn move_to_parent_checked(&mut self) -> Result<NodeId, N> {
                if self.id() == Root {
                    return Err(RootHasNoParent);
                }

                Ok(self.move_to_parent_internal())
            }

            pub fn move_to_parent(&mut self) -> NodeId {
                #[cfg(debug_assertions)]
                {
                    self.move_to_parent_checked().unwrap()
                }
                #[cfg(not(debug_assertions))]
                {
                    self.move_to_parent_internal()
                }
            }

            fn move_to_parent_internal(&mut self) -> NodeId {
                self.inner.id = self.get().parent_id;
                self.inner.id
            }
        }
    };
}

define_cursor!(#[derive(Debug, Clone, Copy, PartialEq, Eq)], BorrowedArenaTreeCursor);

define_cursor!(#[derive(Debug, PartialEq, Eq)], BorrowedArenaTreeCursorMut, mut);

impl<'a, T, const N: usize> BorrowedArenaTreeCursorMut<'a, T, N> {
    pub fn set(&mut self, val: T) {
        self.inner.set(val, self.tree)
    }

    pub fn add_node_checked(&mut self, val: T, branch: usize) -> Result<usize, N> {
        if self.get().get_branch_checked(branch)?.is_some() {
            return Err(BranchOverwrite { id: self.id(), branch });
        }

        Ok(self.add_node_internal(val, branch))
    }

    pub fn add_node(&mut self, val: T, branch: usize) -> usize {
        #[cfg(debug_assertions)]
        {
            self.add_node_checked(val, branch).unwrap()
        }
        #[cfg(not(debug_assertions))]
        {
            self.add_node_internal(val, branch)
        }
    }

    fn add_node_internal(&mut self, val: T, branch: usize) -> usize {
        self.tree.add_node(val, self.id(), branch)
    }

    pub fn delete_branch_checked(&mut self, branch: usize) -> Result<(), N> {
        match *self.get().get_branch_checked(branch)? {
            None => Err(BranchDoesNotExist { branch, id: self.id() }),
            Some(child_id) => {
                self.tree.delete_branch_internal(child_id, self.id(), branch);
                Ok(())
            },
        }
    }

    pub fn delete_branch(&mut self, branch: usize) {
        #[cfg(debug_assertions)]
        {
            self.delete_branch_checked(branch).unwrap();
        }
        #[cfg(not(debug_assertions))]
        {
            self.tree.delete_branch(self.id(), branch);
        }

    }

    pub fn as_cursor(&self) -> BorrowedArenaTreeCursor<'_, T, N> {
        BorrowedArenaTreeCursor {
            inner: self.inner,
            tree: self.tree,
        }
    }
}