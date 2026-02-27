# Sprint 04: Beta Launch

**Duration:** 2 weeks
**Theme:** 100 beta users, NPS > 40
**Goal:** Controlled launch to a hand-picked group; collect real feedback; fix the top blockers before public launch
**Dependency:** Sprint 03 complete (onboarding < 5 min, docs live)

---

## Context

Beta is not a soft launch — it is a structured data collection exercise. The goal is
not 100 users for vanity; it is 100 users whose feedback reveals the 3–5 highest-impact
improvements before the public launch.

A successful beta means: users activate, use the product multiple times, and are willing
to tell you exactly what is broken or confusing.

---

## Beta Criteria

### Who Gets In

The first 100 beta users should be:
- Actively building with LLMs or AI agents (not just curious)
- Willing to spend 30–60 minutes actually using Fetchium
- Reachable for a 20-minute feedback call
- Diverse across: Python/Rust/TypeScript, beginner/expert, use case type

**Recruit from:**
- Your personal network: colleagues, former co-workers, developer friends
- Twitter/X: DM developers who tweet about LLM agents and mention a beta
- Discord: LangChain, CrewAI, AutoGen, LocalLLaMA servers — post in #general
- LinkedIn: message AI engineers at companies you know
- Reddit: r/LocalLLaMA, r/MachineLearning — post a beta access request

**Not suitable for beta:**
- Passive followers who click a button and never try the product
- Non-developers (they cannot evaluate a CLI tool fairly)
- Competitors who are obviously doing research

### Beta Access Mechanism

Simple: create a private waitlist form (Typeform or Google Form):
- Name, email, Twitter/GitHub handle
- "What are you building?" (1–3 sentences)
- "Which AI framework do you use?" (LangChain / CrewAI / AutoGen / raw API / other)
- "What is your biggest pain with web fetch today?" (open text)

Review submissions daily. Send invite codes to approved applicants within 48 hours.

---

## Week 1: Setup & Onboard

### Day 1: Beta Infrastructure

**Task 4.1 — Beta invite code system**

Add simple invite code support to the API registration flow:
- Codes: 16-character alphanumeric strings, single-use
- Store in SQLite: `beta_codes(code TEXT PRIMARY KEY, used_by TEXT, used_at TIMESTAMP)`
- New accounts during beta period require a valid code (or bypass with a flag)
- Admin endpoint: `POST /admin/beta-codes` generates a batch of codes

**Task 4.2 — Beta feedback infrastructure**

- Set up a dedicated `#beta-feedback` Discord channel (visible only to beta users)
- Create a Typeform survey: "Beta Feedback — 5 minutes" (see questions below)
- Prepare a 20-question NPS form via Delighted (free tier: 250 surveys/month)
- Create a `beta-bugs` GitHub label for beta-reported issues

**Task 4.3 — Beta documentation**

Write a `beta-guide.md` sent to every beta user:
- What is Fetchium (1 paragraph)
- How to install and run `fetchium quickstart`
- 3 tasks to try (see below)
- How to report bugs (Discord `#beta-feedback` or GitHub Issues)
- How to request the NPS survey (email `beta@fetchium.com`)

**3 Beta Tasks (guide users to the core value loop):**
1. Fetch a URL and inspect the output: `fetchium fetch "https://hn.algolia.com/?q=llm"`
2. Run an AI query: `fetchium ai "what are the most popular LLM frameworks in 2025?"`
3. Integrate into a Python script: use the provided 20-line starter script

### Day 2–3: Recruit 50 Beta Users

**Target: 50 invited by end of Day 3.**

Outreach channels and scripts:

**Discord template:**
> "Hey — I'm building Fetchium, an open-source web fetch tool designed for AI agents.
> Looking for 50 beta testers who are actively building with LLMs. You'd get early
> access + personal support. DM me if interested — takes 30–60 min."

**Twitter DM template:**
> "Hey [name] — saw your thread about [LangChain/agents/etc.]. I'm running a private
> beta for Fetchium, a typed web fetch tool for AI agents. Would you want to try it?
> 30 min of your time, you get early Pro access. No pressure either way."

**Task 4.4 — Onboard first 20 beta users personally**

For the first 20 users:
- Send a personal email (not a template) with their invite code
- Offer a 15-minute onboarding call via Calendly
- Follow up if they haven't activated within 48 hours (one time only)

### Day 4–7: Monitor and Triage

**Task 4.5 — Daily monitoring during beta week 1**

Every day (30 minutes):
1. Check PostHog: how many beta users activated? What did they fetch?
2. Check Discord `#beta-feedback`: any reports, questions, praise?
3. Check GitHub Issues: any new `beta-bugs` issues?
4. Respond to every piece of feedback within 4 hours

**Task 4.6 — Bug severity triage**

| Severity | Definition | Response Time |
|----------|-----------|--------------|
| P0 (Critical) | Product doesn't work at all for a use case | Fix same day |
| P1 (High) | Major feature broken, no workaround | Fix within 3 days |
| P2 (Medium) | Feature degraded, workaround exists | Fix before public launch |
| P3 (Low) | Minor issue, cosmetic, rare | Fix in post-launch sprint |

