# Phase 6 — Mobile & Global

**Timeline:** Months 25–36
**Theme:** 10x distribution. Meet users where they are — on their phones, in their browsers, in their language.
**Team:** 10-15 people by M36 (engineers, mobile devs, marketing, sales, customer success)

---

## The Distribution Layer

By Month 24, Fetchium has $10M ARR driven primarily by API users and enterprise teams. Phase 6 unlocks the next 10x by attacking distribution:

- **Mobile apps:** 5B smartphone users. Even 0.01% = 500K DAU.
- **Browser extension:** Replaces the search bar for power users. 1M installs = 1M daily impressions.
- **i18n:** English is 25% of the internet. Non-English markets are largely untapped by AI search tools.
- **Regional infrastructure:** Data residency laws (GDPR, India's PDPB, Brazil's LGPD) require local data storage.

**Benchmark:** Perplexity's mobile app drives 40%+ of their MAU. Browser extensions drive 20%+ of organic installs for search tools.

---

## Success Criteria

| Metric | Target |
|--------|--------|
| ARR | $100,000,000 |
| MRR | $8,333,000 |
| Total MAU | 5,000,000+ |
| Mobile DAU | 500,000+ |
| Browser extension installs | 1,000,000+ |
| Countries with paying users | 50+ |
| Languages in UI | 10 |
| API uptime (global) | 99.99% |
| Enterprise accounts | 100+ |

---

## Mobile Apps

### iOS App

**Tech stack:** Flutter 3.x (code-sharing with Android)
**Distribution:** App Store (direct download + TestFlight for beta)

**Feature set (v1.0):**
- Search mode: type or speak a query, get AI-synthesized answer
- Research mode: start a deep research job, get notified when done
- Knowledge Base: browse, search, and annotate your personal KB
- Monitor inbox: alerts when tracked topics update
- Offline mode: read cached KB nodes without internet
- Widgets: iOS 16+ home screen widgets (quick search, monitor alerts)
- Siri Shortcuts: "Hey Siri, search Fetchium for..."

