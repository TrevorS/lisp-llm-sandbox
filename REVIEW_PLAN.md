# Comprehensive Code Review and Cleanup Plan

## Objective
Perform a thorough review to find and fix:
- Outdated or incorrect documentation
- Incorrect or misleading comments
- Dead code or unused functions
- Opportunities for simplification
- Inconsistencies after the Arc refactoring
- Test counts and feature descriptions that need updating

## Codebase Structure (30 source files + 6 stdlib modules + 5 test suites + 9 examples)

### Phase 1: Documentation Review (PARALLEL - 6 agents)
**Goal**: Update all docs to reflect current implementation (spawn, spawn-link, stdlib modules)

1. **Agent 1: CLAUDE.md**
   - Update test counts (currently claims 281, need to verify actual count)
   - Update builtin count (currently 38, verify with spawn/spawn-link additions)
   - Remove outdated "V1 Limitations" section (spawn is implemented!)
   - Update concurrency section with spawn, spawn-link, and stdlib helpers
   - Update stdlib module count (now 6 modules: core, math, string, test, http, concurrency)

2. **Agent 2: README.md**
   - Check if concurrency examples are current
   - Verify feature list matches implementation
   - Update quick start if needed

3. **Agent 3: CONCURRENCY_V2_DESIGN.md**
   - Mark completed features (spawn, spawn-link, channels)
   - Mark incomplete/future features (atoms, timeout, resource limits)
   - Consider archiving or moving to /docs if outdated

4. **Agent 4: SPAWN_POC.md**
   - This is historical POC documentation
   - Should be archived or removed (implementation is complete)

5. **Agent 5: CONCURRENCY_ROADMAP.md**
   - Update roadmap to show what's completed
   - Mark spawn, spawn-link, stdlib helpers as DONE
   - Clarify what's next (if anything)

6. **Agent 6: V2_MIGRATION_STATUS.md**
   - Verify if this is still relevant post-rebase
   - Update or remove based on current status

### Phase 2: Core System Review (SEQUENTIAL - 1 agent with 5 subtasks)
**Goal**: Ensure core evaluator, parser, and type system are clean and consistent
**Why Sequential**: These components are tightly coupled and changes affect each other

**Agent 7: Core System Deep Dive**
- **Task 2.1**: `src/value.rs` - Review Value enum, Display impl, type_name consistency
- **Task 2.2**: `src/error.rs` - Verify all error types are used, check for any remaining Custom() references
- **Task 2.3**: `src/env.rs` - Check for any Rc remnants, verify Arc usage is consistent
- **Task 2.4**: `src/eval.rs` - Review special forms, check for outdated comments about Rc/RefCell
- **Task 2.5**: `src/parser.rs` - Check for unused code, verify error handling

### Phase 3: Builtins Review (PARALLEL - 11 agents)
**Goal**: Each builtin module is self-contained and can be reviewed independently

8. **Agent 8**: `src/builtins/arithmetic.rs` - Check for simplifications, verify error messages
9. **Agent 9**: `src/builtins/comparison.rs` - Check for simplifications
10. **Agent 10**: `src/builtins/logic.rs` - Check for simplifications
11. **Agent 11**: `src/builtins/types.rs` - Verify all Value types have predicates
12. **Agent 12**: `src/builtins/lists.rs` - Check for optimizations
13. **Agent 13**: `src/builtins/strings.rs` - Check for consistency
14. **Agent 14**: `src/builtins/maps.rs` - Verify keyword handling is consistent
15. **Agent 15**: `src/builtins/console.rs` - Simple review
16. **Agent 16**: `src/builtins/filesystem.rs` - Verify sandbox integration
17. **Agent 17**: `src/builtins/network.rs` - Verify error handling
18. **Agent 18**: `src/builtins/concurrency.rs` - **CRITICAL** - Verify spawn/spawn-link, check for issues
19. **Agent 19**: `src/builtins/errors.rs` - Check error handling utilities
20. **Agent 20**: `src/builtins/testing.rs` - Verify test assertions
21. **Agent 21**: `src/builtins/help.rs` - Check help system
22. **Agent 22**: `src/builtins/mod.rs` - Verify all modules registered correctly

### Phase 4: Stdlib Review (PARALLEL - 6 agents)
**Goal**: Review pure Lisp standard library for correctness and optimization

23. **Agent 23**: `src/stdlib/lisp/core.lisp` - Review higher-order functions and map helpers
24. **Agent 24**: `src/stdlib/lisp/math.lisp` - Review math functions
25. **Agent 25**: `src/stdlib/lisp/string.lisp` - Review string utilities
26. **Agent 26**: `src/stdlib/lisp/test.lisp` - Review testing framework
27. **Agent 27**: `src/stdlib/lisp/http.lisp` - Review HTTP helpers
28. **Agent 28**: `src/stdlib/lisp/concurrency.lisp` - **NEW** - Verify all helpers work, check docs

