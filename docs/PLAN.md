# Noesis → WolframAlpha parity: the full plan

This is the working plan for taking Noesis from "covers the core of WolframAlpha's mathematics" to
"answers what WolframAlpha answers", across every topic WolframAlpha lists publicly and every product
feature around it. Any technique is allowed: own algorithms, ported open rule sets, open datasets,
live services, GPU compute and a local language model.

## 0. What "foolproof" means, and what it does not

No plan can make parity true by saying so. WolframAlpha has been curated since 2009 on top of
Mathematica, which dates from 1988, and parts of it rest on licensed data nobody can republish. What a plan *can* do is make failure visible
and local, so that work converges instead of drifting. This plan is built on six rules:

1. **Parity is a number measured against WolframAlpha itself**, on questions the implementer did not
   write, split by topic. Nothing else counts as evidence.
2. **Every feature has an oracle that is independent of the code that implements it**: an open CAS, an
   authoritative dataset, a physical identity, or WolframAlpha.
3. **Every answer checks itself at run time** (differentiate the antiderivative, substitute the root,
   plug the solution into the equation) or is labelled unverified. A wrong result is withheld, not shown.
4. **No query dead-ends.** There is a fixed ladder of fallbacks from exact parsing down to an honest
   "not understood, did you mean…".
5. **Data is generated, never typed.** Every value comes from a reproducible pipeline over an open,
   licence-checked source and carries its source and date into the result.
6. **Every WolframAlpha topic has a line in a coverage ledger** with one of three states: done,
   partial, or excluded with a written reason. Nothing can be skipped silently.

Section 8 lists each way the plan could fail and the mechanism that catches it.

## 1. Where Noesis stands today (measured)

* About 38,000 lines of Rust: own CAS, pods, plots, CLI, egui desktop app (Linux and Windows), 600 KB
  of bundled data, optional live data and an optional language-model bridge.
* Side-by-side with the WolframAlpha Full Results API (`tools/benchmark/wa_bench.py`):
  **90/92 = 97.8 %** on the core set, **55/75 = 73.3 %** on the broad set.
* The broad set shows where the gap is. Mathematics mostly agrees. The misses are data and breadth:
  current positions of the Moon and planets, metro populations, land-area densities, live exchange
  rates, anagrams and definitions (no dictionary), general knowledge (authors, dates, office holders),
  full oxidation-state lists, special-function antiderivatives.
* Structural limits, from the README: hand-written grammar; first matching handler wins (no ranking of
  interpretations); heuristic integration (no Risch); factoring over ℚ only; double precision for
  special functions; a few classes of ODE and PDE; steps for a handful of operations; GPU path never
  run on real hardware; a real language model never tested end to end.

## 2. The target, stated so it can be measured

The coverage map is WolframAlpha's own public "Examples by Topic" catalogue: four sections
(Mathematics, Science & Technology, Society & Culture, Everyday Life), about sixty topics and several
hundred subtopics, plus the product features around them (step-by-step, input modes, API, apps).

| Tier | What | Target on a held-out set |
| --- | --- | --- |
| A | Computation (maths, formulas, units, dates) | ≥ 98 % agreement |
| B | Static knowledge (entities and their properties) | ≥ 95 %, compared with a stated tolerance when the data year differs |
| C | Live data (weather, markets, sky, earthquakes) | ≥ 90 % within a declared tolerance, WolframAlpha re-queried within the hour |
| D | Language | "not understood" < 2 %, wrong interpretation < 3 %, paraphrase consistency ≥ 95 % |
| E | Product features | checklist in section 6, each item demonstrated |

Two scores are always published: **strict** (every question WolframAlpha answers) and **adjusted**
(excluding questions where WolframAlpha demonstrably answered a different question, each with a written
reason). Adjusted never replaces strict.

## 3. Build mechanisms, not answers

WolframAlpha's breadth cannot be matched one query at a time. Eight generic mechanisms carry almost
all of it, and each replaces thousands of special cases:

