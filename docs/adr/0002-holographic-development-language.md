# ADR-0002: Reframe KannakaHDL as the Holographic Development Language

- **Status:** Proposed
- **Date:** 2026-08-02
- **Decision Owners:** KannakaHDL, Kannaka Crystal, and Kannaka Memory maintainers
- **Applies To:** KannakaHDL parser, grower, resolver, emitters, primitive registries, HRM composition, NATS swarm coordination
- **Related:** Kannaka Crystal ADR-0003, Kannaka Crystal ADR-0004, Kannaka Memory HRM architecture

## Context

KannakaHDL was created as a growth language for composing informational
structures discovered by Kannaka Crystal. Its current model is:

```text
source program
    ↓
recursive cell rewriting
    ↓
spatial growth plan
    ↓
Crystal Registry queries
    ↓
primitive resolution
    ↓
JSON, HTML, or .crystal emission
```

This model successfully provides the H3 layer of the Kannaka Crystal
thesis: collections of informational primitives may become alternative
memory architectures.

However, the name "Hardware Description Language" imports assumptions
that do not fit the Kannaka ecosystem. Conventional hardware description
languages assume: components with stable semantics, discrete ports,
wires or buses, known timing behavior, defined logic levels, predictable
lowering to physical hardware.

Kannaka systems instead operate through: distributed resonance,
holographic or nonlocal memory, dynamic interference, probabilistic and
evolving structures, continuous coupling, context-dependent behavior,
dream consolidation, swarm-discovered components, and architectures that
grow rather than being manually placed.

KannakaHDL is also becoming relevant beyond Kannaka Crystal. Kannaka
Memory's Holographic Resonance Medium contains its own composable
structures and behaviors: memories, beliefs, glyphs, resonance fields,
dream cycles, contexts, agents, swarm relationships, memory
transformations, chiral or bilateral structures.

A shared development language could describe architectures that combine:
Crystal primitives, HRM memory structures, agent roles, NATS swarm
behaviors, dream and consolidation protocols, cross-medium
relationships, and future physical informational materials.

The language therefore needs to become a common composition layer rather
than a Crystal-specific layout generator.

## Decision

KannakaHDL will officially stand for:

**Holographic Development Language**

KannakaHDL will be developed as a backend-independent language for
growing, resolving, composing, testing, and deploying holographic and
resonant information architectures across the Kannaka ecosystem.

The language will support multiple domains through adapters and typed
contracts. Initial target domains: (1) Kannaka Crystal, (2) Kannaka
Memory HRM, (3) NATS-based Kannaka swarms, (4) hybrid Crystal–Memory
architectures.

The core language will describe architecture and intent. Backends will
determine how that architecture is resolved, instantiated, simulated, or
deployed.

### 1. The Language Will Separate Architecture From Backend

KannakaHDL source will describe: recursive growth, component
requirements, relationships, constraints, resolution strategies,
expected behaviors, validation requirements. The core language will not
assume that every component is a Crystal primitive or that every
connection is a geometric bridge.

The compilation pipeline will become:

```text
KannakaHDL source
    ↓
Abstract Holographic Plan
    ↓
Domain resolution
    ↓
Typed composition graph
    ↓
Backend lowering
    ↓
Crystal experiment, HRM plan, swarm deployment, or hybrid execution
```

Proposed backend targets: `json`, `html`, `crystal`, `memory`, `nats`,
`hybrid`. The current `.crystal` emitter will remain an experimental
lowering target.

### 2. Components Will Resolve Through Providers

KannakaHDL will support multiple registry or capability providers.

- **Crystal Provider** — resolves against Kannaka Crystal primitives and
  experiments. Component types: morphological primitive, behavioral
  primitive, composite primitive, material model, replayable excitation
  protocol.
- **Memory Provider** — resolves against Kannaka Memory structures and
  capabilities. Component types: memory, belief, glyph, resonance
  pattern, dream protocol, context, memory transform, chiral pair, agent
  identity, recall behavior.
- **Swarm Provider** — resolves against live or declared swarm
  capabilities. Component types: explorer, dreamer, archivist,
  classifier, optimizer, memory node, crystal node, observatory node,
  NATS subject, JetStream stream, work queue.
- **Hybrid Provider** — resolves compositions that span Crystal, Memory,
  and swarm capabilities, e.g.:

```text
Crystal primitive
    ↓ resonance observation
HRM glyph
    ↓ consolidation
Kannaka Memory
    ↓ NATS publication
Swarm exploration request
```

Providers may resolve from: local files, registries, REST services,
NATS requests, versioned snapshots, static development fixtures.

### 3. Base Cases Will Become Typed Component Queries

