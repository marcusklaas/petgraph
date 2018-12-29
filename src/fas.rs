use super::algo::{is_cyclic_directed, has_path_connecting};
use super::graph_impl::{IndexType, NodeIndex};
use super::stable_graph::{StableGraph, EdgeReference};
use super::visit::{
    edge_depth_first_search, Control, DfsEdgeEvent, EdgeRef, IntoEdges, IntoNodeIdentifiers,
    NodeCount, NodeIndexable, Visitable, IntoEdgeReferences,
};
use super::Directed;
use std::hash::Hash;
use std::ops::{Add, Sub};

/// Returns a cycle in reverse order
pub fn find_cycle<G, I>(graph: G, starts: I) -> Option<Vec<G::EdgeRef>>
where
    G: NodeCount + Visitable + IntoEdges + NodeIndexable,
    G::NodeId: Eq + Hash,
    I: IntoIterator<Item = G::NodeId>,
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

/// Approximate Feedback Arc Set (FAS) algorithm for weighted graphs.
///
/// http://wwwusers.di.uniroma1.it/~finocchi/papers/FAS.pdf
/// 
/// This function is *destructive* and will remove edges from the input graph.
/// In addition, it may update edge weights.
pub fn approximate_fas<N, E, Ix, F, K>(
    graph: &mut StableGraph<N, E, Directed, Ix>,
    mut edge_cost: F,
) -> Vec<(NodeIndex<Ix>, NodeIndex<Ix>, E)>
where
    Ix: IndexType,
    F: FnMut(EdgeReference<E, Ix>) -> K,
    // FIXME: can we do w/o both add and sub?
    K: Default + Copy + PartialOrd + Sub<K, Output = K> + Add<K, Output = K>,
{
    let identifiers: Vec<_> = graph.node_identifiers().collect();
    let ix = |i: NodeIndex<Ix>| i.index();
    // method that computes this is private so duplicated here
    let edge_bound = graph.edge_references()
        .next_back()
        .map_or(0, |edge| edge.id().index() + 1);
    let zero_weight = <K as Default>::default();

    let mut arc_set = Vec::new();
    let mut predecessor = vec![None; graph.node_bound()];
    let mut edge_weights = vec![zero_weight; edge_bound];
    let mut cycle = vec![];

    loop {
        // FIXME: this unsafe block shouldn't be necessary. im sure our borrow is sound
        let borrow: &StableGraph<N, E, Directed, Ix> = unsafe { ::std::mem::transmute(&*graph) };

        let lowest_cost_edge =
            edge_depth_first_search(borrow, identifiers.iter().map(|x| *x), |event| {
                match event {
                    DfsEdgeEvent::TreeEdge(e) => {
                        predecessor[ix(e.target())] = Some(e);
                    }
                    DfsEdgeEvent::BackEdge(e) => {
                        return Control::Break(e);
                    }
                    DfsEdgeEvent::Finish(..) => {
                        // FIXME: this isn't sound.
                        // somehow we should be able to do something here tho.
                        // maybe reuse (parts) of the visitor/finished maps used in the DFS?
                        //idx = ::std::cmp::min(idx + 1, identifiers.len() - 1);
                    }
                    _ => {}
                }
                Control::Continue
            })
            .break_value()
            .map(|e| {
                // TODO: double check arithmetic: should we add or subtract lol?
                let orig_edge_cost = edge_cost(e);
                let mut min_weight = orig_edge_cost - edge_weights[e.id().index()];
                let mut pred = e;

                cycle.clear();
                cycle.push((e.id(), orig_edge_cost));

                while e.target() != pred.source() {
                    pred = predecessor[ix(pred.source())].unwrap();
                    let orig_edge_cost = edge_cost(pred);
                    let edge_weight = orig_edge_cost - edge_weights[pred.id().index()];
                    cycle.push((pred.id(), orig_edge_cost));

                    if edge_weight < min_weight {
                        min_weight = edge_weight;
                    }
                }

                min_weight
            });

        if let Some(min_weight) = lowest_cost_edge {
            for &(edge_id, orig_edge_cost) in &cycle {
                let idx = edge_id.index();
                edge_weights[idx] = edge_weights[idx] + min_weight;

                if orig_edge_cost - edge_weights[idx] <= zero_weight {
                    let edge_endpoints = graph.edge_endpoints(edge_id).unwrap();
                    let w = graph.remove_edge(edge_id).unwrap();
                    arc_set.push((edge_endpoints.0, edge_endpoints.1, w));
                }
            }
        } else {
            break;
        }
    }

    let mut result_set: Vec<_> = arc_set.pop().into_iter().collect();

    // try to re-add edges without introducing cycles. skip last one, since that
    // will always introduce a cycle
    for (start, end, w) in arc_set {
        let edge_id = graph.add_edge(start, end, w);

        if is_cyclic_directed(&*graph) {
            let w = graph.remove_edge(edge_id).unwrap();
            result_set.push((start, end, w));
        }
    }

    // // instead of adding edge, checking for cycle, removing, we could
    // // check whether there is a path from end to start in graph.
    // // but for some reason this is slower than the above block...
    // for (start, end, w) in arc_set {
    //     if has_path_connecting(&*graph, end, start, None) {
    //         result_set.push((start, end, w));
    //     } else {
    //         graph.add_edge(start, end, w);
    //     }
    // }

    debug_assert!(!is_cyclic_directed(&*graph));

    result_set
}
