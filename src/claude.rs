use std::collections::{HashMap, VecDeque};
#[derive(Debug, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub capacity: i64,
    pub cost: f64,
    pub flow: i64,
}

#[derive(Debug)]
pub struct MinCostMaxFlow {
    n: usize,
    edges: Vec<Edge>,
    graph: Vec<Vec<usize>>, // adjacency list storing edge indices
}

impl MinCostMaxFlow {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            edges: Vec::new(),
            graph: vec![Vec::new(); n],
        }
    }

    pub fn add_edge(&mut self, from: usize, to: usize, capacity: i64, cost: f64) {
        let edge_idx = self.edges.len();

        // Forward edge
        self.edges.push(Edge {
            from,
            to,
            capacity,
            cost,
            flow: 0,
        });
        self.graph[from].push(edge_idx);

        // Backward edge (reverse)
        self.edges.push(Edge {
            from: to,
            to: from,
            capacity: 0,
            cost: -cost,
            flow: 0,
        });
        self.graph[to].push(edge_idx + 1);
    }

    /// Find minimum cost maximum flow from source to sink
    pub fn min_cost_max_flow(&mut self, source: usize, sink: usize) -> (i64, f64) {
        let mut total_flow = 0i64;
        let mut total_cost = 0f64;
        let mut iteration = 0;

        loop {
            iteration += 1;
            // Use SPFA (Shortest Path Faster Algorithm) to find shortest path
            let (dist, parent) = self.spfa(source, sink);

            if dist[sink] == f64::INFINITY {
                eprintln!(
                    "Iteration {}: No path found (dist[sink] = infinity)",
                    iteration
                );
                break; // No more augmenting paths
            }
            eprintln!(
                "Iteration {}: Found path with cost {}",
                iteration, dist[sink]
            );

            // Find minimum residual capacity along the path
            // Also check for cycles to prevent infinite loops
            let mut path_flow = i64::MAX;
            let mut node = sink;
            let mut visited = vec![false; self.n];
            let mut valid_path = true;
            let mut path_edges = Vec::new();

            while node != source {
                if visited[node] {
                    // Cycle detected - invalid path
                    valid_path = false;
                    eprintln!("  Cycle detected at node {}", node);
                    break;
                }
                visited[node] = true;

                if let Some(edge_idx) = parent[node] {
                    let edge = &self.edges[edge_idx];
                    let residual = edge.capacity - edge.flow;
                    path_edges.push((edge.from, edge.to, residual, edge_idx));
                    path_flow = path_flow.min(residual);
                    node = edge.from;
                } else {
                    // No parent - path doesn't reach source
                    valid_path = false;
                    eprintln!("  No parent for node {}", node);
                    break;
                }
            }

            if !valid_path || path_flow == 0 {
                if path_flow == 0 {
                    eprintln!("  Path found but bottleneck capacity is 0");
                    for (from, to, residual, idx) in path_edges.iter().rev() {
                        eprintln!(
                            "    {} -> {} (edge {}, residual: {})",
                            from, to, idx, residual
                        );
                    }
                }
                if !valid_path {
                    eprintln!("  Attempting to trace parent path from sink {}:", sink);
                    let mut trace_node = sink;
                    let mut trace_count = 0;
                    while trace_count < self.n * 2 {
                        if let Some(edge_idx) = parent[trace_node] {
                            let edge = &self.edges[edge_idx];
                            eprintln!("    {} <- {} (edge {})", trace_node, edge.from, edge_idx);
                            trace_node = edge.from;
                            trace_count += 1;
                            if trace_node == source {
                                eprintln!("    Reached source!");
                                break;
                            }
                        } else {
                            eprintln!("    {} has no parent", trace_node);
                            break;
                        }
                    }
                }
                break; // No valid augmenting path
            }

            eprintln!("  Sending {} units of flow", path_flow);

            // Update flow along the path
            node = sink;
            while node != source {
                let edge_idx = parent[node].unwrap();
                self.edges[edge_idx].flow += path_flow;
                // Update reverse edge
                self.edges[edge_idx ^ 1].flow -= path_flow;
                total_cost += path_flow as f64 * self.edges[edge_idx].cost;
                node = self.edges[edge_idx].from;
            }

            total_flow += path_flow;
        }

        (total_flow, total_cost)
    }

    /// SPFA algorithm for finding shortest path with negative edge costs
    /// Includes cycle detection to avoid infinite loops
    fn spfa(&self, source: usize, sink: usize) -> (Vec<f64>, Vec<Option<usize>>) {
        let mut dist = vec![f64::INFINITY; self.n];
        let mut parent = vec![None; self.n];
        let mut in_queue = vec![false; self.n];
        let mut cnt = vec![0; self.n]; // Count of times each node is added to queue
        let mut queue = VecDeque::new();

        dist[source] = 0.0;
        queue.push_back(source);
        in_queue[source] = true;
        cnt[source] = 1;

        while let Some(u) = queue.pop_front() {
            in_queue[u] = false;

            for &edge_idx in &self.graph[u] {
                let edge = &self.edges[edge_idx];

                // Check if edge has residual capacity
                if edge.flow < edge.capacity {
                    let v = edge.to;
                    let new_dist = dist[u] + edge.cost;

                    // Use a small epsilon for floating point comparison
                    if new_dist < dist[v] - 1e-9 {
                        dist[v] = new_dist;

                        // Check if setting parent[v] = u would create a cycle
                        // by checking if u is reachable from v through current parents
                        let mut creates_cycle = false;
                        let mut check_node = u;
                        let mut visited_check = vec![false; self.n];
                        while check_node != source {
                            if visited_check[check_node] || check_node == v {
                                creates_cycle = true;
                                break;
                            }
                            visited_check[check_node] = true;

                            if let Some(p_edge_idx) = parent[check_node] {
                                let edge1: &Edge = &self.edges[p_edge_idx];
                                check_node = edge1.from;
                            } else {
                                break;
                            }
                        }

                        if creates_cycle {
                            // Skip this edge as it would create a cycle in the parent tree
                            continue;
                        }

                        parent[v] = Some(edge_idx);

                        if !in_queue[v] {
                            queue.push_back(v);
                            in_queue[v] = true;
                            cnt[v] += 1;

                            // If a node is added to queue more than n times, there's a negative cycle
                            if cnt[v] > self.n {
                                return (vec![f64::INFINITY; self.n], vec![None; self.n]);
                            }
                        }
                    }
                }
            }
        }

        (dist, parent)
    }

    /// Get all edges with non-zero flow
    pub fn get_flow_edges(&self) -> Vec<Edge> {
        self.edges
            .iter()
            .step_by(2) // Only forward edges (skip reverse edges)
            .filter(|e| e.flow > 0)
            .cloned()
            .collect()
    }

    /// Get all edges (including those with zero flow)
    pub fn get_all_edges(&self) -> Vec<Edge> {
        self.edges
            .iter()
            .step_by(2) // Only forward edges
            .cloned()
            .collect()
    }

    /// Get the flow paths as a list of sequences
    pub fn get_flow_paths(&self, source: usize, sink: usize) -> Vec<Vec<(usize, usize, i64, f64)>> {
        let mut paths = Vec::new();
        let mut remaining_flow: HashMap<(usize, usize), i64> = HashMap::new();

        // Build a map of remaining flows
        for edge in self.edges.iter().step_by(2) {
            if edge.flow > 0 {
                remaining_flow.insert((edge.from, edge.to), edge.flow);
            }
        }

        // Extract paths using DFS
        while let Some(&start_edge) = remaining_flow.keys().find(|&&(from, _)| from == source) {
            let mut path = Vec::new();
            let mut current = start_edge.0;
            let mut min_flow = i64::MAX;

            // Build path from source to sink
            let mut visited = vec![false; self.n];
            while current != sink {
                if visited[current] {
                    break; // Cycle detected, break out
                }
                visited[current] = true;

                if let Some(((from, to), flow)) = remaining_flow
                    .iter()
                    .find(|((f, _), _)| *f == current)
                    .map(|(k, v)| (*k, *v))
                {
                    // Find the edge to get its cost
                    let edge = self
                        .edges
                        .iter()
                        .step_by(2)
                        .find(|e| e.from == from && e.to == to)
                        .unwrap();

                    path.push((from, to, flow, edge.cost));
                    min_flow = min_flow.min(flow);
                    current = to;
                } else {
                    break;
                }
            }

            if current == sink && !path.is_empty() {
                // Subtract the minimum flow from all edges in the path
                for &(from, to, _, _) in &path {
                    let flow = remaining_flow.get_mut(&(from, to)).unwrap();
                    *flow -= min_flow;
                    if *flow == 0 {
                        remaining_flow.remove(&(from, to));
                    }
                }

                // Store the path with the actual flow value
                let path_with_flow: Vec<_> = path
                    .iter()
                    .map(|&(from, to, _, cost)| (from, to, min_flow, cost))
                    .collect();
                paths.push(path_with_flow);
            } else {
                break; // Can't form a complete path
            }
        }

        paths
    }
}

