# Contributing to SoroTask

Thanks for contributing to SoroTask. This project is split into three parts:

- `contract` (Rust/Soroban smart contract)
- `keeper` (Node.js off-chain bot)
- `frontend` (Next.js dashboard)

## Contribution Flow

1. Fork the repository.
2. Create a branch from `main`:
   - `feat/<short-description>` for features
   - `fix/<short-description>` for bug fixes
   - `docs/<short-description>` for documentation
3. Make focused changes in the relevant package(s).
4. Run formatting/lint checks before opening a PR.
5. Open a Pull Request using the PR template.

## Local Setup

### Contract

```bash
cd contract
cargo build --target wasm32-unknown-unknown --release
```

### Keeper

```bash
cd keeper
npm install
node index.js
```

### Frontend

```bash
cd frontend
npm install
npm run dev
```

## Code Style and Quality Checks

Run the checks for the part(s) you changed.

### Rust (contract)

```bash
cd contract
cargo fmt --all
```

### JavaScript/TypeScript (frontend)

```bash
cd frontend
npm run lint
```

If you changed both Rust and frontend code, run both checks.

## Pull Request Expectations

Every PR should:

- Have a clear title and summary.
- Explain what changed and why.
- Link related issue(s) when applicable.
- Include testing notes (what was run and results).
- Keep scope focused; avoid unrelated refactors.
- Include screenshots or short recordings for frontend UI changes.

## Commit Guidance

- Use small, reviewable commits.
- Write clear commit messages describing intent.
- Keep each commit logically coherent.

## Reporting Bugs and Requesting Features

When opening an issue, include:

- What you expected to happen.
- What actually happened.
- Steps to reproduce.
- Environment details (OS, runtime/tool versions) when relevant.
