//! Commonly used items.
//!
//! ```
//! use petgraph::prelude::*;
//! ```

#[doc(no_inline)]
pub use graph::{DiGraph, EdgeIndex, Graph, NodeIndex, UnGraph};
#[cfg(feature = "graphmap")]
#[doc(no_inline)]
pub use graphmap::{DiGraphMap, GraphMap, UnGraphMap};
#[doc(no_inline)]
#[cfg(feature = "stable_graph")]
pub use stable_graph::{StableDiGraph, StableGraph, StableUnGraph};
#[doc(no_inline)]
pub use visit::{Bfs, Dfs, DfsPostOrder};
#[doc(no_inline)]
pub use {Directed, Direction, Incoming, Outgoing, Undirected};

#[doc(no_inline)]
pub use visit::EdgeRef;
