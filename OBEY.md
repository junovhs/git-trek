
    CODE_WITH_INTENT — Decision Trees (POT-Aligned)
    A) Writing Tree (use in order)

    START: Implement feature/fix

    [STEP 0: GUARDRAILS GATE]
    ├─ Control flow OK? → no recursion; acyclic call graph
    ├─ Loops bounded? → static bound OR counter+hard error
    ├─ Allocations disciplined? → no post-init allocs in hot paths; budgets set
    ├─ Functions tight? → ≤60 SLOC; single responsibility; name ≤3 words
    ├─ Assertions adequate? → ≥2 side-effect-free checks/function (avg); failure → recover/error
    ├─ Scope minimal? → no hidden globals; prefer immutability
    ├─ Returns & params? → every return checked; every input validated
    ├─ Metaprogramming minimal? → no eval/codegen/obfuscating macros; ≤2 flags/component (justified)
    ├─ Indirect calls analyzable? → static dispatch or fixed table set at init
    └─ Tooling wall? → 0 warnings; linters/types/static analysis clean
    IF ANY "NO" → REDESIGN BEFORE CODING

    [STEP 1: CHOOSE APPROACH]
    ├─ Is a simple, obvious solution available?
    │   ├─ YES → Use it
    │   └─ NO  → Write 1 sentence why simple fails
    │           ├─ Weak reason → Use simple anyway
    │           └─ Strong reason → Proceed complex + keep that sentence near code
    └─ Considering something clever? → Justify or drop it

    [STEP 2: STRUCTURE]
    ├─ Can you name each function in ≤3 words, no "and"? 
    │   ├─ YES → keep
    │   └─ NO  → split until true
    ├─ Bounds & budgets visible at the callsite?
    │   ├─ YES
    │   └─ NO  → add constants/guards (loop MAX, mem caps, concurrency caps)
    └─ Add assertions for: pre, post, invariants (avg ≥2/function)

    [STEP 3: ERRORS]
    ├─ Can this operation fail?
    │   ├─ YES → choose path:
    │   │   ├─ Graceful degrade (log+code+context) 
    │   │   └─ Hard fail (clear code+message+next steps)
    │   └─ NO  → continue
    └─ Any silent failure possible? → forbid; make loud or degrade with logs

    [STEP 4: DETERMINISM]
    ├─ Time/random? → inject clock/seed; fix seeds in tests
    ├─ Map/set iteration? → sort keys first
    └─ Concurrency essential?
        ├─ NO → single thread/event loop
        └─ YES → single writer; explicit ordering; timeouts

    [STEP 5: INSTRUMENTATION]
    ├─ Critical path? → add entry/exit + key-decision logs (flag-guarded)
    ├─ Can you debug from logs alone?
    │   ├─ YES
    │   └─ NO  → add structured fields now
    └─ Log format → {ts, level, rid, subsystem, action, code, msg, context}

    [STEP 6: INTERFACES]
    ├─ Must be public?
    │   ├─ YES → document contract; validate inputs; plan compatibility
    │   └─ NO  → keep internal
    └─ Can you expose less? → do it

    B) Pre-Ship Gate (stop on any "NO")

    No recursion; all loops bounded

    No post-init allocs in hot paths; budgets enforced

    Functions ≤60 SLOC; names ≤3 words; single responsibility

    Avg ≥2 assertions/function; failures recover or exit loudly

    Minimal scope; no hidden globals/state

    Every return checked; every param validated

    No eval/obfuscating macros; ≤2 flags/component, justified

    Indirect calls analyzable (static/fixed)

    Build: 0 warnings; linters/type/static analyzers 0 findings

    Logs make failures diagnosable without a debugger

    IF ANY "NO" → FIX, THEN RECHECK.

    C) Delivery Tree (how you send code)
    [DELIVER]
    ├─ Assess Change Scope:
    │   ├─ IF the change is small and self-contained (a single function, struct, or contiguous block)...
    │   │   └─ THEN deliver a SCOPED BLOCK.
    │   └─ ELSE (changes are large, spread across multiple functions, or >50% of the file is new)...
    │       └─ THEN deliver a WHOLE FILE.
    │
    ├─ Format SCOPED BLOCK:
    │   ├─ Must be clearly identified with a header comment specifying the file and scope.
    │   │   └─ Example: // FILE: src-tauri/src/render/pipeline.rs | FUNCTION: create_render_pipeline
    │   └─ Provide the entire function or block, from its signature to its closing brace }.
    │
    ├─ Format WHOLE FILE:
    │   └─ Must be clearly identified with a header comment specifying the full path.
    │   └─ Example: // FILE: src-tauri/src/render/pipeline.rs
    │
    ├─ Provide git commit message:
    │   ├─ WHEN: For any new file, modification, or logical changeset delivered.
    │   ├─ FORMAT: type(scope): subject (Conventional Commit).
    │   └─ RULE: Subject must complete the phrase "If applied, this commit will..."
    │
    └─ Always attach:
        ├─ Diff note: Brief "what & why" of the change.
        └─ Confidence: LOW / MED / HIGH.

    D) Debug Tree (when something breaks)
    [DEBUG]
    ├─ Confidence in root cause?
    │   ├─ HIGH → proceed to fix
    │   └─ <HIGH → gather evidence
    ├─ Gather
    │   ├─ Exact repro steps
    │   ├─ Env details
    │   ├─ Logs (structured)
    │   └─ Smallest failing case + save artifacts/paths
    ├─ Analyze logs
    │   ├─ Clear failure point? → proceed
    │   └─ Not clear → add flag-guarded probes (entry/exit/decisions/fails/state)
    ├─ Fix
    │   ├─ Address root cause (not just symptoms)
    │   ├─ Add defenses/tests to prevent recurrence
    │   └─ State confidence + uncertainties
    └─ Verify
        ├─ Reproduce on old; confirm fixed on new
        └─ Check for new edge failures; document learnings

    E) Language Dials (quick branches)
    [RUST]
    ├─ Deny warnings + clippy -D warnings
    ├─ No unwrap/expect outside tests → use ?
    ├─ Prefer static dispatch; preallocate; no loop heap churn
    ├─ Inject time/random via traits; fixed seeds in tests
    ├─ tracing for structured logs; avoid println! in libs
    └─ Avoid unsafe; if needed, isolate in one audited module

    [TS/JS]
    ├─ TS --strict; ESLint errors-as-errors; no implicit any
    ├─ Ban eval/new Function/with; dynamic import only at init
    ├─ Await all Promises; forbid unhandled rejections; timeouts mandatory
    ├─ Sort keys before iteration; reuse arrays/objects in hot paths
    └─ Node: JSON logs; Browser: build-time flags (no runtime eval)

    [PYTHON]
    ├─ mypy --strict; Ruff/Flake8 as errors; pytest required
    ├─ Ban eval/exec & metaclass tricks in core
    ├─ Typed exceptions; never bare except; include context
    ├─ Inject datetime/random; Random(seed) in tests
    └─ Reuse buffers (bytearray/memoryview) in hot paths; async: timeouts+cancel

    [CSS]
    ├─ Deterministic tokens; no magic values
    ├─ No IDs; depth ≤3; no !important
    ├─ One responsibility per class (BEM-like)
    ├─ Layout via grid/flex; central breakpoint scale
    └─ Size/perf budget; drop unused rules

    F) Mindset Pointers (keep the soul)

    Build simply → justify any cleverness

    Build in pieces → split until one job per unit

    Build to survive → degrade or fail loudly; never silent

    Build to understand → names > abbrevs; stable error codes

    Build to prove → evidence first; state confidence

    Build lean interfaces → public = promise

    Build to adapt → few explicit flags; invisible redundancy

    If it feels complicated → simplify again

    Instruction to AI: Follow the trees top-to-bottom. If any guardrail or pre-ship check fails, stop and redesign—do not request exceptions.
    
    when we lint, we lint with: 
    cargo clippy --all-targets -- -D warnings -D clippy::pedantic