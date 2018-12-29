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
    -> Vec<(NodeIndex<Ix>, NodeIndex<Ix>, E)>
where
    E: Eq + Hash + Default + Copy + PartialOrd, // + Sub<E, Output = E>,
    N: Eq + Hash,
    Ix: IndexType + 'static + Copy,
{
    let mut arc_set = Vec::new();
    let identifiers: Vec<_> = graph.node_identifiers().collect();
    let mut predecessor = vec![None; graph.node_bound()];
    let ix = |i: NodeIndex<Ix>| i.index();
    let mut cycle = vec![];
    let mut idx = 0;

    loop {
        // FIXME: this unsafe block shouldn't be necessary. im sure our borrow is sound
        let borrow: &super::stable_graph::StableGraph<N, E, super::Directed, Ix> =
            unsafe { ::std::mem::transmute(&*graph) };

        if edge_depth_first_search(borrow, identifiers[idx..].iter().map(|x| *x), |event| {
            match event {
                DfsEdgeEvent::TreeEdge(e) => {
                    predecessor[ix(e.target())] = Some(e);
                }
                DfsEdgeEvent::BackEdge(e) => {
                    return Control::Break(e);
                }
                DfsEdgeEvent::Finish(..) => {
                    // FIXME: this isn't sound somehow. it makes sense.
                    // somehow we should be able to do something here tho.
                    // maybe reuse (parts) of the visitor/finished maps used in the DFS?
                    //idx = ::std::cmp::min(idx + 1, identifiers.len() - 1);
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
            break;
        }

        let edge_id = cycle[0];
        let edge_endpoints = graph.edge_endpoints(edge_id).unwrap();
        let w = graph.remove_edge(edge_id).unwrap();
        arc_set.push((edge_endpoints.0, edge_endpoints.1, w));
    }

    let mut result_set: Vec<_> = arc_set.pop().into_iter().collect();
    
    // try to re-add adges. skip last one. this will always introduce a cycle
    // TODO: instead of adding, checking for cycle, removing, we should be 
    // able to simply check whether there is a path from end to start in graph!
    for (start, end, w) in arc_set {
        let edge_id = graph.add_edge(start, end, w);

        if super::algo::is_cyclic_directed(&*graph) {
            let _ = graph.remove_edge(edge_id);
            result_set.push((start, end, w));
        }
    }

    assert!(!super::algo::is_cyclic_directed(&*graph));

    result_set
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