| # | Mechanism | What it unlocks |
| --- | --- | --- |
| 1 | **Ranked interpretations.** Every handler returns scored candidates; the best is shown, the rest become "Assuming … Use … instead". Replaces today's first-match dispatch in `crates/engine/src/engine/mod.rs`. | Ambiguity ("mercury", "pascal", "C"), assumptions UI, fewer misreads |
| 2 | **Entity store + property grammar.** A compact offline store of entities, aliases and typed properties (Wikidata, CC0, as the backbone). Relation phrases come from property labels and aliases, so "who wrote X", "height of X", "X's capital" need no hand-written rule. | Millions of facts; superlatives, counts, filters, temporal and ordinal questions |
| 3 | **Time-series store.** Indicator × place × year, with history plots, rankings, growth rates and comparisons generated from the schema. | Every socioeconomic, demographic, price and climate series |
| 4 | **Formula knowledge base.** Each formula is data: variables with dimensions, the equation, aliases. The CAS solves for any unknown; the unit engine checks dimensions. | Thousands of physics, chemistry, engineering, finance and medical calculators |
| 5 | **Rule compiler.** Declarative rewrite rules with conditions and explanations, compiled to a decision tree. First client: a port of Rubi's integration rules (MIT). | Near-complete integration with human-style steps; reusable for simplification and step-by-step |
| 6 | **Holonomic function evaluator.** A function is its differential equation plus initial values; one arbitrary-precision evaluator with error bounds serves them all. | Hundreds of special functions to any precision, with series, plots and identities |
| 7 | **Schema-driven pods.** An entity's class decides its result page (a mountain gets elevation, prominence, map, comparisons). | A full WolframAlpha-style page for every entity type without per-type code |
| 8 | **Language model as translator only.** A local model turns free English into Noesis's formal query, which is validated, shown as the interpretation, and computed by the engine. It never writes an answer. | Open phrasing, word problems, other languages, without hallucinated results |

### The fallback ladder (rule 4)

1. Exact mathematical syntax (existing parser, plus LaTeX and Wolfram Language function syntax).
2. Grammar rules (existing `nl*.rs`, kept as the fast path).
3. Typed semantic parser over the entity store and formula base.
4. Local language model → formal query → validation → engine.
5. Online encyclopedia summary, labelled as such.
6. "Not understood", with the nearest interpretations offered.

Every tier's output goes through the same verification before display.

## 4. Coverage matrix: every WolframAlpha topic

**Now**: Have = benchmarked and agreeing; Part = some of it; None = absent.
**Phase** refers to section 7. Licences are re-verified by the import step for each source (section 5).
No topic is marked Have yet: none has passed a held-out comparison, which is the only thing that earns it.

### 4.1 Mathematics

