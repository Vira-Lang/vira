Table of Contents

1. How You Can Contribute


2. Setting Up Your Development Environment


3. Reporting Issues — How to Write a Good Bug Report


4. Working on Code & Submitting Pull Requests


5. Code Style, Tests, and CI


6. Security Policy


7. FAQ & Support




---

1) How You Can Contribute

Vira is a collaborative effort — there’s always room to help:

Fix a bug or unexpected behavior.

Improve documentation or write examples/tutorials.

Add or refine tests.

Contribute to the compiler, parser, CLI, or runtime (the project uses Rust, Zig, Go, and JavaScript).

Propose and discuss new language features.


If you’re not sure where to start, look for issues labeled good first issue or help wanted, or open a discussion to share your ideas.


---

2) Setting Up the Environment

Here’s a simple setup workflow — adjust it for your OS and preferred components.

1. Clone the repository:



git clone https://github.com/Vira-Lang/vira.git
cd vira

2. If working with Rust components:



rustup toolchain install stable
rustup default stable
cargo build --workspace
cargo test

3. If working with Zig components: Install the required Zig version (check project docs for specifics), then build using:



zig build

4. If working with JavaScript / Node parts:



cd <your_js_folder>
npm install
npm test

Explore the folders compiler, parser_lexer, cli, examples, and installation to understand the project’s structure and build scripts.

> If something doesn’t work, please include details about your OS, compiler versions, and toolchains in your issue — it helps us reproduce and fix problems quickly.




---

3) Reporting Issues — Be Specific

A clear and detailed issue makes it much easier to solve.

Include:

Title: concise and descriptive (parser: crash on string interpolation).

Description: what happened, what you expected to happen.

Steps to reproduce: minimal reproducible example.

Environment: OS, Rust/Zig/Node versions.

Logs/output: compiler messages, stack traces, or command output.


Template (you can copy this into your issue):

### Description
Brief explanation of the problem.

### Steps to Reproduce
1. ...
2. ...
3. ...

### Expected Behavior
...

### Logs / Output
(Include relevant compiler messages)

### Environment
- OS:
- Rust / Zig / Node version:


---

4) Working on Code & Submitting a Pull Request

1. Fork the repository and create a new branch:



git checkout -b feat/your-feature-name

2. Keep your change focused on a single topic (e.g., parser fix, not parser + CLI refactor).


3. Write tests — unit or integration — to verify your change.


4. Make clear, meaningful commits:



module: short summary

Example:
fix(parser): handle escaped quotes in strings

5. Push to your fork and open a Pull Request (PR):



Explain what your PR does, why, and how to test it.

Link related issues (Fixes #42).

Tag maintainers or reviewers if you know them.


PR Checklist

[ ] References an issue (if applicable)

[ ] Includes or updates tests

[ ] Passes local builds and CI

[ ] Updates docs or examples if needed



---

5) Code Style, Tests, and CI

We value consistency and clarity:

Rust: use rustfmt and clippy.

Zig / JS / Go: follow their official formatters (zig fmt, gofmt, prettier or eslint).

Keep PRs small and well-scoped — large refactors are best split into separate PRs.


> If CI fails, check the logs first. If you can’t reproduce the failure locally, describe your environment and paste the failing output in your PR.




---

6) Security and Responsible Disclosure

If you discover a security issue or vulnerability:

Do not open a public issue.

Instead, contact the maintainers privately via GitHub or email (see repository owner’s profile).

Include detailed reproduction steps and describe potential impact.


We’ll handle the disclosure responsibly and credit your contribution appropriately.


---

7) FAQ & Support

Q: I’m new to Rust/Zig/Go — can I still help?
Absolutely! Documentation, tests, and examples are great starting points. We’re happy to mentor first-time contributors.

Q: How long do reviews take?
Depends on the complexity and maintainer availability — small PRs are usually reviewed faster. Be patient and responsive to feedback.

Q: Can I propose large architectural changes?
Yes, but please open a discussion or issue first to align with maintainers. Major changes should include a migration plan and rationale.


---

Community Guidelines

Be respectful and constructive — we’re all here to learn.

Keep discussions technical and friendly.

Disagreement is fine, but always explain your reasoning clearly.

Mark issues as good first issue if you’d like to mentor newcomers.



---

Thank You 💜

Thanks for contributing to Vira!
Even the smallest contribution — a typo fix, a new test, or a clever idea — helps improve the language for everyone.
Together we’re building something elegant, fast, and expressive. 🚀

— The Vira Language Team
