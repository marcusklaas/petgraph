//use super::algo::is_cyclic_directed;
use super::visit::{EdgeRef, IntoEdges, IntoNeighbors, IntoNodeIdentifiers, Visitable};
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Sub;

pub trait UpdateWeight: EdgeRef {
    fn set_weight(&mut self, new_weight: <Self as EdgeRef>::Weight);
}

// std::num::Zero is unstable
// what to do here?
// we could pass in a zero value explicitly
// or we could compare to x - x with the assumption that it's 0
pub trait HasZero {
    fn zero() -> Self;
}

fn find_cycle<G>(graph: G, removed_edges: &HashSet<G::EdgeRef>) -> Option<Vec<G::EdgeRef>>
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
    G::EdgeRef: Hash + Eq + UpdateWeight,
    <G::EdgeRef as EdgeRef>::Weight: Ord + Sub<Output = <G::EdgeRef as EdgeRef>::Weight> + Clone + HasZero
{
    // FIXME: we could probably do without a clone of the entire graph lol
    let mut cloned = graph.clone();
    let mut arc_set = HashSet::new();

    while let Some(ref mut cycle) = find_cycle(cloned, &arc_set) {
        // FIXME: we should be able to work with fewer clones here
        // FIXME: can we do a min without option result? we know we always get a value here
        if let Some(min_weight_arc) = cycle.iter().map(EdgeRef::weight).cloned().min() {
            for edge in cycle.iter_mut() {
                let old_weight = edge.weight().clone();
                edge.set_weight(old_weight - min_weight_arc.clone());
                if edge.weight() <= &<<G::EdgeRef as EdgeRef>::Weight as HasZero>::zero() {
                    arc_set.insert(edge.clone()); // more clones lol
                }
            }
        }
    }

    let mut final_arc_set: HashSet<_> = arc_set.clone();

    for edge in &arc_set {
        final_arc_set.remove(edge);

        if find_cycle(cloned, &final_arc_set).is_some() {
            // adding this edge back introduces a cycle, so definitively remove it
            final_arc_set.insert(*edge);
        }
    }

    (cloned, final_arc_set)
}
