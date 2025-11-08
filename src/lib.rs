use std::collections::binary_heap::BinaryHeap;

use ordered_float::OrderedFloat;

// Type for flows/capacities. Must be signed.
type F = isize;
// Type for weight/costs. Must be signed.
type W = f32;
static FINF: F = 10isize.pow(18);
static WINF: W = 1e18;

struct Edge {
    // Constant values.
    /// index of neighbour
    v: usize,
    /// index of reverse edge
    r: usize,
    /// capacity
    cap: F,
    /// cost
    cost: W,

    // Variable.
    /// current flow.
    f: F,
}

impl Edge {
    fn new(v: usize, r: usize, cap: F, cost: W) -> Self {
        Edge {
            v,
            r,
            cap,
            cost,
            f: 0,
        }
    }
}

pub struct FlowGraph {
    edges: Vec<Vec<Edge>>,
}

impl FlowGraph {
    pub fn new(n: usize) -> Self {
        FlowGraph {
            edges: (0..n).map(|_| vec![]).collect(),
        }
    }
    /// Add undirected edge from u to v with given capacity and cost.
    pub fn add_edge(&mut self, u: usize, v: usize, c: F, cost: W) {
        // fwd edge
        let r = self.edges[v].len();
        self.edges[u].push(Edge::new(v, r, c, cost));

        // rev edge
        let r = self.edges[u].len() - 1;
        self.edges[v].push(Edge::new(u, r, c, -cost));
    }
    /// Add directed edge from u to v with given capacity and cost.
    pub fn add_arc(&mut self, u: usize, v: usize, c: F, cost: W) {
        // fwd edge
        let r = self.edges[v].len();
        self.edges[u].push(Edge::new(v, r, c, cost));

        // rev edge
        let r = self.edges[u].len() - 1;
        self.edges[v].push(Edge::new(u, r, 0, -cost));
    }
}

/// Queue element for Dijkstra, sorted in reverse so that the binary heap has
/// small `w` first, instead of large `w` by default.
struct Q {
    /// Target vertex.
    u: usize,
    /// Current flow to u.
    c: F,
    /// Total cost.
    w: W,
}
impl PartialEq for Q {
    fn eq(&self, other: &Self) -> bool {
        self.w == other.w
    }
}
impl Eq for Q {}
impl PartialOrd for Q {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.w.partial_cmp(&self.w)
    }
}
impl Ord for Q {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        OrderedFloat(other.w).cmp(&OrderedFloat(self.w))
    }
}

/// Min-cost-max-flow implementation using Edmonds-Karp with Dijkstra.
pub struct MCMF<'g> {
    /// the underlying graph
    g: &'g mut FlowGraph,
    /// the number of nodes
    n: usize,
    /// the 'start' node of the flow
    s: usize,
    /// the 'target' node of the flow
    t: usize,
    /// The current potentials
    pot: Vec<W>,
}

impl<'g> MCMF<'g> {
    pub fn new(g: &'g mut FlowGraph, s: usize, t: usize) -> Self {
        let n = g.edges.len();
        Self {
            g,
            n,
            s,
            t,
            pot: vec![WINF; n],
        }
    }
    pub fn run(&mut self) -> (F, W) {
        // I'm lazy :)
        let Self { g, n, s, t, pot } = self;
        let n = *n;
        let s = *s;
        let t = *t;

        let mut maxflow: F = 0;
        let mut cost: W = 0.0;
        pot[s] = 0.0;
        for _ in 0..n - 1 {
            let mut relax = false;
            // Try to Relax Dijkstra potentials.
            for u in 0..n {
                if pot[u] != WINF {
                    for e in &g.edges[u] {
                        if e.cap > e.f && pot[u] + e.cost < pot[e.v] {
                            pot[e.v] = pot[u] + e.cost;
                            relax = true;
                        }
                    }
                }
            }
            if !relax {
                break;
            }
        }
        for u in 0..n {
            if pot[u] == WINF {
                pot[u] = 0.;
            }
        }

        // Queue for Dijkstra.
        let mut q = BinaryHeap::new();

        // Indices (u, nbi) of the best/chosen edge to v, to trace back the path.
        // The second component is the index in `g.edges[u]` of the edge.
        let mut p = vec![None; n];

        // Best distances.
        let mut dist = vec![WINF; n];

        loop {
            // Clear all structures.
            q.clear();
            p.fill(None);
            dist.fill(WINF);

            q.push(Q {
                u: s,
                c: FINF,
                w: 0.,
            });
            dist[s] = 0.;

            // current flow.
            let mut f: F;
            // total flow to the target.
            let mut tf: F = -1;

            while let Some(Q { u, c, w }) = q.pop() {
                f = c;
                if w != dist[u] {
                    // outdated queue element; a better path to u was found meanwhile.
                    continue;
                }
                // Update the flow to the target if none found yet.
                if u == t && tf < 0 {
                    tf = f;
                }

                // nbi: neighbour index.
                for (nbi, e) in g.edges[u].iter().enumerate() {
                    let d = w + e.cost + pot[u] - pot[e.v];
                    if e.cap > e.f && d < dist[e.v] {
                        dist[e.v] = d;
                        q.push(Q {
                            u: e.v,
                            c: std::cmp::min(f, e.cap - e.f),
                            w: d,
                        });
                        p[e.v] = Some((u, nbi));
                    }
                }
            }
            // No new path to t found.
            let mut it = p[t];
            if it.is_none() {
                return (maxflow, cost);
            };
            // The new flow we ended up finding.
            f = tf;
            // Update the total flow.
            maxflow += f;
            // Fails once we get to `s`.
            while let Some((u, nbi)) = it {
                let e = &mut g.edges[u][nbi];
                // Add flow to forward edge.
                e.f += f;
                // Copy values we need, and drop the reference, so we can mutate
                // the reverse edge r below.
                let Edge {
                    v: ev,
                    r: er,
                    cost: ecost,
                    ..
                } = *e;
                // reverse edge.
                let r = &mut g.edges[ev][er];
                cost += f as W * ecost;
                // Remove flow from reverse edge.
                r.f -= f;
                it = p[r.v];
            }
            // Update potentials.
            for u in 0..n {
                if dist[u] != WINF {
                    pot[u] += dist[u];
                }
            }
        }
    }
}