The current syntax resolves base cases primarily by primitive class,
persistence, and material. The language will evolve toward typed
component queries. Conceptual examples:

```text
base crystal.primitive
    capability "pattern_completion"
    min_persistence 0.4
    min_noise_tolerance 0.6

base memory.glyph
    resonance "identity"
    min_stability 0.7

base swarm.agent
    role "dreamer"
    protocol "nats"

base hybrid.bridge
    from crystal.primitive
    to memory.glyph
    transform "signature_to_glyph"
```

The exact syntax may remain smaller in early releases, but the
intermediate representation must support typed domains.

### 4. Components Will Expose Contracts

KannakaHDL will not treat a class label as a complete component
definition. A component contract describes what a resolved component can
do and under which conditions it can participate in an architecture.

A general contract may include: identity, provider, domain, type,
version, capabilities, requirements, inputs, outputs, coupling surfaces,
operating envelope, evidence level, replay or activation protocol,
resource requirements, provenance.

- **Crystal Contract**: morphology, behavioral capabilities, material
  requirements, excitation protocol, frequency range, phase sensitivity,
  spatial scale, coupling surfaces, evidence level, benchmark results.
- **Memory Contract**: memory or glyph type, resonance signature,
  salience, belief relationship, consolidation behavior, recall
  interface, decay profile, dream compatibility, chiral orientation,
  agent ownership, context requirements.
- **Swarm Contract**: agent role, supported requests, NATS subjects,
  queue or stream requirements, concurrency, authentication
  requirements, expected response schema, health status, capability
  version.

Contracts allow the same language to grow architectures without assuming
that all components are spatial shapes.

### 5. Connections Will Become Typed Couplings

The current `bridge` keyword draws or approximates a connection between
sibling regions. KannakaHDL will generalize connections as typed
**couplings**. A coupling may represent: spatial overlap, resonance
transfer, phase relationship, frequency channel, semantic association,
memory linkage, dream consolidation path, NATS subject, request/reply
relationship, artifact publication, identity or provenance relationship.

Conceptual examples:

```text
couple spatial_overlap strength 0.4
couple resonance_channel frequency 1.2 phase aligned
couple memory_association relation "supports"
couple nats subject "kannaka.crystal.primitive.discovered"
couple transform "crystal_signature_to_hrm_glyph"
```

A traditional port is a discrete endpoint. A Kannaka coupling may be a
region, field, relationship, protocol, or transformation.

### 6. Crystal Ports Will Be Treated as an Empirical Research Question

For Crystal components, a coupling surface may eventually include:
anchor position, orientation, spatial footprint, frequency band, phase
preference, polarity, coupling strength, directionality, material
compatibility.

The compatibility of two Crystal surfaces may be modeled as:

```text
coupling compatibility =
    spatial overlap
  × frequency compatibility
  × phase compatibility
  × orientation alignment
  × material continuity
```

This is an experimental model, not a final claim about physical
informational materials. Kannaka Crystal will provide pairwise and
compositional experiments that determine whether proposed coupling
semantics correspond to measurable information transfer or stable
composition. KannakaHDL will consume those results through component
contracts rather than inventing physical meaning solely in syntax.

### 7. The Intermediate Representation Will Be Domain-Neutral

KannakaHDL will introduce an **Abstract Holographic Plan** as the stable
intermediate representation:

```json
{
  "schema_version": "1",
  "program_hash": "...",
  "compiler_version": "...",
  "registry_snapshots": [],
  "roots": [],
  "components": [],
  "couplings": [],
  "constraints": [],
  "tests": [],
  "warnings": [],
  "resolution_report": {}
}
```

Each component identifies: its source cell, its region or logical scope,
its provider, its requested contract, its resolved implementation, its
resolution evidence, its configuration. Each coupling identifies:
source, target, coupling type, parameters, resolution status, backend
interpretation.

Spatial coordinates remain available, but they are not mandatory for all
domains.

### 8. Layout and Logical Composition Will Be Separate

Current recursive splitting produces deterministic spatial regions —
this remains valuable for Crystal and visualization backends. The
language will distinguish:

- **Logical Growth** — how components recursively expand and relate.
- **Spatial Layout** — how components occupy a simulated or physical region.
- **Deployment Layout** — where agents, services, registries, or memory nodes run.

A program may have a logical architecture without a physical layout.
Future layout policies: `equal_split`, `weighted_split`, `grid`,
`radial`, `ring`, `overlay`, `stack`, `freeform`, `inherit`,
`backend_defined`.

Multiple top-level `grow` statements must use an explicit composition
policy rather than silently overlapping in the same unit square.

