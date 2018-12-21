use super::algo::is_cyclic_directed;
use super::visit::{EdgeRef, IntoEdges, IntoNeighbors, IntoNodeIdentifiers, Visitable};
use std::collections::HashSet;
use std::hash::Hash;

fn find_cycle<G>(graph: G) -> Option<HashSet<G::EdgeRef>>
where
    G: IntoNodeIdentifiers + IntoNeighbors + Visitable + IntoEdges,
{
    None
}

/// Approximate Feedback Arc Set (FAS) algorithm for weighted graphs.
///
/// http://wwwusers.di.uniroma1.it/~finocchi/papers/FAS.pdf
///
pub fn fas<G>(graph: G) -> (G, HashSet<G::EdgeRef>)
where
    G: IntoNodeIdentifiers + IntoNeighbors + Visitable + IntoEdges + Clone,
    G::EdgeRef: Hash + Eq,
{
    let cloned = graph.clone();
    let mut arc_set = HashSet::new();

    while let Some(cycle) = find_cycle(cloned) {
        let _min_weight_arc = cycle.iter().map(|_| 1 /* get edge weight */).min();
        for _edge in cycle {
            // cloned.edge.weight -= _min_weight_arc;
            // if cloned.edge.weight == 0 {
            //     arc_set.insert(cloned.edge);
            //     // cloned.remove_arc(cloned.edge); ??
            // }
        }
    }

    (cloned, arc_set)
}