| Topic | Now | What closes the gap | Oracle | Phase |
| --- | --- | --- | --- | --- |
| Elementary math | Part | Digit-level step generators (long division, column arithmetic, fraction work), place value, rounding, number lines; word problems through ladder tier 4 | Exact arithmetic; WA | 2, 4 |
| Algebra | Part | Multivariate GCD (Brown, Zippel) and factorization (Zassenhaus/van Hoeij, Wang EEZ); factoring over extensions; Gröbner bases (Buchberger, then F4) for polynomial systems; algebraic numbers as roots with isolating intervals; cylindrical algebraic decomposition for real systems and inequalities; finite fields, permutation groups (Schreier–Sims), small-group tables, quaternions | SymPy, Singular, PARI/GP; substitution | 2 |
| Calculus: integrals | Part | Rubi rule port; Risch for transcendental elementary functions (Hermite, Lazard–Rioboo–Trager, Risch differential equation); Meijer-G method for definite integrals; answers in erf, Ei, Si, Li, elliptic, hypergeometric and polylog functions; tanh-sinh, Gauss–Kronrod and Levin quadrature; multidimensional cubature on GPU | Differentiate-back; Rubi suite (72,000 problems); SymPy | 2 |
| Calculus: limits, series | Part | Gruntz algorithm; Laurent, Puiseux and asymptotic series; convergence tests with reasons | Numeric approach from both sides; SymPy | 2 |
| Calculus: sums, products | Part | Gosper, Zeilberger, Petkovšek; rational summands through polygamma; numeric acceleration (Levin, Richardson) | Numeric partial sums; SymPy | 2 |
| Vector analysis, transforms | Part | grad, div, curl, Laplacian in standard coordinate systems; line, surface and volume integrals; Laplace, Fourier, Z, Mellin, Hankel through tables, residues and Meijer-G | Numeric integration | 2 |
| Differential equations | Part | Kovacic algorithm; recognition of Bessel, Airy, Legendre, hypergeometric equations; Lie symmetries; Riccati, Abel, exact, homogeneous classes; Frobenius series; boundary-value and Sturm–Liouville problems; stiff solvers (BDF, Radau); slope fields and phase portraits | Residual check; Kamke's collection; SymPy `dsolve` | 2 |
| Partial differential equations | Part | General separation of variables; characteristics for quasilinear equations; Fourier and Laplace transform methods; Green's functions; travelling-wave solutions for named nonlinear equations; method of lines and finite elements, on GPU | Residual check; numeric cross-check | 2 |
| Numbers | Part | Real arbitrary precision with error bounds (extend `bigfloat.rs` to a ball-arithmetic type, binary splitting for constants); integer-relation search (PSLQ, LLL) behind `identify_constant`; minimal polynomials; Egyptian fractions | mpmath, Arb | 2 |
| Trigonometry | Part | Identity proving by normal forms (complex exponential, half-angle) and Fu's algorithm; exact values at rational multiples of π; spherical trigonometry | Random-point equality | 2 |
| Linear algebra | Part | Fraction-free exact elimination; Cholesky, Schur, Hessenberg, pseudoinverse, matrix functions, norms, condition numbers, Gram–Schmidt, bases of subspaces, minimal polynomial, Smith and Hermite forms; numeric kernels of LAPACK quality with GPU for large matrices | Reconstruction (A = QR …); NumPy | 2 |
| Number theory | Part | ECM and quadratic sieve; primality proofs (BPSW plus APR-CL or ECPP); π(x) by Meissel–Lehmer; quadratic forms, sums of squares, Dirichlet characters, Jacobi symbols; sequence recognition (Berlekamp–Massey, polynomial and holonomic guessing) | PARI/GP | 2 |
| Discrete mathematics | Part | Graph package: named graphs, colouring, paths, flows, planarity, isomorphism (VF2), spanning trees, drawing; permutations, posets, lattices, set partitions, Young tableaux; combinatorial identities | NetworkX, Sage | 2 |
| Complex analysis | Part | Domain colouring and Riemann-surface plots, branch cuts, Laurent series, conformal maps, argument principle | Numeric contour integrals | 2 |
| Applied mathematics | Part | Exact simplex, branch and bound, constrained nonlinear and global optimization (KKT with Gröbner/CAD for polynomials, interval branch-and-bound otherwise); interpolation and splines; nonlinear fits (Levenberg–Marquardt); matrix games; dynamical systems (fixed points, stability, bifurcation diagrams, Lyapunov exponents); L-systems, IFS and Newton fractals; cellular automata | SciPy; exact certificates | 2 |
| Logic and set theory | Part | Minimal forms (Quine–McCluskey, Espresso), BDDs, circuit diagrams, Venn diagrams, set algebra on intervals and finite sets, cardinal arithmetic | Truth-table equivalence | 2 |
| Statistics | Part | One distribution framework (symbolic pdf, moments by the CAS, cdf and quantiles through incomplete gamma and beta) for about a hundred distributions; multiple regression, ANOVA, nonparametric and normality tests; box, probability and residual plots | R, SciPy | 2 |
| Probability | Part | Exact enumeration with rationals; poker, bridge, blackjack, lotteries; streaks; Markov chains; Bayes problems; random walks | Monte Carlo on GPU; exact counts | 2 |
| Mathematical functions | Part | Mechanism 6 for the full families (Bessel, Legendre, orthogonal polynomials, elliptic and theta, incomplete gamma and beta, polylog, Hurwitz and Lerch, 1F1, pFq, Meijer G, Mathieu, parabolic cylinder); property pods (series, equation satisfied, special values, plots); injectivity, periodicity, parity, continuity | mpmath, Arb to 30+ digits | 2 |
| Geometry | Part | Shape library with formulas derived by the CAS at build time; polyhedra (Platonic, Archimedean, Johnson) with 3-D rendering; coordinate geometry, transformations, triangle centers, named curves and surfaces, tilings, packings | Numeric integration of each formula | 2 |
| Plotting | Part | Inequality regions, vector and slope fields, stream plots, 3-D parametric and implicit surfaces (marching cubes), density and complex plots, log axes, data charts; interval-arithmetic sampling for robust implicit plots; parameter sliders | Image-level regression tests | 2, 5 |
| Definitions, famous problems, continued fractions | Part | Glossary from Wikidata descriptions and Wikipedia summaries; curated status table for theorems and open problems; generalized continued fractions | Source cross-check | 1 |
| Step-by-step, all of the above | Part | Pedagogical solver on mechanism 5: every rule has a human explanation; method choice; hints one step at a time; each displayed step is checked equivalent to the previous one | Step equivalence check | 5 |

