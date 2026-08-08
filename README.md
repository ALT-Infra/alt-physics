# ALT Physics

ALT Physics is a provider- and UI-neutral Rust engine for deterministic graph
geometry. It accepts labeled rectangles, directed displacement preferences,
symmetric relationships, ports, prior positions, hard pins, and generic
one-axis position/offset/separation constraints; it returns node centers,
obstacle-safe routes, metrics, and solver diagnostics.

The engine owns mathematics and geometry only. It does not know what a Router,
Lead, specialist, peer, LLM, session, TUI, or GUI is. Applications translate
their own semantics into constraints. Renderers consume the resulting geometry.

## Pipeline

1. Validate and canonicalize ids.
2. Initialize the horizontal field spectrally and solve the directed hierarchy
   field as a weighted quadratic energy.
3. Minimize stress, hierarchy, axis constraints, repulsion, overlap, and
   temporal-prior energies with maintained L-BFGS and line-search
   implementations from `argmin`.
4. Project rectangular non-overlap and axis-separation inequalities exactly,
   then polish without moving fixed nodes.
5. Resolve free/fixed boundary ports and route around expanded rectangular
   obstacles through a deterministic visibility graph.
6. Report energy, stress, hierarchy error, overlaps, crossings, crossing angle,
   route length, bends, and solver behavior.

The public API is in [`src/model.rs`](src/model.rs). The evidence and rejected
alternatives are recorded in [`RESEARCH.md`](RESEARCH.md).
