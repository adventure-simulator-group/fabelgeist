//! Hierarchical level of detail.
//!
//! A scene of many objects -- a forest of thousands of plants -- costs a
//! renderer in proportion to its objects, however few pixels the far ones
//! cover. A hierarchy of clusters breaks that: objects are grouped into a
//! bounding-volume hierarchy ([`Tree`]), every internal node carries a
//! *proxy* standing in for its whole subtree at less detail, and a frame
//! draws the cut through the tree where each node's detail just suffices:
//! the leaves up close, a node's proxy for a whole grove far away.
//!
//! What a proxy is belongs to the caller -- the instances of a plant's
//! thickest branches and merged leaves, a decimated mesh, an impostor. This
//! crate needs only, per node, where its proxy's instances lie in each of
//! the caller's instance streams (a range per stream) and its **error**: the
//! size, in world units, of the largest detail the proxy leaves out. Leaves
//! -- objects at full detail -- have error zero.
//!
//! # The cut, in parallel
//!
//! A node is drawn when its own error, projected to the screen, is within a
//! tolerance and its parent's is not ([`Hierarchy`] keeps both for every
//! node). Errors only grow up the tree and every parent's sphere holds its
//! children's, so projected errors only grow going up too -- from any point
//! of view -- and exactly one node on every path from the root to a leaf
//! passes: the test needs no traversal, and every node takes it at once
//! (as Nanite chooses its clusters). A node outside the view is culled with
//! everything below it by the same one test.
//!
//! A drawn node's ranges are appended to draw lists -- one per stream and
//! per level of detail, the level chosen from the node's size on screen --
//! whose lengths are the instance counts of indirect draws
//! ([`gpu::GpuHierarchy`]). [`Hierarchy::select`] is the same on the host.

#[cfg(feature = "gpu")]
pub mod gpu;
pub mod hierarchy;
pub mod tree;

pub use hierarchy::{Cluster, Hierarchy, Node, View};
pub use tree::Tree;
