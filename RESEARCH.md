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
bends, crossing angle, and incident-edge angular resolution load-bearing.
Gestalt evidence adds proximity, continuation, connectedness, contour,
orientation, symmetry, and stable figure-ground grouping, but those principles
must agree with semantic structure rather than merely decorate it. Symmetry,
mental-map preservation, compactness, and density judgment remain task-dependent;
no scalar average is allowed to stand in for visual quality. ALT Physics
therefore returns a metric vector and deterministic geometry rather than a
single undocumented “beauty score.” Deterministic multi-start selection follows
that same ordering: a lower optimizer energy cannot purchase an extra crossing
or a worse locally traceable junction.

The UI layer has separate obligations which do not belong in this crate. A
multi-scale image should expose the same Router/Lead/contributor structure at
overview, branch, and label scales; explicit group contours must reflect actual
possibly-overlapping pools; visual balance accounts for weight, not symmetry
alone; interaction may focus a subgraph without falsifying the stored graph.
Geometry metrics are therefore necessary evidence, never a substitute for
rendered-image audits and task-based inspection.

Primary perception and aesthetics sources include Bennett et al., *The
Aesthetics of Graph Visualization*; Wattenberg and Fisher, *A Multi-Scale Model
of Perceptual Organization in Information Graphics*; Rosenholtz et al.,
*Designing Interfaces and Graphics with Intuitive Perceptual Grouping*; Zhao et
al., *Understanding and Designing Visual Information Flows*; and Heinrich,
*Diagram Aesthetics: Beauty and the Sublime in—and through—Diagrams*. Together
they reject the false choice between legibility and beauty: a successful
diagram is a calm, perceivable gestalt whose operative structure can reveal
change without dissolving into visual noise.

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