### 4.2 Science and technology

| Topic | Now | What closes the gap | Data (licence) | Oracle | Phase |
| --- | --- | --- | --- | --- | --- |
| Physics | Part | Mechanism 4 with 1,500+ formulas across mechanics, waves, thermodynamics, electromagnetism, optics, relativity, quantum, nuclear, particle, fluids, astrophysics | CODATA 2022 constants; PDG particle data (CC BY 4.0); NUBASE/AME nuclides (open) | Dimensional analysis; textbook values (OpenStax, CC BY) | 3 |
| Chemistry | Part | Compound store with structure diagrams (stored 2-D coordinates), cheminformatics from SMILES; stoichiometry, limiting reagent, yields; reaction thermodynamics and equilibrium (ICE tables through the solver); redox balancing; kinetics; gas laws; buffers, titration curves, Ksp; electrochemistry; orbitals; decay chains | PubChem (public), Wikidata (CC0), NASA/Burcat thermochemistry, NUBASE | RDKit; CoolProp | 1, 3 |
| Units and measures | Part | 2,000+ units and unit systems; "comparable to" pods from a table of reference magnitudes; dimensional analysis queries; industrial scales (wire gauge, paper, hardness, sizes) | QUDT (CC BY 4.0), Wikidata | Round-trip conversions | 1 |
| Engineering | None | Circuits (impedance, filters, resonance); control (poles, step response, Bode, Nyquist, root locus); signals (FFT, filter design); beams (singularity functions integrated by the CAS); fluid flow (Reynolds, Darcy–Weisbach, Colebrook, Manning); steam tables and refrigerants; orbital mechanics; acoustics | IAPWS-IF97 formulation; CoolProp equations (MIT) | CoolProp; SciPy signal | 3 |
| Computational sciences | Part | Cellular automata, Turing machines, substitution systems; asymptotic comparison of functions; check digits and error-correcting codes; more hashes; IEEE 754 and two's-complement views; image processing on uploads; Wolfram Language function syntax as input | none | Reference implementations | 2, 5 |
| Earth sciences | Part | WGS84 geodesics (Karney), bearings, projections, UTM; magnetic declination (WMM); full standard atmosphere; solar position; seasons; tides (station harmonics); earthquake catalogue and live feed; climate normals; geologic time; seawater properties | WMM and EGM96 (public), USGS (public domain), IBTrACS (NOAA), TEOS-10 | NOAA calculators; USGS | 1, 3 |
| Materials | None | Property tables for metals, alloys, minerals, woods, plastics, semiconductors, with comparisons | Wikidata, USDA Wood Handbook (public domain), public handbooks | Cross-source | 1 |
| Transportation | None | Airports, airlines, aircraft, bridges, tunnels, ships, railways, vehicles and fuel economy; flight time estimates; live flights and driving routes as optional online adapters | OurAirports (public domain), EPA fuel economy (public domain), Wikidata | Cross-source | 1 |
| Astronomy | Part | Ephemeris for Sun, Moon and planets; rise, set, transit, phases, eclipses, conjunctions; star charts for any place and time; stars, constellations, deep-sky objects, exoplanets, minor planets, comets; satellites by SGP4; cosmology calculator; time scales | JPL DE440 excerpt (public), VSOP87, HYG (CC BY-SA), OpenNGC (CC BY-SA), NASA Exoplanet Archive, MPC, CelesTrak | JPL Horizons | 1 |
| Life sciences | None | Species taxonomy and traits; dinosaurs; genes and proteins; sequence tools (GC content, translation, alignment by Needleman–Wunsch and Smith–Waterman); codon tables, Punnett squares, Hardy–Weinberg; population and epidemic models through the ODE engine; anatomy ontology | NCBI Taxonomy (public domain), GBIF (CC BY), UniProt (CC BY), Paleobiology DB (CC BY), Reactome | Biopython | 1, 3 |
| Technological world | None | IP and subnet calculators; Unicode character data; barcodes and QR codes; photography calculators; file-transfer and display maths; power plants; structures; inventions; web lookups as online adapters | Unicode UCD, IANA registries, WRI power plants (CC BY 4.0), Wikidata | Reference implementations | 1, 3 |
| Food science and nutrition | Part | Full nutrition label for any food and amount, recipe arithmetic, daily values, per-food cooking conversions, comparisons | USDA FoodData Central (CC0) | USDA values | 1 |
| Health and medicine | Part | About a hundred published clinical and fitness formulas (BMR, body fat, eGFR, dosing by weight, blood alcohol, due dates, heart-rate zones, activity energy); growth and lab percentiles by age and sex (LMS method); life tables; diagnosis codes; drug label facts; always with a medical disclaimer | CDC growth charts and NHANES (public domain), ICD-10-CM (public domain), openFDA (CC0), SSA life tables | Published worked examples | 1, 3 |
| Weather and meteorology | Part | History for any date since 1940, climate normals, hurricanes, tornadoes, METAR, air quality, space weather; derived indices (heat index, wind chill, dew point) | Open-Meteo (CC BY 4.0), IBTrACS, NOAA SWPC, OpenAQ | Source values | 1 |