**Voice search:**
- On-device speech recognition (Apple's SFSpeechRecognizer)
- No audio sent to Fetchium servers
- Supports 10 languages (matching UI localization)

**Build + release pipeline:**
- Codemagic CI/CD: push to main → TestFlight within 15 minutes
- Shorebird code push: Dart-level patches deployed in < 2 minutes
- App Review: plan 1-week buffer for App Store review cycles
- TestFlight beta: 1,000 users invited before public launch

**iOS-specific features:**
- Share extension: highlight text anywhere → "Search Fetchium" context menu
- Safari extension: Fetchium button in Safari toolbar
- iCloud sync: KB backed up to user's iCloud (E2E encrypted)

### Android App

**Distribution:** Google Play Store + direct APK (for enterprise sideloading)

**Feature set:** Mirrors iOS v1.0 plus:
- Taskbar widget (Android 12+)
- Google Assistant integration
- Material You theming (dynamic color)
- Background sync: KB updates sync when on Wi-Fi

**Build pipeline:**
- Same Flutter codebase as iOS
- Codemagic: push to main → Play Store internal track within 20 minutes
- FastLane for metadata + screenshot management

---

## Browser Extension

### Chrome Extension (primary)

**Distribution:** Chrome Web Store
**Estimated install conversion:** 15-25% of extension page visitors

**Features:**
- **Search hijack (opt-in):** Replace Google results page with Fetchium sidebar showing AI synthesis
- **Inline research:** Right-click any selected text → "Research with Fetchium"
- **Page summarizer:** Click extension icon → AI summary of current page
- **KB clipper:** Save current page to your Fetchium KB with one click
- **Citation checker:** Highlight a claim → Fetchium verifies against sources

**Popup UI:**
- Search bar (uses current page context as implicit context)
- Recent searches
- KB quick-add button
- Monitor inbox badge (red dot for unread alerts)

**Privacy:**
- No page content sent without explicit user action
- API key stored in browser's secure storage (not localStorage)
- Extension permissions: activeTab (not `<all_urls>`) to minimize footprint

### Firefox Extension

**Distribution:** addons.mozilla.org (AMO)
Same codebase as Chrome (WebExtension API compatible), separate AMO listing.

### Safari Extension

**Distribution:** Mac App Store (bundled with a tiny Mac app)
Safari requires a native app wrapper. Distribute as "Fetchium for Safari" Mac app.

---

## Internationalization (i18n)

### Languages — Phase 6 Launch Order

| Priority | Language | Region | Market Size |
|----------|----------|--------|-------------|
| 1 | English | US, UK, AU, CA | Existing base |
| 2 | Japanese | Japan | 125M, high tech adoption |
| 3 | Korean | South Korea | 52M, high tech adoption |
| 4 | German | DE, AT, CH | 100M, strong developer community |
| 5 | French | FR, BE, CA | 300M speakers |
| 6 | Spanish | ES, LATAM | 500M speakers |
| 7 | Portuguese | BR, PT | 220M speakers |
| 8 | Chinese (Simplified) | CN | Complex (requires separate infra) |
| 9 | Hindi | IN | 600M speakers, fastest growing market |
| 10 | Arabic | MENA | 400M speakers, RTL support required |

### Localization Approach

**Tools:**
- i18next (web app) + Flutter's intl package (mobile)
- Crowdin: translation management platform (community translators + paid review)
- Translation memory: reuse approved translations across contexts

**Process:**
1. English strings extracted to JSON locale files
2. Crowdin sends to translator pool (mix of community + professional)
3. Professional reviewer approves (paid per word, ~$0.10-0.15/word)
4. CI check: fails build if any locale is missing a key
5. In-app language switcher (no browser/OS dependency)

**RTL support (Arabic):**
- CSS: `direction: rtl` + `text-align: start`
- Flutter: `Directionality.of(context)` for dynamic RTL layouts
- Test: dedicated RTL testing viewport in CI screenshots

### Cross-Lingual Query Expansion (CLQB)

The CLQB algorithm (already built in fetchium-core) expands queries across languages:

```bash
# User searches in Japanese → Fetchium also searches in English
fetchium search "量子コンピューティング" --clqb

# Returns: Japanese results + English results (translated snippets in Japanese)
```

**CLQB in Phase 6:**
- Enabled by default for non-English queries
- Uses Gemini for translation (zero marginal cost, already in provider chain)
- Result merging: HyperFusion handles cross-language deduplication

---

## Regional Infrastructure

### Deployment regions

| Region | Location | Serves | Compliance |
|--------|----------|--------|------------|
| US-East | Hetzner Ashburn | Americas | SOC 2 |
| EU-Central | Hetzner Frankfurt | Europe | GDPR |
| APAC | Hetzner Singapore | Asia-Pacific | PDPB (India) |

### Data residency

Enterprise customers choose their region at signup. Data never leaves that region:
- User accounts, KB, audit logs: stored in chosen region
- Search queries: routed to region's SearXNG instance
- AI synthesis: uses region-local inference endpoint

**EU data residency:** Required for GDPR. All EU customer data stays in Frankfurt.
**India data residency (M30+):** Required for PDPB compliance. India → Singapore.

### CDN Strategy

- Static assets: Cloudflare (global, free tier until > $1K/mo)
- API responses (cacheable): Cloudflare cache at edge (search results, 60s TTL)
- API responses (private): always origin, never cached at CDN
- Mobile app binaries: distributed via App Store / Play Store CDN

---

## On-Device AI (Phase 6 Preview)

**Goal:** Reduce API latency and enable offline mode by running small models on-device.

**iOS:** Core ML models (mlmodel format, < 50MB)
- Use case: intent classification, query reformulation, KB local search
- Not: full AI synthesis (too large for Phase 6)

**Android:** TensorFlow Lite / ONNX Runtime Mobile
- Same use cases as iOS

**Approach:**
- Export small ONNX models from server-side training
- Quantize to INT8 (4x size reduction)
- Test on iPhone 12 and Pixel 6 (minimum target hardware)

---

## Week-by-Week Timeline

### Month 25-26 (iOS Beta)
- W97: Flutter app shell + navigation structure
- W98: Search + AI modes implemented (wraps API)
- W99: KB browse + annotate
- W100: Monitor inbox
- W101: TestFlight beta (1,000 users)
- W102: iOS App Store submission

### Month 27-28 (Android + Extension)
- W103: Android build (same Flutter codebase)
- W104: Android Google Play internal track
- W105: Chrome extension v1 (popup + search hijack)
- W106: Chrome Web Store submission

### Month 29-30 (i18n Wave 1)
- W107: Japanese + Korean + German localization
- W108: i18n CI checks + Crowdin pipeline
- W109: Firefox extension port
- W110: EU regional infrastructure live

### Month 31-33 (Scale + i18n Wave 2)
- W111-W116: French, Spanish, Portuguese, Hindi localizations
- W115: APAC (Singapore) infrastructure live
- W116: Safari extension (Mac App Store)

### Month 34-36 (Polish + $100M push)
- W117: Arabic (RTL) support
- W118: Chinese Simplified (separate infra for compliance)
- W119: On-device AI (intent classification, offline KB search)
- W120: $100M ARR milestone review — Series B closing

---

## Acquisition Channels at Phase 6

| Channel | Estimated MAU contribution |
|---------|--------------------------|
| App Store organic search | 500K MAU |
| Browser extension Chrome Web Store | 400K MAU |
| API developer network (referrals) | 1.5M MAU |
| Enterprise seat expansion | 800K MAU |
| Direct / brand search | 1.2M MAU |
| International (non-English) | 600K MAU |
| **Total** | **5M MAU** |

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Apple App Store rejection | Review guidelines early; no AI-generated content claims without disclosure |
| Chrome extension policy changes | Build extension with minimal permissions; fallback to bookmarklet |
| i18n translation quality | Professional review layer on all public-facing strings |
| China market complexity | Defer: separate legal entity, ICP license, data residency required |
| Mobile team scaling | Hire experienced Flutter dev by M26; use Codemagic CI to reduce DevOps overhead |
| $100M ARR requires CAC efficiency | PLG motion scales efficiently; enterprise sales > 3x LTV/CAC ratio |