// Example usage
fn main_old() {
    let mut graph = MinCostMaxFlow::new(6);

    // Add edges: from, to, capacity, cost
    graph.add_edge(0, 1, 10, 2.0);
    graph.add_edge(0, 2, 10, 4.0);
    graph.add_edge(1, 2, 2, 1.0);
    graph.add_edge(1, 3, 4, 2.0);
    graph.add_edge(1, 4, 8, 4.0);
    graph.add_edge(2, 4, 9, 1.0);
    graph.add_edge(3, 5, 10, 3.0);
    graph.add_edge(4, 3, 6, 2.0);
    graph.add_edge(4, 5, 10, 2.0);

    let source = 0;
    let sink = 5;

    let (max_flow, min_cost) = graph.min_cost_max_flow(source, sink);

    println!("Maximum Flow: {}", max_flow);
    println!("Minimum Cost: {:.2}", min_cost);
    println!("\nEdges with non-zero flow:");

    for edge in graph.get_flow_edges() {
        println!(
            "  {} -> {}: flow = {}/{}, cost = {:.2}, total = {:.2}",
            edge.from,
            edge.to,
            edge.flow,
            edge.capacity,
            edge.cost,
            edge.flow as f64 * edge.cost
        );
    }

    println!("\nFlow paths:");
    for (i, path) in graph.get_flow_paths(source, sink).iter().enumerate() {
        print!("Path {}: ", i + 1);
        for (j, &(from, to, flow, cost)) in path.iter().enumerate() {
            if j > 0 {
                print!(" -> ");
            }
            print!("{} -> {} (flow: {}, cost: {:.2})", from, to, flow, cost);
        }
        let total_cost: f64 = path
            .iter()
            .map(|&(_, _, flow, cost)| flow as f64 * cost)
            .sum();
        println!(" | Total cost: {:.2}", total_cost);
    }
}