### 4.3 Society and culture

| Topic | Now | What closes the gap | Data (licence) | Phase |
| --- | --- | --- | --- | --- |
| People | Part | Mechanism 2 for notable people: dates, places, offices with terms, works, awards, relations; ages and timelines computed; name popularity and estimated age of bearers; kinship calculator with tree diagrams; wages by occupation | Wikidata (CC0), SSA names (public domain), Census surnames, BLS (public domain) | 1 |
| Arts and media | None | Films, television, books, music, artworks, games, fictional characters, awards; music theory (intervals, chords, scales, staff rendering, tone playback); statistics of public-domain texts | Wikidata, MusicBrainz (CC0), Open Library, Project Gutenberg | 1, 3 |
| Dates and times | Part | Full IANA time zones; calendar systems (Hebrew, Islamic, Chinese, Persian, Indian, Mayan and others, reimplemented from the published arithmetic); holidays for about 200 countries; business-day arithmetic; on-this-day events; Moon phase on any date | IANA tzdb (public domain), Wikidata | 1, 2 |
| Words and linguistics | Part | Definitions, synonyms, antonyms; pronunciation, rhymes, syllables; etymology; word frequency over time; anagrams, patterns, word ladders; languages and families; single-word translation; number names in 50+ languages; ciphers and encodings; document statistics and readability | WordNet (permissive), CMUdict (BSD), ENABLE (public domain), Google Ngrams (CC BY 3.0), CLDR; Wiktionary (CC BY-SA) as a separate pack | 1 |
| Money and finance | Part | Generic time-value solver for any variable; amortization schedules; bonds (price, yield, duration); NPV, IRR, depreciation; options (Black–Scholes, binomial, Greeks, implied volatility); inflation and purchasing power since 1913; tax tables; company fundamentals; quotes and history through pluggable providers | BLS CPI, SEC EDGAR, Treasury (all public domain), ECB rates, World Bank | 1, 3 |
| Places and geography | Part | Countries with about 150 properties and histories; all cities above 500 people plus urban agglomerations; US states, counties, places and ZIP areas; mountains, rivers, lakes, islands, deserts; locator maps, choropleths and size-overlay comparisons; geocoding as an online adapter | Wikidata, GeoNames (CC BY 4.0), UN WPP and WUP (CC BY 3.0 IGO), World Bank WDI (CC BY 4.0), World Factbook (public domain, dated), US Census (public domain), Natural Earth (public domain) | 1 |
| History | None | Events with dates, places and participants; periods and former states; timelines as a pod type; historical numeral systems; long-run prices and populations | Wikidata, Maddison Project (CC BY 4.0), Bank of England and BLS long series | 1 |
| Socioeconomic data | Part | Mechanism 3 over about 1,500 indicators: economy, demographics, health, education, energy, emissions, agriculture, crime | World Bank WDI, UN, Our World in Data (CC BY 4.0), FBI and Census (public domain) | 1 |
| Education | None | Universities (enrolment, tuition, admissions), school districts, score-percentile tables, grade calculators | IPEDS and NCES (public domain), ROR (CC0) | 1 |
| Government and politics | None | Heads of state and government over time, legislatures, election results, public debt | Wikidata, congress-legislators (public domain), MIT Election Lab, Treasury | 1 |
| Sports and games | None | Olympic medals, international football results, baseball history, records and venues; game solvers and exact odds (poker, blackjack, lotteries, sudoku, Nim); pace, rating and scoring calculators; live scores as an optional adapter | Wikidata, football results (CC0), Lahman (CC BY-SA, separate pack) | 1, 2 |

### 4.4 Everyday life

