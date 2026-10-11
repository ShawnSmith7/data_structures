use super::{NodeId, ArenaTree, Node, ArenaTreeError};
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

    pub fn get_checked(&self, tree: &'a ArenaTree<T, N>) -> Result<&'a Node<T, N>, ArenaTreeError<N>> {
        tree.get_checked(self.id)
    }

    pub fn get(&self, tree: &'a ArenaTree<T, N>) -> &'a Node<T, N> {
        tree.get(self.id)
    }

    pub unsafe fn get_unchecked(&self, tree: &'a ArenaTree<T, N>) -> &'a Node<T, N> {
        unsafe {
            tree.get_unchecked(self.id)
        }
    }

    pub fn set_checked(&self, val: T, tree: &mut ArenaTree<T, N>) -> Result<(), ArenaTreeError<N>> {
        tree.set_checked(self.id, val)
    }

    pub fn set(&self, val: T, tree: &mut ArenaTree<T, N>) {
        tree.set(self.id, val)
    }

    pub unsafe fn set_unchecked(&self, val: T, tree: &mut ArenaTree<T, N>) {
        unsafe {
            tree.set_unchecked(self.id, val)
        }
    }

    pub fn add_node_checked(&self, val: T, branch: usize, tree: &mut ArenaTree<T, N>) -> Result<usize, ArenaTreeError<N>> {
        tree.add_node_checked(val, self.id, branch)
    }

    pub fn add_node(&self, val: T, branch: usize, tree: &mut ArenaTree<T, N>) -> usize {
        tree.add_node(val, self.id, branch)
    }

    pub unsafe fn add_node_unchecked(&self, val: T, branch: usize, tree: &mut ArenaTree<T, N>) -> usize {
        unsafe {
            tree.add_node_unchecked(val, self.id, branch)
        }
    }

    pub fn delete_branch_checked(&self, branch: usize, tree: &mut ArenaTree<T, N>) -> Result<(), ArenaTreeError<N>> {
        tree.delete_branch_checked(self.id, branch)
    }

    pub fn delete_branch(&self, branch: usize, tree: &mut ArenaTree<T, N>) {
        tree.delete_branch(self.id, branch)
    }

    pub unsafe fn delete_branch_unchecked(&self, branch: usize, tree: &mut ArenaTree<T, N>) {
        unsafe {
            tree.delete_branch_unchecked(self.id, branch)
        }
    }

    pub fn move_to_child_checked(&mut self, branch: usize, tree: &ArenaTree<T, N>) -> Result<NodeId, ArenaTreeError<N>> {
        let id = Id(self.get_checked(tree)?
            .get_branch(branch)?
            .ok_or(BranchDoesNotExist { branch, id: self.id })?
        );

        self.id = id;
        Ok(id)
    }

    pub fn move_to_child(&mut self, branch: usize, tree: &ArenaTree<T, N>) -> NodeId {
        self.move_to_child_checked(branch, tree).expect("Invalid state passed to ArenaTreeCursor::move_to_child")
    }

    pub unsafe fn move_to_child_unchecked(&mut self, branch: usize, tree: &ArenaTree<T, N>) -> NodeId {
        let id = unsafe {
            Id(self.get_unchecked(tree).branches.get_unchecked(branch).unwrap())
        };

        self.id = id;
        id
    }

    pub fn move_to_parent_checked(&mut self, tree: &ArenaTree<T, N>) -> Result<NodeId, ArenaTreeError<N>> {
        if self.id == Root {
            return Err(RootHasNoParent);
        }
        let id = self.get_checked(tree)?.parent_id;

        self.id = id;
        Ok(id)
    }

    pub fn move_to_parent(&mut self, tree: &ArenaTree<T, N>) -> NodeId {
        self.move_to_parent_checked(tree).expect("Invalid state passed to ArenaTreeCursor::move_to_parent")
    }

    pub unsafe fn move_to_parent_unchecked(&mut self, tree: &ArenaTree<T, N>) -> NodeId {
        let id = unsafe {
            self.get_unchecked(tree).parent_id
        };

        self.id = id;
        id
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
                unsafe {
                    self.inner.get_unchecked(self.tree)
                }
            }

            pub fn move_to_child_checked(&mut self, branch: usize) -> Result<NodeId, ArenaTreeError<N>> {
                let id = Id(self.get()
                    .get_branch(branch)?
                    .ok_or(BranchDoesNotExist { branch, id: self.id() })?
                );

                self.inner.id = id;
                Ok(id)
            }

            pub fn move_to_child(&mut self, branch: usize) -> NodeId {
                self.move_to_child_checked(branch).expect(concat!("Invalid state passed to ", stringify!($name),"::move_to_child"))
            }

            pub unsafe fn move_to_child_unchecked(&mut self, branch: usize) -> NodeId {
                let id = unsafe {
                    Id(self.get().branches.get_unchecked(branch).unwrap())
                };

                self.inner.id = id;
                id
            }

            pub fn move_to_parent_checked(&mut self) -> Result<NodeId, ArenaTreeError<N>> {
                if self.id() == Root {
                    return Err(RootHasNoParent);
                }

                unsafe {
                    Ok(self.move_to_parent_unchecked())
                }
            }

            pub fn move_to_parent(&mut self) -> NodeId {
                self.move_to_parent_checked().expect(concat!("Invalid state passed to ", stringify!($name),"::move_to_parent"))
            }

            pub unsafe fn move_to_parent_unchecked(&mut self) -> NodeId {
                let id = self.get().parent_id;

                self.inner.id = id;
                id
            }
        }
    };
}

