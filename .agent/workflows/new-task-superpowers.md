---
description: Superpowers Workflow - Mandatory process for handling new tasks (Brainstorm -> Plan -> Execute)
---

# Superpowers Workflow

This workflow enforces the "Superpowers" methodology. It is the **MANDATORY** standard procedure for handling **NEW TASKS** or complex requests.

## 1. Brainstorm (头脑风暴)

**Goal**: Deeply understand the problem and design the solution _before_ writing code.
**Actions**:

- **Analyze**: Read the user's request carefully.
- **Clarify**: If requirements are vague, ask questions.
- **Design**: Explore approaches. For complex tasks, write a short design summary or RFC draft in the chat or a scratchpad.
- **Output**: A clear conceptual understanding or design.

## 2. Write Plan (写计划)

**Goal**: Create a detailed, step-by-step implementation plan.
**Actions**:

- **Document**: Create or update `implementation_plan.md`.
- **Break Down**: Split work into atomic, verifiable tasks.
- **Detail**: specific file paths, method names, and data structures.
- **Verify Design**: Define how each step will be verified (automated tests or manual checks).
- **Review**: **MUST** present the plan to the user for approval before proceeding.

## 3. Execute Plan (按计划实现和测试)

**Goal**: Implement the solution following the approved plan.
**Actions**:

- **Iterate**: Execute one checklist item from `implementation_plan.md` at a time.
- **TDD / Verification**:
  - Write a test (or use an existing one) that fails.
  - Write minimal code to pass.
  - Refactor.
- **Verify**: Run tests after every significant change.
- **Update Status**: Mark items as completed in `implementation_plan.md` (or `task.md` if synced).

---

**Trigger**:

- Automatically applies when the user provides a **new non-trivial task**.
- Explicitly invoked when user mentions "superpowers" or "Brainstorm/Plan/Execute".