| Topic | Now | What closes the gap | Phase |
| --- | --- | --- | --- |
| Personal health, personal finance | Part | Covered by Health and Money above | 1, 3 |
| Household maths and science | Part | Paint, flooring, concrete, mulch, pool volume; appliance running cost; trip fuel cost; safe cooking temperatures; all as formula-base entries | 3 |
| Hobbies | None | Photography, astronomy, music tuning, cycling gears, running pace, aquarium volume, hardiness zones, knot invariants | 3 |
| Travel | Part | Flight time, time zones, currency, weather, airports, regional price levels (BEA, public domain), plug types and driving side | 1 |
| Today's world | Part | Earthquakes, weather, space weather, exchange rates, satellites overhead, public debt, all through live adapters | 1 |
| Surprises | None | A few hundred self-knowledge and humour entries in Noesis's own words | 4 |

### 4.5 Deliberately excluded or substituted

These rest on data that is licensed, not open. The ledger records each with its substitute:

| WolframAlpha feature | Why it cannot be copied | Substitute |
| --- | --- | --- |
| Real-time and intraday market data | Licensed feeds | End-of-day and delayed data through a provider key the user supplies; fundamentals from SEC filings |
| Cost-of-living index | Proprietary (C2ER) | BEA regional price parities; World Bank price levels |
| Live flight tracking, driving directions | Commercial or usage-limited services | Optional online adapters; great-circle results offline |
| Film ratings, charts, box office dailies, TV ratings | Proprietary | Wikidata facts only |
| Detailed league statistics for every sport | Mostly proprietary | Open datasets where they exist, Wikidata otherwise |
| Drug interaction checking | Licensed and liability-bearing | Label facts from openFDA, with a disclaimer |
| MathWorld text, "popular curves", running arbitrary Wolfram Language | Wolfram's own content and product | Own glossary; Wolfram-style function syntax for mathematics only |

## 5. The knowledge base

**Store.** A new `noesis-data` crate: entity table, alias index as a finite-state transducer, typed
property columns, time-series blocks, all memory-mapped. Queries go through one API used by the
semantic parser and by pod generation.

**Packs.** Offline first, in three sizes (targets, to be confirmed by measurement):

| Pack | Contents | Size target | Delivery |
| --- | --- | --- | --- |
| Core | Today's data plus constants, elements, countries, 26,000 cities, common words, truncated ephemeris | ≤ 15 MB | Embedded in the binary |
| Standard | About 500,000 notable entities, WDI series, foods, compounds, dictionary, DE440 excerpt, maps | about 400 MB | Downloaded once, checksum-verified |
| Add-ons | Share-alike sources, one pack each: Wiktionary extract, star and deep-sky catalogues, baseball history | tens of MB each | Optional |
| Full | Several million entities, all GeoNames places, US Census detail, large catalogues | several GB | Optional |

Share-alike sources (Wiktionary, OpenNGC, HYG, Lahman) ship as separate add-on packs so their terms
never attach to the MIT-licensed engine or the other packs. Data under non-commercial licences is not
used.

**Pipeline.** `tools/datagen/` replaces `tools/build_data.py`: one importer per source, each recording
URL, retrieval date, licence text and checksum in a manifest. The manifest drives the "Sources" line
under every data pod and the `NOTICE` file. Importers validate ranges and units and compare
overlapping sources (for example population from the UN, the World Bank and Wikidata); disagreements
beyond a tolerance fail the build until resolved by an explicit precedence rule.

**Precedence.** Authoritative source for its field first (UN for population, NIST for constants, USDA
for foods, JPL for ephemerides), Wikidata as the general fallback, and every value keeps its year.

**Live adapters.** One trait, one file per service, each with a cache, a timeout and an offline
fallback that says how old its number is: weather and history (Open-Meteo), exchange rates (ECB),
earthquakes (USGS), satellites (CelesTrak plus local SGP4), space weather (NOAA), METAR, air quality,
public debt, encyclopedia; quotes, crypto, flights and routing only with a user-supplied key. Each
adapter's endpoint is configurable, because some free endpoints (Open-Meteo's, for one) are offered
for non-commercial use only.

**WolframAlpha is never a data source.** Its API is used only to compare answers. Nothing it returns
is stored in the repository or copied into a pack.

## 6. Product features

