use super::visit::{IntoEdges, IntoNodeIdentifiers, GraphBase, EdgeRef,
    Visitable, edge_depth_first_search, DfsEdgeEvent, Control, NodeIndexable,
    NodeCount};
use std::hash::Hash;
use std::collections::HashMap;

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

/// Returns a cycle in reverse order
pub fn find_cycle<G>(graph: G, starts: &mut Vec<G::NodeId>) -> Option<Vec<G::EdgeRef>>
where
    G: NodeCount + Visitable + IntoEdges + NodeIndexable,
    <G as GraphBase>::NodeId: Eq + Hash
{
    // let mut predecessor: Vec<Option<G::EdgeRef>> = vec![None; graph.node_bound()];
    // let ix = |i| graph.to_index(i);
    let mut predecessor: HashMap<G::NodeId, G::EdgeRef> = HashMap::new();
    // FIXME: we assume that the dfs goes thru it in order of iterator!
    let mut dropcount = 0;

    let result = edge_depth_first_search(graph, starts.iter().cloned().rev(), |event| {
        match event {
            DfsEdgeEvent::TreeEdge(e) => {
                predecessor.insert(e.target(), e);
            }
            DfsEdgeEvent::BackEdge(e) => {
                return Control::Break(e);
            }
            DfsEdgeEvent::Finish(_, _) => {
                dropcount += 1;
            }
            _ => {}
        }
        Control::Continue
    });

    for _ in 0..dropcount {
        starts.pop();
    }

    result.break_value().map(|e| {
        if e.source() == e.target() {
            return vec![e];
        }

        let mut arc_set = vec![e];
        let mut pred = predecessor[&e.source()];
        while e.target() != pred.source() {
            arc_set.push(pred);
            pred = predecessor[&e.source()];
        }
        arc_set
    })
}

// super naive implementation of fas - simply remove first edge from each cycle until
// there are no more cycles lol
pub fn naive_fas<N, E>(graph: &mut super::stable_graph::StableGraph<N, E, super::Directed>)
    -> Vec<super::graph_impl::EdgeIndex>
where
    E: Eq + Hash,
    N: Eq + Hash
{
    let mut arc_set = Vec::new();
    let mut identifiers: Vec<_> = graph.node_identifiers().collect();

    while let Some(edge_id) = find_cycle(&*graph, &mut identifiers)
                                .map(|cycle| cycle[0].id()) {
        graph.remove_edge(edge_id);
        arc_set.push(edge_id);
    }

    arc_set
}


// Approximate Feedback Arc Set (FAS) algorithm for weighted graphs.
//
// http://wwwusers.di.uniroma1.it/~finocchi/papers/FAS.pdf
// pub fn approximate_fas<G, I>(graph: G, starts: I) -> (G, HashSet<G::EdgeRef>)
// where
//     G: IntoNodeIdentifiers + IntoNeighbors + Visitable + IntoEdges + Clone,
//     G::EdgeRef: Hash + Eq + UpdateWeight,
//     <G as GraphBase>::NodeId: Eq + Hash  + Clone,
//     <G::EdgeRef as EdgeRef>::Weight: Ord + Sub<Output = <G::EdgeRef as EdgeRef>::Weight> + Clone + HasZero,
//     I: Iterator<Item=G::NodeId> + Clone
// {
//     // TODO: look into using EdgeFiltered graph here. we may not need to clone, but can just filter :-)

//     // FIXME: we could probably do without a clone of the entire graph lol
//     let mut cloned = graph.clone();
//     let mut arc_set = HashSet::new();

//     while let Some(ref mut cycle) = find_cycle(&cloned, starts.clone()) {
//         // FIXME: we should be able to work with fewer clones here
//         // FIXME: can we do a min without option result? we know we always get a value here
//         if let Some(min_weight_arc) = cycle.iter().map(EdgeRef::weight).cloned().min() {
//             for edge in cycle.iter_mut() {
//                 let old_weight = edge.weight().clone();
//                 edge.set_weight(old_weight - min_weight_arc.clone());
//                 if edge.weight() <= &<<G::EdgeRef as EdgeRef>::Weight as HasZero>::zero() {
//                     arc_set.insert(edge.clone()); // more clones lol
//                 }
//             }
//         }
//     }

//     let mut final_arc_set: HashSet<_> = arc_set.clone();

//     for edge in &arc_set {
//         final_arc_set.remove(edge);

//         if find_cycle(&cloned, starts.clone()).is_some() {
//             // adding this edge back introduces a cycle, so definitively remove it
//             final_arc_set.insert(*edge);
//         }
//     }

//     (cloned, final_arc_set)
// }
