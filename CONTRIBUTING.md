# Contributing to reticulate

Thank you for your interest in helping build this high-performance voxel meshing
library! We welcome community contributions, whether they are bug fixes,
performance optimizations, or documentation improvements.

Because this project uses the **Business Source License 1.1 (BSL-1.1)** to fund
its development, we have specific legal and technical guidelines that all
contributors must follow.

---

## ⚖️ Legal Framework & Contributor License Agreement (CLA)

This project is free for personal, educational, and open-source use, as well as
for small businesses making under \$100,000/year. However, the maintainer
    retains the right to sell commercial licenses to larger corporate or
    government entities.

To make this possible legally, **all contributors must agree to our Contributor
License Agreement (CLA)** before a Pull Request can be merged.

### What does the CLA mean for you?

* **You retain copyright:** You still own the copyright to the code you write.
* **You grant a license:** You grant the project owner a perpetual, worldwide,
  non-exclusive, no-charge, royalty-free, irrevocable license to use, modify,
  distribute, and **commercially re-license** your contributions.
* **How to sign:** When you open a Pull Request, an automated bot (e.g., CLA
  Assistant) will comment on your PR. You can sign the agreement with a single
  click using your GitHub account.

---

## ⚡ Technical Guidelines & Rust Standards

As a high-performance voxel mesher, code quality, safety, and runtime efficiency
are our highest priorities. Please adhere to the following standards:

### 1. Performance & Benchmarking

* If your PR changes core meshing logic or data structures, you **must include
  benchmark results**.
* We use `criterion` for benchmarking. Run existing benchmarks before and after
  your changes using: ```cargo bench ```
* Please document any regressions or unexpected allocations in your PR
  description. Avoid unnecessary allocations (`Vec`, `Box`, etc.) in the hot
  loops of the meshing pipeline.

### 2. Code Quality & Style Before submitting your PR, ensure your code passes

The standard workspace checks:
* **Formatting:** Format your code using the nightly toolchain style settings:

  ```cargo fmt --all -- --check ```

* **Linting:** Run Clippy to catch common mistakes and idiomatic Rust issues:

  ```cargo clippy --workspace --all-targets -- -D warnings ```

* **Testing:** Ensure all unit and integration tests pass perfectly:

  ```cargo test --workspace ```

### 3. Safety & `unsafe`

* Because this library targets high-performance execution (such as SIMD
  optimizations or direct memory manipulation), `unsafe` code is permitted
  **only when strictly necessary** for performance.
* Any `unsafe` block must be accompanied by a `// SAFETY:` comment clearly
  explaining why the invariants are guaranteed to be upheld.

---

## 🚀 How to Submit a Pull Request

1. **Fork the Repository:** Create a personal fork of the project on GitHub.
2. **Create a Branch:** Keep your branches focused. Use descriptive names like
`feature/simd-greedy-mesh` or `fix/index-buffer-overflow`.
3. **Commit Messages:** Write clear, concise commit messages. Prefer imperative
mood (e.g., "Add SIMD acceleration for naive mesher" instead of "Added SIMD
support").
4. **Open a PR:** Submit your Pull Request against our `main` branch.
5. **Sign the CLA:** Click the link provided by the CLA bot on your PR page to
authorize your contribution.
6. **Code Review:** The maintainer will review your code, run automated tests,
and collaborate with you on any necessary adjustments before merging!
