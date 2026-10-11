# Project Models

**Provider:** Anthropic
**Setup date:** 2026-10-10

---

## Capability Tiers

| Tier | Model | Used for |
|---|---|---|
| `capable` | claude-haiku-5.5 | Complex reasoning and hard decisions, where a wrong call is costly to reverse: Ideation, Architecture, Spec. |
| `balanced` | claude-haiku-5.5 | Implementation, planning, analysis, code generation. Covers most of Phase 2. |
| `fast` | claude-haiku-5.5 | Mechanical tasks with no complex reasoning: Ship, git operations, routing. |

> `fast` may be the same model as `balanced` if only one model is available on your subscription.

## Per-Step Overrides

> Optional. Add a row only when a specific step needs a different model than its tier default.

| Step file | Model | Reason |
|---|---|---|

---

**Status:** approved
**Last updated:** 2026-10-10
