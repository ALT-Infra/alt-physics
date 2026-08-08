# Research basis

Status: implementation evidence ledger. Primary texts take precedence over
search-engine synthesis; benchmarks and human-perception results are not proofs
of universal beauty.

## Physics-first core

- Carmel, Harel, and Koren, *Combining Hierarchy and Energy for Drawing Directed
  Graphs*: encode directional flow as a continuous quadratic hierarchy energy,
  including cycles and mixed directed/undirected structure, instead of inferring
  direction from discrete ranks.
- Kamada and Kawai, *An Algorithm for Drawing General Undirected Graphs*: graph
  distance supplies ideal spring distance.
- Gansner, Koren, and North, *Graph Drawing by Stress Majorization*: stress is a
  measurable energy; monotone majorization is a baseline against which general
  optimizers must be checked.
- Dwyer, Marriott, and Wybrow, *Integrating Edge Routing into Force-Directed
  Layout*: rectangular separation constraints and routing belong inside the
  geometry pipeline rather than after a semantically unrelated layered layout.
- CoDaFlow: constrained stress can combine flow, rectangular dimensions, ports,
  and orthogonal routing, but its crossing/runtime tradeoffs require separate
  measurement.
- CoSEP: port constraints materially change the equilibrium; ports cannot be a
  renderer-only afterthought.
- fCoSE: spectral initialization plus constrained force refinement is a useful
  practical pattern, not a guarantee of minimum crossings.
- `(SGD)^2`, AutoFDP, t-FDP, SNAP-tFDP, and stochastic-stress work show that
  objective composition and force evaluation can scale. Their large-network
  community objectives do not directly transfer to small labeled rectangles.
- Persistent-homology and recent coordinate-Newton initialization work motivate
  topology-aware initialization and finite-budget evaluation.

## Human-facing objectives

Controlled graph-reading studies consistently make crossings, path continuity,
bends, and crossing angle load-bearing. Symmetry, mental-map preservation, and
compactness are task-dependent; no scalar average is allowed to stand in for
visual quality. ALT Physics therefore returns a metric vector and deterministic
geometry rather than a single undocumented “beauty score.”

## Routing

The routing stage starts from the final energy equilibrium. It uses exact
rectangle boundaries and obstacle expansion, then a visibility graph and stable
shortest-path tie-breaking. Orthogonal/libavoid-style routing, topology-preserving
incremental routes, spline smoothing, bundle separation, and affected-edge-only
updates remain planned independent stages. Smoothing may not be admitted until
the resulting curve is proved obstacle-safe.

## Explicit non-goals

- A generic force-directed cloud that erases directed flow.
- Role inference inside the geometry crate.
- Random layouts whose seed or ordering is implicit.
- Claiming that one metric, one paper, or one physics analogy proves objective
  beauty.
- Reimplementing numerical optimization or linear algebra already maintained by
  stronger libraries.
