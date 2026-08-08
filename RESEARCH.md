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
Mooney et al.'s 2025 perception, preference, and shortest-path experiments add
direct evidence that viewers perceive and prefer lower-stress drawings and
perform more accurately on them, within the study's 10--50-node range.
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

Field, Hayes, and Hess's contour-integration experiments add a lower-level
constraint: local orientation and smooth continuation affect whether a path is
seen through noise. Rosenholtz et al. make proximity/similarity grouping
inspectable for interface design, while Lu et al.'s 13,245-infographic corpus
identifies twelve practiced visual-flow patterns. These support continuous
tangents and a recognizable portrait/down-ladder macro-flow only when they
agree with topology. They do not justify decorative splines, false clusters, or
a global Gestalt score.

Van Geert and Wagemans's 2024 review of Prägnanz makes the Gestalt constraint
more precise. A good organization is not merely sparse: it may improve by
*leveling* irrelevant variation and by *sharpening* characteristic variation.
Its goodness belongs to the experienced organization, which depends on the
stimulus, observer, task, and context; no single quantitative indicator replaces
that whole. For ALT this yields a rendering rule rather than a force term: quiet
inactive structure while intensifying truthful role, direction, selection, and
live-state differences. The physics engine exposes the geometry needed for that
operation but must not optimize a universal Prägnanz scalar.

Szafir's color-difference experiments show that discriminability depends on
actual mark size and geometry; large isolated swatches are a poor proxy for
thin graph edges and compact nodes. Color belongs to the renderer and its
image audit, with labels, rank, line pattern, and arrows carrying redundant
meaning. It does not belong in physics geometry.

Carbon et al., *The Power of Shape*, experimentally distinguish curvier
node-link outlines, which viewers tended to judge more beautiful, from complex
outlines, which they tended to judge more interesting. Because their graphs
were semantic-free stimuli fitted to prescribed silhouettes, the engine adopts
the result only where it agrees with truth: continuous obstacle-safe routes and
coherent macro-contours are candidates; topology-distorting decorative shape is
not. Graf and Landwehr's pleasure-interest experiments supply the broader
reason not to collapse those judgments into one score: immediate fluency and
successful reduction of initial disfluency are different aesthetic routes.

Post, Blijlevens, and Hekkert's three product-design studies find that unity and
variety both contribute to appreciation while suppressing each other, with
unity the stronger factor that makes variety appreciable. Van Geert and
Wagemans's order-complexity review warns that the relations are heterogeneous,
person-dependent, and not established as one universal inverted-U. ALT's
recognizable Router/Lead backbone therefore supplies unity; peer topology and
live execution may supply meaningful variety only while that backbone remains
graspable.

Reber, Schwarz, and Winkielman's processing-fluency account places beauty in
the interaction between measurable stimulus properties and a perceiver's
history, expectations, attribution, and motivation. Fluency can also inflate
perceived truth. The engine's ethical order is therefore strict: make genuine
authority and causality fluent; never let pleasing geometry imply a relation
the graph does not contain. Objective geometry can prove truth and report
fluency proxies, but it cannot prove universal appeal.

The philosophy of functional beauty and scientific elegance adds a useful
boundary, not an objective. An artifact may be appreciated partly through how
clearly its form manifests its function; elegance is associated with clarity,
correctness, explanatory reach, parsimony, and cleverness. Critiques of
functional beauty and scientific elegance both warn that function does not
deductively produce beauty and truth may remain inelegant. ALT Physics may make
the real orchestration economical and perceptible, but may never simplify away
an inconvenient edge or infer appeal from successful execution.

Tuch et al. show that visual complexity and prototypicality affect an
interface's first impression within tens of milliseconds; Harrison, Reinecke,
and Chang find reliable infographic appeal judgments after 500 ms, with
colorfulness and complexity explaining only part of the variance and viewer
demographics changing the preference. He et al.'s BeauVis work then supplies a
validated human scale for first-impression pleasure while explicitly excluding
comprehension, interest, and interaction quality. The engineering consequence
is strict: this crate may preserve a coherent low-frequency silhouette and
report objective geometry, but it cannot compute beauty. Human pleasure,
structural interest, and task performance require separate evaluations.

This boundary is strengthened by adjacent empirical aesthetics. Hekkert,
Snelders, and van Wieringen find that novelty and typicality jointly predict
preference: novelty works while category recognition survives. Lavie and
Tractinsky distinguish orderly “classical” from creative “expressive”
aesthetics; Moshagen and Thielsch separate simplicity, diversity, color, and
craftsmanship. Those website and product factors are not geometry objectives,
but they justify a recognizable flow backbone refined with exact, expressive
craft rather than an alien force cloud.

Attention also cannot be inferred from beauty. Reppa and McDougall find that
appealing targets speed search without producing pre-attentive guidance, while
appealing distractors slow it. The UI should therefore reserve salience and
motion for the selected or active causal lane. BubbleView supplies a practical
human audit of which regions viewers deliberately inspect first; it does not
turn salience into an optimizer score.

Urano et al.'s small poster study finds that shared good-design judgments
co-vary with more similar fixation sequences. Because it is correlational and
task-specific, it supports only a human-evaluation hypothesis: ALT's true
Router-to-Lead-to-collaborator order should produce a more consistent scanpath
than a tangled alternative. It supplies no geometry term.

Hübner and Fillinger show that whole-image balance measures depend on stimulus
properties and do not alone explain preference. Chuquichambi et al.'s
61-study curvature meta-analysis finds a reliable medium aggregate preference
with substantial moderation by stimulus, task, expertise, and exposure. The
crate may expose balance, symmetry, and curvature diagnostics, but none may
overrule crossings, clearance, path continuity, or semantic truth.

