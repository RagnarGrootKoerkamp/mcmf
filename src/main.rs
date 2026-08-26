use mcmf::{FlowGraph, F, FINF, W};

fn small() {
    let mut graph = FlowGraph::new(6);

    graph.add_arc(0, 1, 10, 2 as W);
    graph.add_arc(0, 2, 10, 4 as W);
    graph.add_arc(1, 2, 2, 1 as W);
    graph.add_arc(1, 3, 4, 2 as W);
    graph.add_arc(1, 4, 8, 4 as W);
    graph.add_arc(2, 4, 9, 1 as W);
    graph.add_arc(3, 5, 10, 3 as W);
    graph.add_arc(3, 4, 6, 2 as W);
    graph.add_arc(4, 5, 10, 2 as W);

    let source = 0;
    let sink = 5;

    let (max_flow, min_cost) = mcmf::MCMF::new(&mut graph, source, sink).run();
    println!("Maximum Flow: {}", max_flow);
    println!("Minimum Cost: {:.2}", min_cost);
}

fn large() {
    let n = 2000usize;
    let m = 100usize;
    let s = n + m;
    let t = n + m + 1;
    let mut graph = FlowGraph::new(n + m + 2);
    for i in 0..n {
        graph.add_arc(s, i, 1, 0 as W);
    }
    for j in n..n + m {
        graph.add_arc(j, t, n.div_ceil(m) as F, 0 as W);
    }
    // Add edges.
    for _ in 0..10000 {
        let i = rand::random_range(0..n);
        let j = rand::random_range(n..n + m);
        // let capacity = rand::random_range(0u32..20u32) as F;
        let capacity = rand::random_range(1u32..1000) as F;
        let cost = (1000000.0 / rand::random_range(0.0..1.0)) as W;
        graph.add_arc(i, j, capacity, cost);
        // eprintln!(
        //     "Edge: {:>2} -> {:>2}, cap = {:>4}, cost = {:.2}",
        //     i, j, capacity, cost
        // );
    }

    let (max_flow, min_cost) = mcmf::MCMF::new(&mut graph, s, t).run();
    println!("Maximum Flow: {}", max_flow);
    println!("Minimum Cost: {:.2}", min_cost);
}

fn main() {
    large();
}

// println!("\nEdges with non-zero flow:");

// for edge in graph.get_flow_edges() {
//     println!(
//         "  {} -> {}: flow = {}/{}, cost = {:.2}, total = {:.2}",
//         edge.from,
//         edge.to,
//         edge.flow,
//         edge.capacity,
//         edge.cost,
//         edge.flow as f64 * edge.cost
//     );
// }

// println!("\nFlow paths:");
// for (i, path) in graph.get_flow_paths(source, sink).iter().enumerate() {
//     print!("Path {}: ", i + 1);
//     for (j, &(from, to, flow, cost)) in path.iter().enumerate() {
//         if j > 0 {
//             print!(" -> ");
//         }
//         print!("{} -> {} (flow: {}, cost: {:.2})", from, to, flow, cost);
//     }
//     let total_cost: f64 = path
//         .iter()
//         .map(|&(_, _, flow, cost)| flow as f64 * cost)
//         .sum();
//     println!(" | Total cost: {:.2}", total_cost);
// }