| Feature | Now | Plan | Phase |
| --- | --- | --- | --- |
| Step-by-step solutions | Part | All operation classes, method choice, hints, checked steps | 5 |
| Problem generator and quizzes | None | Parameterised templates per topic and level; answers checked by CAS equivalence | 5 |
| Assumptions and alternate interpretations | None | Mechanism 1, clickable in GUI and listed in CLI/JSON | 0 |
| Mathematical input | Part | LaTeX parser, Wolfram-style function syntax, 2-D template editor, symbol palette | 5 |
| Image input | None | Photo of a problem → local OCR model or vision model through the bridge → formal query shown for confirmation; plain image analysis (size, histogram, colours, EXIF, filters) | 5 |
| File and data input | None | CSV, TSV, XLSX, JSON, text, audio, GPX, molecule files: automatic summary, plots, fits | 5 |
| Sessions and notebook | None | Variables and results carried between queries; saved notebooks | 5 |
| Pod controls | Part | More digits, more terms, units toggle, exact/approximate switch, per-pod copy as text, LaTeX or Wolfram form | 5 |
| Interactive results | Part | Sliders for free parameters, plot tracing, rotatable 3-D for all surface types | 5 |
| Export | Part | PNG, SVG, CSV per pod; PDF of the whole result | 5 |
| Sources and provenance | None | Source and date under every data pod (from the manifest) | 1 |
| API | Part | `noesis serve`: local HTTP API with full-result JSON, short answer, spoken text, and a compact form for language-model tools (MCP server) | 5 |
| Web and mobile | None | Engine compiled to WebAssembly with a web front end; the same build wrapped for mobile | 5 |
| Localization and personal defaults | None | Location, units, currency and time zone from settings; entity names in other languages from Wikidata labels; interface translations | 4, 5 |
| Accessibility | Part | Keyboard paths, spoken forms of results, contrast themes, font scaling | 5 |
| Performance | Part | Parallel candidate handlers, streamed pods, cancellation, caches; budgets enforced in CI | 0 |
| GPU | Part | Compute shaders for plots, Monte Carlo, large linear algebra, PDE stepping and image work; CPU and GPU results compared by test; `noesis --selftest` for real hardware | 5 |
| Installers and updates | None | MSI, AppImage, deb; pack manager with checksums | 5 |

## 7. Phases and exit gates

Each phase ends with a fresh held-out benchmark for its topics. A phase is not finished until its gate
passes, CI is green on Linux and Windows, and the size and latency budgets hold. Sizes are relative:
S about one working session, M several, L dozens, XL a long-running programme.

| Phase | Work | Size | Exit gate |
| --- | --- | --- | --- |
| **0 Foundations** | Ranked interpretations; `noesis-data` store and pack format; `tools/datagen` skeleton with manifest; run-time verification module; open-oracle harness; coverage ledger; benchmark upgrades (section 8); the known mismatches (section 10) | M | Core set ≥ 98 %, broad set ≥ 85 % strict; ledger lists every topic |
| **1 Knowledge** | Wikidata pack and property grammar; authoritative overrides; time-series store; ephemeris and sky; dictionary; foods; compounds; maps; live adapters; sources under every pod | L | Held-out Society, Everyday and data-backed Science topics ≥ 85 %, then ≥ 95 % |
| **2 Kernel** | Ball arithmetic and holonomic evaluator; polynomial algebra, Gröbner, CAD; rule compiler and Rubi port; Risch; Meijer-G; Gruntz; summation algorithms; ODE and PDE methods; simplifier with assumptions; number theory, graphs, optimization, statistics, logic, geometry, plot types | XL | Held-out Mathematics ≥ 97 %; Rubi suite ≥ 95 % solved, 100 % of shown answers verified; Kamke ≥ 85 % |
| **3 Calculators** | Formula base to 1,500+ entries; physics, chemistry, engineering, earth-science, medical and finance calculators; thermodynamic property models | L | Held-out Science & Technology ≥ 92 %; every formula passes dimensional and worked-example checks |
| **4 Language** | Semantic parser with entity linking and popularity priors; bundled optional local model with constrained output; paraphrase suite; did-you-mean; other languages; word problems | L | Tier D targets of section 2 |
| **5 Product** | Section 6 items | L | Checklist complete; CPU ≡ GPU suite passes on at least one real GPU per vendor |
| **6 Upkeep** | Monthly data refresh, fresh held-out benchmark, fix the largest failure clusters, track new WolframAlpha topics | ongoing | Scores do not regress; ledger stays complete |

Phases 1 and 2 are independent and can interleave. Phase 4's ranking work starts in phase 0 because
everything else depends on it. Order inside a phase is always "largest failing cluster in the latest
benchmark first".