### Phase 5: Infrastructure Review (PARALLEL - 6 agents)
**Goal**: Review infrastructure, config, and support code

29. **Agent 29**: `src/main.rs` - Check CLI args, REPL initialization, stdlib loading
30. **Agent 30**: `src/config.rs` - Check for unused constants
31. **Agent 31**: `src/sandbox.rs` - Verify sandbox logic is current
32. **Agent 32**: `src/help.rs` - Review help registry and formatting
33. **Agent 33**: `src/macros.rs` - Review macro system
34. **Agent 34**: `src/highlighter.rs` - Review syntax highlighting
35. **Agent 35**: `src/tools.rs` - Check Tool trait
36. **Agent 36**: `src/stdlib_registry.rs` - Verify registrations
37. **Agent 37**: `src/stdlib/json.rs` - Review JSON builtins

### Phase 6: Test Review (PARALLEL - 5 agents)
**Goal**: Verify tests are comprehensive and check for outdated patterns

38. **Agent 38**: `tests/integration_test.rs` - Review integration tests
39. **Agent 39**: `tests/stdlib_tests.rs` - Review stdlib tests
40. **Agent 40**: `tests/string_tests.rs` - Review string tests (already updated for Arc)
41. **Agent 41**: `tests/concurrency_tests.rs` - Review concurrency tests (29 tests)
42. **Agent 42**: `tests/repl_integration.rs` - Review REPL tests

### Phase 7: Examples Review (PARALLEL - 9 agents)
**Goal**: Ensure all examples run and demonstrate best practices

43. **Agent 43**: `examples/concurrency_examples.lisp` - Verify all patterns work
44. **Agent 44**: `examples/parallel_computation_demo.lisp` - Already tested, verify docs
45. **Agent 45**: `examples/data_processing.lisp` - Check if still relevant
46. **Agent 46**: `examples/factorial.lisp` - Simple verification
47. **Agent 47**: `examples/fibonacci.lisp` - Check for TCO examples
48. **Agent 48**: `examples/functional_programming.lisp` - Verify FP patterns
49. **Agent 49**: `examples/maps_and_json.lisp` - Verify map/JSON usage
50. **Agent 50**: `examples/quicksort.lisp` - Verify sorting example
51. **Agent 51**: `examples/stdlib_demo.lisp` - Update for new stdlib modules

### Phase 8: Final Cleanup (SEQUENTIAL)
**Goal**: Apply all findings and create summary

52. **Agent 52: Cleanup Executor**
   - Collect all findings from agents 1-51
   - Categorize issues by priority
   - Apply fixes in order
   - Run full test suite
   - Generate cleanup report

## Execution Strategy

### Parallel Batches
1. **Batch 1** (6 parallel): Documentation review (Agents 1-6)
2. **Batch 2** (1 sequential): Core system (Agent 7)
3. **Batch 3** (15 parallel): Builtins review (Agents 8-22)
4. **Batch 4** (6 parallel): Stdlib review (Agents 23-28)
5. **Batch 5** (9 parallel): Infrastructure (Agents 29-37)
6. **Batch 6** (5 parallel): Tests (Agents 38-42)
7. **Batch 7** (9 parallel): Examples (Agents 43-51)
8. **Batch 8** (1 sequential): Final cleanup (Agent 52)

## Review Checklist for Each Agent

Each agent should look for:
- [ ] **Comments** - Outdated, incorrect, or misleading
- [ ] **Dead Code** - Unused functions, commented-out code
- [ ] **Simplification** - Overly complex logic that could be simplified
- [ ] **Consistency** - Naming, error handling, documentation style
- [ ] **Arc Migration** - Any remaining Rc, RefCell references
- [ ] **Error Handling** - Use of new error system (runtime_error, type_error, arity_error)
- [ ] **Documentation** - Docstrings match implementation
- [ ] **Tests** - Adequate coverage, no outdated assertions

## Expected Findings

Based on the recent Arc refactoring and concurrency additions:

### High Priority
1. CLAUDE.md "V1 Limitations" section - **OUTDATED** (spawn is implemented)
2. Test counts in docs - Need verification
3. SPAWN_POC.md / CONCURRENCY_V2_DESIGN.md - May be obsolete
4. Any remaining `Rc<>` or `RefCell<>` references in comments

### Medium Priority
5. Builtin/stdlib function counts in docs
6. Example files that may not run with new stdlib structure
7. Comments referring to "V1" or "planned for V2"

### Low Priority
8. Code simplification opportunities
9. Duplicate logic that could be abstracted
10. Performance optimizations

## Success Criteria

- [ ] All documentation reflects current implementation
- [ ] No outdated comments or dead code
- [ ] All tests pass (301+ tests)
- [ ] All examples run successfully
- [ ] Consistent error handling throughout
- [ ] Clean git status with no unnecessary changes
