# state-tutorial
Component state diagram

## Overview

This project implements an **ESSOR/SCA-inspired component lifecycle state machine** in Rust.

The component exposes a simplified version of the [Software Communications Architecture (SCA)](https://en.wikipedia.org/wiki/Software_Communications_Architecture) / ESSOR interface, consisting of six methods:

| Method | Description |
|--------|-------------|
| `config(params)` | Configure the component (drives Inactive→Loaded and Loaded→Ready) |
| `start()` | Activate the component (Ready→Running) |
| `stop()` | Deactivate the component (Running→Ready) |
| `reset()` | Hard-reset to initial state (any→Inactive) |
| `query()` | Read-only status query (no state change) |
| `test()` | Built-in self-test (valid in Loaded or Ready) |

---

## State Machine

### States

| State | Description |
|-------|-------------|
| `INACTIVE` | Component exists but has not been loaded or configured |
| `LOADED` | Component has been loaded with initial configuration |
| `READY` | Component is fully configured and ready to start |
| `RUNNING` | Component is actively processing |
| `ERROR` | Component encountered an error; must be reset before reuse |

### Transition Table

| Current State | Method     | Next State | Notes |
|---------------|------------|------------|-------|
| `INACTIVE`    | `config()` | `LOADED`   | Load & initial configuration |
| `LOADED`      | `config()` | `READY`    | Final configuration |
| `READY`       | `start()`  | `RUNNING`  | Activate the component |
| `RUNNING`     | `stop()`   | `READY`    | Deactivate the component |
| any           | `reset()`  | `INACTIVE` | Hard reset to initial state |
| `LOADED`      | `test()`   | `LOADED`   | Self-test (state unchanged on success) |
| `READY`       | `test()`   | `READY`    | Self-test (state unchanged on success) |
| any           | `query()`  | —          | Read-only, no state change |
| any (failure) | *any*      | `ERROR`    | Transition on internal error |

### State Diagram

```
                         ┌───────────────────────────────────────────┐
                         │              reset()                      │
                         │         ┌──────────────────────────────┐  │
                         │         │           reset()            │  │
                         │         │    ┌─────────────────────┐   │  │
                         ▼         │    │       reset()        │   │  │
                   ┌──────────┐    │    │  ┌───────────────┐   │   │  │
            ──────►│ INACTIVE │    │    │  │               │   │   │  │
                   └──────────┘    │    │  │   reset()     │   │   │  │
                         │         │    │  │    ┌──────────┴───┴───┴──┴─┐
                    config()       │    │  │    │        ERROR          │
                         │         │    │  │    └───────────────────────┘
                         ▼         │    │  │              ▲
                   ┌──────────┐    │    │  │          (failure)
                   │  LOADED  │────┘    │  │
                   └──────────┘         │  │
                         │              │  │
                    config()            │  │
                         │              │  │
                         ▼              │  │
                   ┌──────────┐─────────┘  │
                   │  READY   │────────────┘
                   └──────────┘
                         │         ▲
                    start()     stop()
                         │         │
                         ▼         │
                   ┌──────────┐────┘
                   │ RUNNING  │
                   └──────────┘
                         │
                      reset()
                         │
                         ▼
                   ┌──────────┐
                   │ INACTIVE │  (see top)
                   └──────────┘
```

---

## Usage

```bash
# Build
cargo build

# Run demo
cargo run

# Run tests
cargo test
```

### Example output

```
=== ESSOR/SCA Component State Machine Demo ===

[query]  Component 'radio-component' is inactive.
[config] Component 'radio-component' is loaded with 1 configuration entries.
[test]   passed=true, details=Self-test passed: 1 configuration entries verified.
[config] Component 'radio-component' is fully configured and ready to start.
[test]   passed=true, details=Self-test passed: 3 configuration entries verified.
[start]  Component 'radio-component' is running.
[stop]   Component 'radio-component' is fully configured and ready to start.
[reset]  Component 'radio-component' is inactive.

Demo complete.
```