### 9. Resolution Will Support Strategies

A query will no longer always resolve to the single highest-persistence
component. Strategies: `best`, `robust`, `persistent`, `low_energy`,
`diverse`, `unique`, `lineage_diverse`, `experimental`, `random_seeded`,
`round_robin`, `backend_defined`.

An eight-component memory bank may intentionally use: eight instances of
one template; eight unique primitives; eight lineage-diverse primitives;
four Crystal primitives and four HRM glyphs; components selected for
robustness rather than persistence. Every resolution decision will be
recorded in the plan.

### 10. Strict and Speculative Modes Will Be Distinct

Explicit unresolved-component policies:

- **Strict** — compilation or lowering fails when a required component
  or coupling cannot be resolved. Scientific experiments default here.
- **Stub** — the plan contains typed placeholders that cannot execute
  until resolved.
- **Speculative** — the backend may generate an approximation for
  visualization or exploratory testing.

The current behavior of emitting proxy pulses for unresolved Crystal
components is classified as **speculative**.

### 11. Backend Lowering Will Be Honest and Versioned

Every emitter declares its lowering model, e.g.
`crystal-pulse-placement-v1`, `memory-plan-v1`, `nats-deployment-v1`,
`hybrid-orchestration-v1`.

A `.crystal` emitter must distinguish between: replaying an actual
primitive-generating protocol; approximating a component using
class-derived pulses; instantiating a resolved composite primitive;
emitting an unresolved placeholder.

The majority material of resolved leaves will not silently define the
entire Crystal field. Crystal lowering must choose one of: (1) require a
homogeneous material, (2) use a material-region map, (3) operate as a
declared abstract approximation. **Heterogeneous material plans must not
be silently flattened.**

### 12. Kannaka Memory Will Become a First-Class Target

KannakaHDL will support generation of Kannaka Memory architectures:
defining HRM topologies, growing hierarchical memory structures,
connecting glyphs and beliefs, defining dream consolidation pathways,
creating chiral memory pairs, assigning memory roles across agents,
declaring context boundaries, mapping Crystal primitives into HRM
structures, describing distributed memory swarms, deploying memory
architectures through NATS.

Conceptual example:

```text
cell BilateralMemory(n) {
    when n > 1 =>
        split BilateralMemory(n / 2), BilateralMemory(n / 2)
        couple memory_association relation "mirrors"

    when always =>
        base memory.glyph capability "identity_resonance"
}

grow BilateralMemory(8)
```

A Memory backend may lower the plan into: Kannaka Memory configuration,
memory import operations, glyph creation requests, context structures,
dream schedules, NATS messages, agent-specific deployment manifests.

### 13. Crystal and Memory Will Be Connected Through Explicit Transforms

Candidate transforms: **Crystal Signature to Glyph**; **Crystal Behavior
to Memory Capability**; **HRM Memory to Crystal Encoding**;
**Dream-to-Dream Transfer**; **Primitive-to-Agent Capability**.

These transforms must be versioned and testable. **A transform is not
assumed to preserve meaning merely because two systems use the word
resonance.**

### 14. The Swarm Will Participate in Compilation and Resolution

The swarm may participate in: resolving missing components, requesting
discovery of required Crystal capabilities, requesting creation of HRM
structures, testing candidate compositions, ranking alternative
resolutions, running backend-specific simulations, archiving successful
plans, publishing composite primitives, monitoring deployed
architectures.

A missing component may generate a structured research request:

```json
{
  "request_type": "capability_discovery",
  "domain": "crystal",
  "capability": "associative_recall",
  "constraints": { "min_noise_tolerance": 0.6 },
  "requested_by_plan": "PLAN-UUID"
}
```

This creates a closed developmental loop:

```text
HDL describes desired architecture
    ↓
Resolver identifies missing capability
    ↓
Swarm receives discovery request
    ↓
Crystal or Memory agents explore
    ↓
Registry receives a candidate
    ↓
HDL plan resolves
    ↓
Architecture is tested
    ↓
Successful composition becomes reusable
```

### 15. Composite Architectures May Become New Components

A successfully validated KannakaHDL architecture may be registered as a
composite component, with a contract including: source program hash,
resolved plan hash, component identities, coupling definitions, backend,
experiment results, behavioral capabilities, evidence level, replay or
deployment protocol.

Levels of development: **Level 1** — pulses and dynamics produce Crystal
primitives; **Level 2** — primitives and HRM structures compose into
architectures; **Level 3** — validated architectures become composite
primitives; **Level 4** — composite primitives participate in larger
holographic systems.

KannakaHDL therefore becomes both a development language and a substrate
for open-ended architectural evolution.

