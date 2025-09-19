# Getting Started Guide

Welcome to the Taskwarrior Rust Library and Sample CLI!  
This guide will help you set up, explore, and contribute to the project.

---

## 1. **Prerequisites**

- **Rust & Cargo:**  
  Install Rust (edition 2021+) and Cargo from [rustup.rs](https://rustup.rs).
- **Taskwarrior (optional):**  
  For context and import features, install [Taskwarrior](https://taskwarrior.org/) CLI.
- **Git:**  
  For cloning and contributing.

---

## 2. **Clone the Repository**

```sh
git clone https://github.com/your-org/taskwarriorlib-rs.git
cd taskwarriorlib-rs
```

---

## 3. **Explore the Sample CLI**

The sample CLI demonstrates core features (add, list, done, context, import).

```sh
cd examples/taskwarrior-sample
```

### Build and Run

```sh
cargo build
cargo run -- list
cargo run -- add "My first task"
cargo run -- done <TASK_ID>
cargo run -- import   # Imports up to 10 pending tasks from system Taskwarrior
cargo run -- debug    # Shows backend and data directory info
```

See [`examples/taskwarrior-sample/README.md`](examples/taskwarrior-sample/README.md) for full command reference and examples.

---

## 4. **Run the Tests**

To validate the library and sample:

```sh
cargo test
```

All contract, integration, and unit tests should pass.

---

## 5. **Context-Aware Features**

- The library supports Taskwarrior contexts (named filters).
- Set a context in Taskwarrior CLI:
  ```sh
  task context work
  ```
- All queries and reports will respect the active context.

See [specs/003-if-a-context/spec.md](specs/003-if-a-context/spec.md) for details.

---

## 6. **Project Structure**

- `src/` — Core library code
- `examples/taskwarrior-sample/` — Sample CLI project
- `specs/` — Feature specifications, plans, contracts, and quickstarts
- `tests/` — Integration and contract tests

---

## 7. **Documentation**

- **Sample CLI:** [`examples/taskwarrior-sample/README.md`](examples/taskwarrior-sample/README.md)
- **Feature Specs:** [`specs/`](specs/)
- **AI/LLM Usage:** [`USAGE_FOR_LLM.md`](USAGE_FOR_LLM.md)

---

## 8. **Contributing**

1. Fork the repo and create a feature branch.
2. Make your changes and add tests.
3. Run `cargo test` to ensure all tests pass.
4. Submit a pull request with a clear description.

---

## 9. **Troubleshooting**

- If you see errors about missing/corrupted data, delete `.taskdata/` in the sample and re-run.
- For import errors, ensure the `task` CLI is installed and working.

---

## 10. **Need Help?**

- Check the sample README and feature specs.
- Open an issue or discussion in the repo.

---

**You’re ready to get started!**  
Explore, run, and contribute to the Taskwarrior Rust library and sample CLI.