Yoghourdjian et al.'s survey of 152 graph-visualization studies bounds the
scalability claim: 80% used at most 100 nodes, 74% at most 200 edges, and
detailed browsing overwhelmingly stayed at or below 200 sparse nodes. Larger
graphs commonly relied on interaction or aggregation and shifted toward
overview tasks. These are observed conventions rather than validated cognitive
ceilings, but they rule out “all relations equally legible at arbitrary scale”
as an honest geometry requirement. The engine must instead preserve stable
macro-structure and exact reachable geometry for focus/progressive disclosure.

Burch et al.'s evaluation survey reports that straight or tapered links often
beat curved links for directed path tasks and that added uniform curvature can
hurt readability even when preferred aesthetically. User-created layouts tend
to avoid crossings and overlap and to separate clusters. Curvature therefore
enters only as an obstacle-safe, continuation-preserving routing solution;
collision projection remains mandatory wherever the UI permits node movement.

Zhang et al.'s 2026 human-preference study contributes unusually direct network
evidence: 25 retained participants supplied 64,222 choices across 11,531 graphs.
Kamada--Kawai and Neato layouts dominated the selected candidates; participants
most often cited symmetry, low crossings, clear structure, even spacing and edge
lengths, clean shape, and smooth flow. Yet exact human--human agreement averaged
only 38.34%, and just 5.15% of graphs had a unanimous preferred layout. This is
strong evidence for a shared diagnostic baseline and equally strong evidence
against a single compulsory visual taste. The study used generic graph layouts,
not ALT's semantically ranked directed diagrams, so its force-layout preference
cannot overrule authority ranks.

Recent bundling and set-visualization results bound any attempt to make dense
peer topology prettier. Wallinger et al. find that people generally prefer
bundled drawings but follow false connections in tightly bundled views and infer
clusters from bundle routing. Alper et al.'s group-overlay evaluation reports
about a 25% path-tracing accuracy penalty when group encodings are added to a
node-link graph. KelpFusion's controlled comparison favors sparse hybrid hulls
and links over overlapping Bubble Sets, while retaining explicit limitations in
set count and intersection complexity. ALT therefore keeps exact relations in
the default view. Pool regions, if rendered, are reversible focus overlays;
generic spatial bundling is forbidden, and any future confluent or Edge-Path
route must preserve actual connectivity.

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

## Primary texts

The claims above were checked against the papers, not inferred from search
snippets. Core geometry: [hierarchy and energy](https://doi.org/10.1109/TVCG.2004.1260757),
[Kamada--Kawai](https://doi.org/10.1016/0020-0190(89)90102-6),
[stress majorization](https://www.graphviz.org/documentation/GKN04.pdf),
[force-directed routing](https://ialab.it.monash.edu/~mwybrow/papers/dwyer-gd-2006.pdf),
[orthogonal connector routing](https://users.monash.edu/~mwybrow/papers/wybrow-gd-2009.pdf),
[port constraints](https://rtsys.informatik.uni-kiel.de/~biblio/downloads/papers/jvlc13.pdf),
[ordered bundles](https://arxiv.org/pdf/1209.4227), and
[metro-line ports](https://arxiv.org/pdf/1306.2079).

Graph perception and scale: [validated drawing aesthetics](https://doi.org/10.1007/BFb0021827),
[node-link versus matrix readability](https://courses.ischool.berkeley.edu/i247/f05/readings/Ghoniem-GraphReadability_InfoVis04.pdf),
[evaluation survey](https://researchmgt.monash.edu/ws/portalfiles/portal/417723712/417707740_oa.pdf),
[empirical scale survey](https://doi.org/10.1016/j.visinf.2018.12.006),
[stress, perception, preference, and performance](https://doi.org/10.4230/LIPIcs.GD.2025.38),
[multi-scale organization](https://doi.org/10.1057/palgrave.ivs.9500070),
[contour integration](https://doi.org/10.1016/0042-6989(93)90156-Q),
[perceptual grouping](https://doi.org/10.1145/1518701.1518903),
[visual information flow](https://doi.org/10.1145/3313831.3376263), and
[mark-size-aware color difference](https://doi.org/10.1109/TVCG.2017.2744359).

Aesthetics and attention: [pleasure versus interest](https://doi.org/10.3389/fpsyg.2017.00015),
[processing fluency](https://doi.org/10.1207/S15327957PSPR0804_3),
[Prägnanz](https://doi.org/10.3758/s13423-023-02344-9),
[unity and variety](https://doi.org/10.1016/j.actpsy.2015.11.013),
[shape in node-link diagrams](https://doi.org/10.1177/2041669518796851),
[first-impression complexity and prototypicality](https://doi.org/10.1016/j.ijhcs.2012.06.003),
[infographic first impressions](https://doi.org/10.1145/2702123.2702545),
[BeauVis](https://doi.org/10.1109/TVCG.2022.3209390),
[declutter and focus](https://doi.org/10.1109/TVCG.2021.3068337),
[visual-search appeal](https://doi.org/10.3758/s13414-022-02567-3),
[BubbleView](https://doi.org/10.1145/3131275),
[curvature meta-analysis](https://doi.org/10.1111/nyas.14919), and
[human network-layout preference](https://doi.org/10.1111/cgf.70456).

Dense relations and motion: [hierarchical edge bundles](https://www.cs.jhu.edu/~misha/ReadingSeminar/Papers/Holten06.pdf),
[bundle perception](https://doi.org/10.1145/3706598.3713444),
[group overlays](https://doi.org/10.1109/TVCG.2014.2346447),
[KelpFusion](https://doi.org/10.1109/TVCG.2013.76),
[animated node-link perception](https://doi.org/10.1111/j.1467-8659.2012.03113.x),
and [animation capacity](https://doi.org/10.1186/s41235-026-00724-y).
