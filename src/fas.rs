use super::algo::is_cyclic_directed;
use super::graph_impl::{IndexType, NodeIndex};
use super::stable_graph::{StableGraph, EdgeReference};
use super::visit::{
    EdgeRef, IntoNodeIdentifiers,
    NodeIndexable, Visitable, IntoEdgeReferences, VisitMap,
};
use super::Directed;
use std::ops::{Add, Sub};
use std::cmp::Ordering;

/// Returns a cycle in reverse order
fn find_cycle<'g, N, E, Ix>(
    graph: &'g StableGraph<N, E, Directed, Ix>,
    predecessor: &mut Vec<Option<EdgeReference<'g, E, Ix>>>,
    discovered: &mut <StableGraph<N, E, Directed, Ix> as Visitable>::Map,
    finished: &mut <StableGraph<N, E, Directed, Ix> as Visitable>::Map,
    start: NodeIndex<Ix>
) -> Option<EdgeReference<'g, E, Ix>>
where Ix: IndexType
{
    if !discovered.visit(start) {
        return None;
    }

    for e in graph.edges(start) {
        let v = e.target();

        if !discovered.is_visited(&v) {
            predecessor[v.index()] = Some(e);
            
            if let Some(e2) = find_cycle(graph, predecessor, discovered, finished, v) {
                return Some(e2);
            }
        } else if !finished.is_visited(&v) {
            return Some(e);
        }
    }

    finished.visit(start);

    None
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
    K: Default + Copy + PartialOrd + Sub<K, Output = K> + Add<K, Output = K>,
{
    // method that computes this is private so duplicated here
    let edge_bound = graph.edge_references()
        .next_back()
        .map_or(0, |edge| edge.id().index() + 1);
    let zero_weight = <K as Default>::default();

    let mut arc_set = Vec::new();
    let mut predecessor = vec![None; graph.node_bound()];
    let mut edge_weights = vec![zero_weight; edge_bound];
    let mut cycle = Vec::new();

    // DFS stuff
    let mut discovered = graph.visit_map();
    let mut finished = graph.visit_map();

    // keep cycles until there are none left by removing one of their edges
    loop {
        // FIXME: this unsafe block shouldn't be necessary. im sure our borrow is sound
        let borrow: &StableGraph<N, E, Directed, Ix> = unsafe { ::std::mem::transmute(&*graph) };
        discovered.as_mut_slice().copy_from_slice(finished.as_slice());

        let lowest_cost =
            borrow
            .node_identifiers()
            .filter_map(|start| find_cycle(borrow, &mut predecessor, &mut discovered, &mut finished, start))
            .next()
            .map(|e| {
                let orig_edge_cost = edge_cost(e);
                let mut min_weight = orig_edge_cost - edge_weights[e.id().index()];
                let mut pred = e;

                cycle.clear();
                cycle.push((e.id(), orig_edge_cost));

                while e.target() != pred.source() {
                    pred = predecessor[pred.source().index()].unwrap();
                    let orig_edge_cost = edge_cost(pred);
                    let edge_weight = orig_edge_cost - edge_weights[pred.id().index()];
                    cycle.push((pred.id(), orig_edge_cost));

                    if edge_weight < min_weight {
                        min_weight = edge_weight;
                    }
                }

                min_weight
            });

        if let Some(min_weight) = lowest_cost {
            let mut removed = false;

            // update the weights of all arcs in the cycle and remove the
            // first one that hits zero
            for &(edge_id, orig_edge_cost) in &cycle {
                let idx = edge_id.index();
                edge_weights[idx] = edge_weights[idx] + min_weight;

                if removed || orig_edge_cost - edge_weights[idx] <= zero_weight {
                    let edge_endpoints = graph.edge_endpoints(edge_id).unwrap();
                    let w = graph.remove_edge(edge_id).unwrap();
                    arc_set.push((edge_endpoints.0, edge_endpoints.1, w, orig_edge_cost));
                    removed = true;
                }
            }
        } else {
            break;
        }
    }

    let arc_set_len = arc_set.len();
    let mut result_set = Vec::with_capacity(arc_set_len);
    
    // always include the last edge, since re-adding that
    // will always introduce a cycle
    if let Some((start, end, w, _edge_cost)) = arc_set.pop() {
        result_set.push((start, end, w));
    }

    // sorting arc_set by cost should improve final solution
    arc_set.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(Ordering::Equal));

    // try to re-add edges without introducing cycles
    for (start, end, w, _edge_cost) in arc_set {
        // FIXME: wtf?!
        let borrow: &StableGraph<N, E, Directed, Ix> = unsafe { ::std::mem::transmute(&*graph) };
        let edge_id = graph.add_edge(start, end, w);

        discovered.clear();
        finished.clear();

        if find_cycle(borrow, &mut predecessor, &mut discovered, &mut finished, start).is_some() {
            let w = graph.remove_edge(edge_id).unwrap();
            result_set.push((start, end, w));
        }
    }

    debug_assert!(!is_cyclic_directed(&*graph));

    result_set
}
