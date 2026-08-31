# AGENTS.md

This project uses **Spec-Driven Development (SDD)**. The specs in `specs/` are the source of
truth — read them before changing anything, and keep them in sync with the code.

## Source of truth
- `specs/constitution/` — the spine: `mission.md`, `tech-stack.md`, `roadmap.md`
- `specs/capabilities/<capability>/spec.md` — LIVING description of what the system does now
- `specs/changes/<name>/` — in-flight proposals (proposal, spec-delta, tasks, validation)
- `specs/changes/archive/` — applied changes, filed by date

## Workflow (per roadmap item)
1. **Propose** — on a branch, create a change under `specs/changes/<name>/`: proposal, spec-delta,
   tasks, and a validation scorecard.
2. **Implement** — build it in small task groups with frequent commits.
3. **Validate** — the human reviews against the scorecard (does it work, does it match the spec).
4. **Archive** — merge the spec-delta into `specs/capabilities/`, move the change to
   `specs/changes/archive/YYYY-MM-DD-<name>/`, and check off the roadmap item.
5. **Replan** — revisit the roadmap and constitution; update them as you learn.

## Rules for agents
- Read `specs/` before changing anything; treat `specs/capabilities/` as current truth.
- Keep specs and code in sync: when a decision changes, update the spec in the same change so the
  docs never drift from reality.
- Work in small, reviewable steps; don't dump large unreviewable diffs.
- Ask the human for key product and architecture decisions; capture omissions in the spec rather
  than silently inventing them.

> Portability: this file follows the open AGENTS.md convention so any compliant coding agent can
> adopt the workflow. For Claude Code, the root `CLAUDE.md` imports this file via a single line,
> `@AGENTS.md`, because Claude Code doesn't auto-load AGENTS.md. Keep AGENTS.md canonical and let
> other agent-specific files point here rather than duplicating content.
