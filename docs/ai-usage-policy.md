---
title: "AI usage: default quota policy"
description: "Guidance for quota-aware waiting and safe development handoffs"
---

AI subscription limits are operational constraints, **not** project acceptance
gates. Follow the default policy below when quota information is available.

- Prefer bounded work and concise handoffs over repeatedly checking usage.
- Never spawn an agent merely to monitor quota.
- Missing tools or ambiguous quota information do not block normal work.
- Do not relax statistical, data-integrity or safety checks just to fit a budget;
  report what has not been validated and leave a focused next step.
- Commit, push and merge only within the user's authorization. Quota pressure
  never justifies bypassing required checks or merging unfinished work.

## codexbar

Use `codexbar` when available to identify AI budgets for 5-hour and weekly windows.

```bash
codexbar --provider codex
```

Default policy:

- Session (5-hour window): When 10% or less is left, wait until session limits reset
  (you may use `sleep`); allow a two-minute buffer. For example, if the limit
  resets in 15 minutes, wait 17 minutes.
- Weekly (7-day window): When 3% or less is left, wrap up ongoing tasks safely,
  commit/push when authorized, and merge a PR into main only if it is ready and
  all required checks are satisfied. Otherwise leave a focused WIP handoff.
- Weekly wrap-up takes precedence over waiting for a session reset. Record what
  remains when work cannot continue; do not promise an unattended weekly restart.

Note the policies above can be overriden by any subsequent user-provided steering prompts or requests.