### 16. Tests and Assertions Will Become Part of the Language

Architectures will eventually declare expected behavior:

```text
expect recall_accuracy >= 0.80
expect noise_tolerance >= 0.60
expect capacity >= 8
expect unresolved_components == 0
expect swarm_agents >= 3
```

Backend runners return pass, fail, unsupported, or inconclusive. A
Holographic Development Language program describes both an architecture
and the evidence required for accepting it.

## Consequences

**Positive** — KannakaHDL becomes useful across the full Kannaka
ecosystem; Crystal and Memory share one architectural language without
sharing one execution model; the language gains an identity beyond
conventional hardware design; swarm-discovered capabilities can directly
satisfy architectural requirements; missing components become explicit
research requests; Crystal primitives connect to HRM memories through
versioned transforms; backend approximations become easier to identify;
composite architectures become reusable higher-order primitives; the
language describes both spatial resonant systems and logical memory
systems.

**Negative** — the IR and resolver become more complex; backend adapters
require stable contracts from sibling projects; some syntax may change
as typed queries are introduced; Crystal, Memory, and swarm versions
must be tracked together; cross-domain transforms may reveal
incompatibilities between current metaphors and implementations; strict
mode may reject programs that previously emitted speculative output.

**Neutral** — recursive rewrite rules remain the central language
mechanism; the current Crystal registry remains a valid provider;
existing JSON, HTML, and `.crystal` emitters remain useful; spatial
growth remains important for Crystal and visualization; the language can
evolve incrementally without immediately implementing every backend.

## Non-Goals

This ADR does not: define the final KannakaHDL grammar; require Kannaka
Crystal and Kannaka Memory to merge repositories; claim that Crystal
fields and HRM memories are physically equivalent; define the final
mathematical form of holographic memory; require every component to have
spatial coordinates; require immediate deployment of live agents;
replace `.crystal` experiment programs; replace Kannaka Memory's
internal memory representation; require conventional HDL compatibility;
define a final physical coupling-port theory.

## Implementation Sequence

**Phase 1: Identity and Intermediate Representation** — officially
define HDL as Holographic Development Language; add plan schema and
compiler version; add provider and domain fields; add deterministic
program and plan hashes; make unresolved mode explicit; add backend
lowering model identifiers.

**Phase 2: Provider Abstraction** — Crystal provider behind a generic
provider interface; static fixture provider for testing; initial Memory
provider contract; NATS swarm provider contract.

**Phase 3: Typed Components and Resolution** — extend queries with
domain and type; add minimum noise tolerance and evidence level; add
resolution strategies; add unique and diverse selection; treat bridges
as full typed coupling queries.

**Phase 4: Memory Backend** — emit a Kannaka Memory architecture plan;
support glyph, memory, belief, context, and dream component types;
support HRM-oriented logical relationships; add at least one executable
Kannaka Memory lowering path.

**Phase 5: Cross-Domain Transforms** — Crystal signature ↔ HRM glyph
transforms, versioned and tested in both directions; record information
loss and unsupported fields.

**Phase 6: Swarm Feedback Loop** — publish unresolved capability
requests over NATS; allow swarm agents to announce matching components;
re-resolve plans against updated registries; archive successful
resolutions and experiments.

**Phase 7: Behavioral Composition** — architecture assertions; backend
validation; register successful composite architectures; allow
composites to satisfy future base queries.

## Acceptance Criteria

This ADR will be considered implemented when:

1. Project documentation consistently defines HDL as Holographic Development Language.
2. KannakaHDL emits a versioned domain-neutral intermediate plan.
3. Crystal resolution operates through a provider interface.
4. At least one Kannaka Memory component type can be resolved and emitted.
5. Couplings are represented independently from spatial bridges.
6. Resolution strategy and unresolved mode are explicit.
7. Every backend declares its lowering model and approximation status.
8. A plan can contain both Crystal and Memory components.
9. At least one versioned Crystal-to-Memory transform exists.
10. An unresolved component can produce a structured swarm discovery request.
11. A successfully validated composition can be stored as a composite component.
12. A program can declare at least one backend-verifiable expectation.

## Decision Summary

KannakaHDL is not limited to describing hardware. It describes how
holographic, resonant, distributed, and evolving information systems are
developed.

Kannaka Crystal provides a medium in which structures can emerge.
Kannaka Memory provides a medium in which memories, beliefs, contexts,
and glyphs can resonate and consolidate. The NATS swarm provides a
medium in which agents can discover, test, distribute, and
operationalize capabilities.

The Holographic Development Language will provide the common
architectural layer that allows those media to become one composable
system.