## 8. Verification: how each failure is caught

| How the plan could fail | What catches it |
| --- | --- |
| A wrong symbolic answer is shown | Run-time verification for every result class (derivative of antiderivative, residual of equation, ODE and PDE, partial sums, two-sided numeric limits, matrix reconstruction, random-point equality before any simplification is accepted); failing results fall back to numeric with a caveat |
| Tests that only confirm the implementer's own expectations | Questions come from third parties: open textbook problem sets, the Rubi and Kamke collections, test suites of open CAS, blind paraphrases; plus the WolframAlpha comparison |
| Overfitting to a fixed benchmark | Dev set (visible) and held-out set (only per-topic totals are read); the held-out set is replaced each cycle and then joins the dev set |
| A topic silently skipped | Coverage ledger checked in CI: every topic is done, partial or excluded with a reason |
| Wrong or stale data | Generated data only; range and unit validators; cross-source agreement; year shown with every value; monthly refresh job |
| A licence problem | Manifest gate: no source without a recorded licence; share-alike sources isolated; non-commercial sources refused; `NOTICE` generated |
| The language model invents an answer | It can only emit a formal query, which is validated against a schema, displayed as the interpretation and computed by the engine |
| Misread questions that still produce an answer | WolframAlpha's own "input interpretation" is compared with Noesis's interpretation, scored separately from the value; paraphrase consistency tests; must-not-match negative examples |
| Regressions | Every answer that has ever matched is pinned; CI fails when a pinned answer changes; Linux and Windows on every push |
| Slowness or bloat as data grows | Budgets in CI: p95 latency per query class, memory, binary and pack sizes |
| Hardware paths nobody ran | CPU ≡ GPU equivalence suite and a user-runnable self-test; the README keeps listing a path as untested until a real run is recorded |
| Dependence on the WolframAlpha API (quota, terms) | Open oracles are the daily check; WolframAlpha is a sampled audit sized to the monthly quota |
| Scope that never ends | Generic mechanisms first; gates with numbers; exclusions written down |

**Benchmark upgrades** to `tools/benchmark/wa_bench.py`: per-topic stratification and scores with
confidence intervals; dev and held-out files; fitting of integration constants when comparing general
solutions of differential equations; antiderivatives compared by differentiation; annotations for
time-varying questions (re-fetch within the hour, stated tolerance) and for data-year differences;
WolframAlpha error outputs counted as its failure; a `wa-misread` class that needs a written reason
and is reported separately; a history file so every published score can be traced.

**Open-oracle harness** `tools/oracle/`: generators for random problems per class, run nightly against
SymPy, mpmath, PARI/GP, SciPy, RDKit and JPL Horizons. These are development tools only and are not
shipped.

## 9. Decisions for the owner

Work proceeds on the stated defaults unless changed:

1. **Data delivery.** Default: small embedded core, larger packs downloaded on demand from GitHub
   Releases.
2. **Share-alike data.** Default: allowed, but only as separate optional packs.
3. **Language model.** Default: optional download of a small local model; never required for
   mathematics; any OpenAI-compatible endpoint still works.
4. **WolframAlpha API.** The comparison uses your AppID. Please confirm that benchmarking is within
   the API terms for your account. The account's monthly call limit sets the sample size: with 2,000
   calls, about 1,500 new questions a month plus re-queries of time-varying ones.
5. **Keyed online services** (quotes, crypto, flights, routing). Default: off until a key is entered
   in Settings.
6. **Development-only Python tooling** under `tools/` for the oracles. Default: yes.

## 10. First work block

Already identified by the current benchmark, in order:

1. Ephemeris for the Moon and planets (truncated series in core): current distances, phases; use it
   in "distance from earth to mars" and in gravitational-force questions, showing mean and current.
2. Data corrections found by comparison: Moon's mass from GM/G; Tokyo as metropolis and as urban
   agglomeration; population density on land area.
3. Word list and dictionary: anagrams, patterns, definitions.
4. First slice of mechanism 2: authors of works, start and end dates of events, ordinal office
   holders, from a Wikidata extract.
5. Complete oxidation states; taxicab numbers; nth digit of a constant.
6. Antiderivatives in hypergeometric and elliptic functions for binomial integrands.
7. Benchmark upgrades from section 8, then a third, held-out question set of about 150 questions
   drawn from third-party sources, and the first per-topic score table in the README.