define_cursor!(#[derive(Debug, Clone, Copy, PartialEq, Eq)], BorrowedArenaTreeCursor);

define_cursor!(#[derive(Debug, PartialEq, Eq)], BorrowedArenaTreeCursorMut, mut);

impl<'a, T, const N: usize> BorrowedArenaTreeCursorMut<'a, T, N> {
    pub fn set(&mut self, val: T) {
        unsafe {
            self.inner.set_unchecked(val, self.tree)
        }
    }

    pub fn add_node_checked(&mut self, val: T, branch: usize) -> Result<usize, ArenaTreeError<N>> {
        if self.get().get_branch(branch)?.is_some() {
            return Err(BranchOverwrite { id: self.id(), branch });
        }

        unsafe {
            Ok(self.add_node_unchecked(val, branch))
        }
    }

    pub fn add_node(&mut self, val: T, branch: usize) -> usize {
        self.add_node_checked(val, branch).expect("Invalid usize passed to BorrowedArenaTreeCursorMut::add_node")
    }

    pub unsafe fn add_node_unchecked(&mut self, val: T, branch: usize) -> usize {
        unsafe {
            self.tree.add_node_unchecked(val, self.id(), branch)
        }
    }

    pub fn delete_branch_checked(&mut self, branch: usize) -> Result<(), ArenaTreeError<N>> {
        match *self.get().get_branch(branch)? {
            None => Err(BranchDoesNotExist { branch, id: self.id() }),
            Some(child_id) => unsafe {
                self.tree.delete_branch_inner(child_id, self.id(), branch);
                Ok(())
            },
        }
    }

    pub fn delete_branch(&mut self, branch: usize) {
        self.delete_branch_checked(branch).expect("Invalid usize passed for BorrowedArenaTreeCursorMut::delete_branch");
    }

    pub unsafe fn delete_branch_unchecked(&mut self, branch: usize) {
        unsafe {
            self.tree.delete_branch_unchecked(self.id(), branch);
        }
    }

    pub fn as_cursor(&self) -> BorrowedArenaTreeCursor<'_, T, N> {
        BorrowedArenaTreeCursor {
            inner: self.inner,
            tree: self.tree,
        }
    }
}