---

## Week 2: Feedback Analysis & Fixes

### Day 8–9: Collect Structured Feedback

**Task 4.7 — NPS survey delivery**

Send the NPS survey (Delighted) to all beta users who have been active for at least 5 days.
- Email subject: "Quick question about Fetchium — 2 minutes"
- NPS question: "How likely are you to recommend Fetchium to a developer friend? (0–10)"
- Follow-up open text: "What's the main reason for your score?"
- Follow-up: "What would make Fetchium a 10/10 for you?"

**Task 4.8 — Qualitative feedback calls**

Schedule 5–8 user interviews with beta participants (mix of high and low NPS scores):
- Duration: 20 minutes
- Format: screen share + questions (Loom or Zoom)
- Questions:
  1. Walk me through how you tried to use Fetchium
  2. What worked well?
  3. What confused you or frustrated you?
  4. Did you integrate it with anything? What happened?
  5. What would you need to see to pay $12/month for this?

Record with permission; take notes in a shared doc.

**Task 4.9 — Feedback synthesis**

After all feedback is collected:
1. Group feedback by theme (Affinity mapping — use sticky notes or FigJam)
2. Count: how many users mentioned each theme?
3. Rank themes by impact and frequency
4. Create a prioritized list of "things to fix before public launch"

Common themes to expect:
- "I couldn't figure out how to configure AI keys"
- "The output format was confusing"
- "It was slow" (specific to which mode?)
- "LangChain integration didn't work"
- "I wanted [feature X] that isn't there yet"

### Day 10–12: Top Bug Fixes

**Task 4.10 — Fix the top 5–10 P0/P1 bugs**

Based on feedback synthesis, fix the highest-impact issues.

Likely fixes from beta (predictions based on common patterns):
1. **Auth token confusion** — `fetchium setup` flow unclear for API key entry
2. **SearXNG not running** — better error message + auto-start suggestion
3. **AI mode timeout** — need progress indicator + configurable timeout
4. **Output format** — some users wanted JSONL, others wanted Markdown
5. **Rate limit error** — message should be clearer about what the limit is

**Each fix:**
- Open a GitHub Issue with the beta-user quote and `beta-bugs` label
- Fix, test, commit with `fix: [description]`
- Close the issue with a comment mentioning the beta user

### Day 13: Recruit Second Wave (50 more)

**Task 4.11 — Expand to 100 beta users**

With the P0/P1 bugs fixed, invite 50 more beta users from the waitlist.
These users get a slightly better product and serve as a signal that the fixes worked.

**Measure:** Did the second wave's activation rate improve vs. the first wave?

### Day 14: Beta Wrap-Up

**Task 4.12 — Beta thank-you**

Send every beta user a personal thank-you email:
- What you learned from their feedback
- What you fixed because of them
- They get 3 months of Pro free at launch (no strings attached)
- Early access to public launch — they can share the launch post

**Task 4.13 — Beta retrospective document**

Write a 1-page document:
- What was the top feedback theme?
- What was the NPS score?
- What did we fix?
- What did we defer to post-launch?
- Are we ready to launch publicly?

---

## Feedback Form Questions (Full Set)

For the beta feedback survey (Typeform, not NPS):

1. How easy was it to install Fetchium? (1–5 scale)
2. How easy was it to get your first result? (1–5 scale)
3. Which mode did you use most? (fast / extract / clean / ai / deep / headless / research)
4. What did you use Fetchium for? (open text)
5. What was the most frustrating part? (open text)
6. What was the best part? (open text)
7. Did you try to integrate Fetchium with anything? If so, what? (open text)
8. Did the integration work? (yes / mostly / no / didn't try)
9. Would you use Fetchium in a real project? (yes / maybe / no)
10. What would make you pay $12/month for this? (open text)

---

## Definition of Done

Sprint 04 is complete when:
- [ ] 100 beta users have been invited (not just signed up — invited with code)
- [ ] >= 70 beta users have activated (completed at least one fetch)
- [ ] NPS score is >= 40 from >= 30 respondents
- [ ] All P0 bugs are fixed
- [ ] All P1 bugs are fixed or have a documented workaround
- [ ] Beta retrospective document written
- [ ] "Ready for public launch" decision made by the founder

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Beta users invited | 100 |
| Activation rate | >= 70% |
| NPS score | >= 40 |
| P0 bugs | 0 open |
| P1 bugs | 0 open |
| User interviews completed | >= 5 |
| Feedback themes identified | >= 5 |
| Fixes shipped during beta | >= 5 |

---

## What "Beta Failed" Looks Like (and What to Do)

**NPS < 30:** The product has a fundamental UX or reliability problem. Do not launch publicly.
Extend beta for 2 more weeks and fix the root cause.

**Activation rate < 40%:** The onboarding is still too hard. Run another round of Sprint 03
improvements. The problem is in installation, quickstart, or the first-run experience.

**P0 bug after launch:** Roll back immediately. Fix. Re-deploy. Communicate to beta users.

**No feedback from users:** They are not engaged enough to care. This is a signal the
product is not compelling — not a notification problem. Talk to users directly.
