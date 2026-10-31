# Lab Engineering Standards Adoption

**Status:** Proposed for adoption in this repository.  
**Canonical source:** [Framework Desktop AI Lab — Engineering Governance](https://github.com/Dukeraider2/framework-desktop-ai-lab/tree/main/docs/engineering)  
**Canonical change set:** [Governance and knowledge-base PR #2](https://github.com/Dukeraider2/framework-desktop-ai-lab/pull/2)

This repository adopts the shared engineering baseline maintained in the canonical AI lab repository. The canonical documents are authoritative for shared process and engineering policy; this file provides a concise local entry point. Project-specific requirements may add stricter controls but must not silently weaken the shared baseline.

## Required engineering practices

1. **Evidence first:** Verify hardware, software, APIs, package names, compatibility, and configuration against authoritative sources. Clearly distinguish verified facts, estimates, hypotheses, and untested proposals.
2. **Specification before implementation:** Significant work begins with scope, functional and non-functional requirements, acceptance criteria, dependencies, risks, resource expectations, and an accessible core reference package.
3. **Three-stage workflow:** Stage 1 task specification and reference package; Stage 2 implementation with rationale summaries, source provenance, tests, limitations, resource measurements, and time report; Stage 3 independent review of coherence, evidence, code, tests, security, and resource impact.
4. **Independent review:** An implementer does not approve its own work. Review findings and scoring must be documented. Critical security or policy failures block approval regardless of score.
5. **Podman first:** Podman is the required default container runtime. A Docker-based upstream project or installer must trigger source/dependency and compatibility analysis, a Podman migration/port where feasible, rebuild, and validation. Do not install Docker merely to bypass this requirement; unresolved incompatibilities go to the project owner for an explicit decision.
6. **Resource efficiency:** Measure relevant CPU, RAM, GPU/VRAM, storage/network I/O, startup/idle overhead, latency, throughput, and power where measurable. Compare against a baseline and document tradeoffs. Prefer lean Python for orchestration and automation where appropriate; use other languages or hardware-specific optimizations when benchmarks justify them.
7. **Hardware-aware optimization:** Verify CPU instruction sets, SIMD/vectorization, accelerators, parallelism, memory locality, and optimized libraries on the target system. Detect capabilities at runtime and provide correct fallbacks where portability is required.
8. **Least privilege and escalation:** Agents request access with the resource, permission, reason, goal, scope, and duration. A denial must include a workable alternative. If no safe alternative resolves the need, escalate risks and options to the project owner for the final decision.
9. **Reproducibility and auditability:** Track decisions, sources, versions, commands, tests, review findings, approvals, and deployment/rollback instructions in GitHub. Never commit secrets or sensitive operational data.
10. **Knowledge-base contributions:** Organize durable knowledge by project and hardware/equipment. Equipment records must explain integration with the lab. Index manufacturer documentation separately. Contributions remain Proposed / Unverified until independently checked for provenance, revision, accuracy, and applicability.

## Task reporting

Use the canonical Stage 1, Stage 2, and Stage 3 templates in the [engineering templates directory](https://github.com/Dukeraider2/framework-desktop-ai-lab/tree/main/docs/engineering/templates). Record estimated and actual active time, blocked/wait time, total elapsed time, and estimate variance for each stage. Do not invent measurements or precise time records.

## Project-specific applicability

Apply these standards in a way that respects this repository's domain, safety constraints, licenses, and existing architecture. Keep domain-specific validation and professional review requirements. Do not represent a prototype, research concept, or untested integration as production-ready.

## Adoption and change control

This file is a local adoption pointer, not a copy of every canonical policy. Changes to shared standards should be proposed in the canonical repository through a reviewed pull request. Local deviations must be documented with rationale, scope, risk, and project-owner approval.
