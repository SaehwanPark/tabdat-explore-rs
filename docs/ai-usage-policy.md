---
title: "AI usage: optional budget awareness"
description: "Guidance for resource-conscious agent development without quota-driven orchestration"
---

AI subscription limits are operational constraints, **not** project acceptance
gates. A developer may check available usage before a particularly long or
expensive task using an available tool. Do not depend on a specific monitoring
command, harness, model tier or quota-percentage threshold.

- Prefer bounded work and concise handoffs over repeatedly checking usage.
- Never spawn an agent merely to monitor quota.
- Do not `sleep`, poll for quota resets, or promise to resume unattended after a
  reset. Stop cleanly and record what remains when work cannot continue.
- Missing or ambiguous quota information does not block normal work.
- Do not relax statistical, data-integrity or safety checks just to fit a budget;
  report what has not been validated and leave a focused next step.

The previous codexbar-specific thresholds are historical operational practice,
not requirements for future contributors or agents.
