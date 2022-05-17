use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::unwrap;

use super::{binary_index::*, BinaryDecisionDiagram, NodePtrMut};

#[derive(Debug, Clone, Copy)]
pub struct NodeHandler<T>(pub(super) super::Link<T>)
where
    T: Clone;

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Element<T> {
    Variable(T),
    Binary(bool),
}

impl<T> NodeHandler<T>
where
    T: Clone,
{
    pub fn get_child(&self, child_index: BinaryIndex) -> Option<NodeHandler<T>> {
        match &self.0 {
            super::Link::Node(node) => unsafe {
                Some(match child_index {
                    BinaryIndex::Left => NodeHandler((*(*node)).links.0.clone()),
                    BinaryIndex::Right => NodeHandler((*(*node)).links.1.clone()),
                })
            },
            super::Link::Leaf(_) => None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        return matches!(self, Self(super::Link::Leaf(_)));
    }

    pub(super) fn get_parents<'a>(
        &self,
        diagram: &'a BinaryDecisionDiagram<T>,
    ) -> &'a std::collections::HashSet<NodePtrMut<T>> {
        match self.0 {
            super::Link::Node(node) => &unsafe { &mut *node }.parents,
            super::Link::Leaf(value) => match value {
                true => &diagram.leaf_parents.1,
                false => &diagram.leaf_parents.0,
            },
        }
    }

    pub fn get_element(&self) -> Element<&T> {
        match &self.0 {
            super::Link::Node(node) => Element::Variable(unsafe { &(*(*node)).variable }),
            super::Link::Leaf(value) => Element::Binary(*value),
        }
    }
}

// For Display
// returns (index, flag)
// `flag` is true iff the index is generated by this function call
impl<T> NodeHandler<T>
where
    T: Clone + Display,
{
    fn get_index(
        node_handler: &NodeHandler<T>,
        f: &mut std::fmt::Formatter<'_>,
        index: &mut u32,
        visit_record: &mut HashMap<NodePtrMut<T>, u32>,
    ) -> (u32, bool) {
        match node_handler.0 {
            super::Link::Node(node) => {
                if visit_record.contains_key(&node) {
                    return (visit_record[&node], false);
                } else {
                    writeln!(
                        f,
                        "{index} [label=\"{var}\"]",
                        var = unwrap!(node_handler.get_element(), Element::Variable(var), var)
                    )
                    .unwrap();
                    let node_index = index.clone();
                    visit_record.insert(node, node_index);
                    *index += 1;
                    return (node_index, true);
                }
            }
            super::Link::Leaf(value) => return (value as u32, false),
        }
    }

    fn fmt_reclusive(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        index: &mut u32,
        visit_record: &mut HashMap<NodePtrMut<T>, u32>,
    ) -> std::fmt::Result {
        match self.get_element() {
            Element::Variable(var) => {
                let (parent_index, _) = Self::get_index(self, f, index, visit_record);
                let ((left_index, left_recursive_flag), (right_index, right_recursive_flag)) = (
                    Self::get_index(
                        &mut self.get_child(BinaryIndex::Left).unwrap(),
                        f,
                        index,
                        visit_record,
                    ),
                    Self::get_index(
                        &mut self.get_child(BinaryIndex::Right).unwrap(),
                        f,
                        index,
                        visit_record,
                    ),
                );
                writeln!(f, "{parent_index} -> {left_index} [label=\"0\"]")?;
                writeln!(f, "{parent_index} -> {right_index} [label=\"1\"]")?;
                if left_recursive_flag {
                    self.get_child(BinaryIndex::Left).unwrap().fmt_reclusive(
                        f,
                        index,
                        visit_record,
                    )?;
                }
                if right_recursive_flag {
                    self.get_child(BinaryIndex::Right).unwrap().fmt_reclusive(
                        f,
                        index,
                        visit_record,
                    )?;
                }
                Ok(())
            }
            Element::Binary(value) => Ok(()),
        }
    }
}

impl<T> Display for NodeHandler<T>
where
    T: Clone + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "digraph{{")?;
        writeln!(f, r#"0 [label="false"]"#)?;
        writeln!(f, r#"1 [label="true"]"#)?;
        let mut index = 2;
        self.fmt_reclusive(f, &mut index, &mut HashMap::new())?;
        writeln!(f, "}}")?;
        Ok(())
    }
}