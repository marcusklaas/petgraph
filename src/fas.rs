use super::visit::{IntoEdges, IntoNodeIdentifiers, EdgeRef,
    Visitable, edge_depth_first_search, DfsEdgeEvent, Control, NodeIndexable,
    NodeCount};
use super::graph_impl::{NodeIndex, IndexType};
use std::hash::Hash;

pub trait UpdateWeight: EdgeRef {
    fn set_weight(&mut self, new_weight: <Self as EdgeRef>::Weight);
}

/// Returns a cycle in reverse order
pub fn find_cycle<G, I>(graph: G, starts: I) -> Option<Vec<G::EdgeRef>>
where
    G: NodeCount + Visitable + IntoEdges + NodeIndexable,
    G::NodeId: Eq + Hash,
    I: IntoIterator<Item=G::NodeId>
{
    let mut predecessor: Vec<Option<G::EdgeRef>> = vec![None; graph.node_bound()];
    let ix = |i| graph.to_index(i);

    let result = edge_depth_first_search(graph, starts, |event| {
        match event {
            DfsEdgeEvent::TreeEdge(e) => {
                predecessor[ix(e.target())] = Some(e);
            }
            DfsEdgeEvent::BackEdge(e) => {
                return Control::Break(e);
            }
            _ => {}
        }
        Control::Continue
    });

    result.break_value().map(|e| {
        let mut arc_set = vec![e];
        let mut pred = e;
        while e.target() != pred.source() {
            arc_set.push(pred);
            pred = predecessor[ix(pred.source())].unwrap();
        }
        arc_set
    })
}

use std::ops::Sub;

// super naive implementation of fas - simply remove first edge from each cycle until
// there are no more cycles lol
pub fn naive_fas<N, E, Ix>(graph: &mut super::stable_graph::StableGraph<N, E, super::Directed, Ix>)
    -> Vec<super::graph_impl::EdgeIndex<Ix>>
where
    E: Eq + Hash + Default + Copy + PartialOrd, // + Sub<E, Output = E>,
    N: Eq + Hash,
    Ix: IndexType + 'static + Copy,
{
    let mut arc_set: Vec<super::graph_impl::EdgeIndex<Ix>> = Vec::new();
    let identifiers: Vec<_> = graph.node_identifiers().collect();
    let mut predecessor: Vec<Option<_>> = vec![None; graph.node_bound()];
    let ix = |i: NodeIndex<Ix>| i.index();
    let mut cycle = vec![];

    loop {
        {
            // FIXME: this unsafe block shouldn't be necessary. im sure our borrow is sound
            let borrow: &super::stable_graph::StableGraph<N, E, super::Directed, Ix> =
                unsafe { ::std::mem::transmute(&*graph) };

            if edge_depth_first_search(borrow, identifiers.iter().map(|x| *x), |event| {
                match event {
                    DfsEdgeEvent::TreeEdge(e) => {
                        predecessor[ix(e.target())] = Some(e);
                    }
                    DfsEdgeEvent::BackEdge(e) => {
                        return Control::Break(e);
                    }
                    _ => {}
                }
                Control::Continue
            })
            .break_value().map(|e| {
                //let mut min_weight: E = <_>::default();
                cycle.clear();
                cycle.push(e.id());
                let mut pred = e;
                while e.target() != pred.source() {
                    cycle.push(pred.id());
                    pred = predecessor[ix(pred.source())].unwrap();
                    //min_weight = if min_weight < *e.weight() { min_weight } else { *e.weight() };
                }
            }).is_none() {
                return arc_set;
            }
        };

        {
            let edge_id = cycle[0];
            graph.remove_edge(edge_id);
            arc_set.push(edge_id);
        }
    }
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
