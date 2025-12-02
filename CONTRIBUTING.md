# Contributing to VC BC

Thank you for your interest in contributing to VCBC! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

This project adheres to a code of conduct to ensure a welcoming environment for all contributors. By participating, you agree to:

- Be respectful and inclusive
- Focus on constructive feedback
- Accept responsibility for mistakes
- Show empathy towards other contributors
- Help create a positive community

## Getting Started

### Prerequisites

- **Rust**: Version 1.70 or later
- **Git**: Version control system
- **Cargo**: Rust package manager (included with Rust)

### Setup

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/vestcodes/vcbc.git
   cd vcbc
   ```

3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/vestcodes/vcbc.git
   ```

4. **Install development tools**:
   ```bash
   # Code formatting
   rustup component add rustfmt

   # Linting
   rustup component add clippy

   # Additional tools
   cargo install cargo-audit    # Security auditing
   cargo install cargo-tarpaulin # Code coverage
   cargo install cargo-benchcmp  # Benchmark comparison
   ```

## Commit Guidelines

This project uses **Conventional Commits** for automatic semantic versioning and changelog generation. All commits must follow the conventional commit format:

### Format
```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools
- `ci`: Changes to CI configuration files and scripts
- `build`: Changes that affect the build system or external dependencies
- `revert`: Reverts a previous commit

### Examples
```bash
feat: add MPT path compression optimization
fix(blockchain): resolve chain validation deadlock
docs: update API reference for new endpoints
style: format code with rustfmt
refactor: extract common validation logic
perf: optimize proof verification algorithm
test: add integration tests for network sync
chore: update Cargo.lock dependencies
ci: add security audit to GitHub Actions
build: upgrade to Rust 1.75
revert: remove faulty MPT optimization
```

### Scope (Optional)
Scopes help categorize changes:
- `blockchain`: Core blockchain logic
- `mpt`: Merkle-Patricia Trie
- `network`: Networking and peer communication
- `api`: REST API endpoints
- `cli`: Command-line interface
- `docs`: Documentation

### Validation
All commits are automatically validated by GitHub Actions:
- **Commit Message Format**: Must follow conventional commit standards
- **Code Quality**: Rustfmt and Clippy checks pass
- **Tests**: All tests must pass with minimum 80% coverage
- **Security**: No security vulnerabilities detected

Pull requests with invalid commits will be rejected automatically.

### Commit Message Helper

A helper script is provided to assist with creating properly formatted commit messages:

```bash
# Interactive mode to build commit message
./scripts/commit-msg-helper.sh --interactive

# Validate a commit message
./scripts/commit-msg-helper.sh --validate "feat: add new feature"

# Show examples
./scripts/commit-msg-helper.sh --examples
```

## Development Workflow

### 1. Choose an Issue

- Check the [issue tracker](https://github.com/vestcodes/vcbc/issues) for open issues
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to indicate you're working on it

### 2. Create a Branch

Create a feature branch from `main`:
```bash
git checkout main
git pull upstream main
git checkout -b feature/your-feature-name
```

### 3. Make Changes

- Write tests first (TDD approach)
- Implement the feature
- Ensure all tests pass
- Run quality checks

### 4. Commit Changes

Follow conventional commit format:
```bash
git add .
git commit -m "feat: add new MPT optimization

- Implements path compression for better storage efficiency
- Adds benchmark tests for performance validation
- Updates documentation with new optimization details

Closes #123"
```

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Code Style

This project follows Rust's official style guidelines with additional rules:

### Formatting

- Use `rustfmt` for automatic formatting:
  ```bash
  cargo fmt
  ```

- Check formatting without changes:
  ```bash
  cargo fmt --check
  ```

### Linting

- Use `clippy` for additional linting:
  ```bash
  cargo clippy
  ```

- Fix common issues automatically:
  ```bash
  cargo clippy --fix
  ```

### Naming Conventions

- **Functions**: `snake_case`
- **Types/Structs**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

### Documentation

- All public APIs must have documentation comments
- Use proper Rustdoc syntax
- Include examples where helpful

```rust
/// Calculates the Merkle root hash for the trie.
///
/// # Examples
///
/// ```
/// use clean_blockchain::MerklePatriciaTrie;
///
/// let mut trie = MerklePatriciaTrie::new();
/// trie.insert("key", b"value".to_vec());
/// let root_hash = trie.root_hash();
/// ```
pub fn root_hash(&self) -> Vec<u8> {
    // implementation
}
```

## Testing

### Unit Tests

- Write comprehensive unit tests for all new code
- Test edge cases and error conditions
- Use descriptive test names

```rust
#[test]
fn test_mpt_insert_and_retrieve() {
    let mut trie = MerklePatriciaTrie::new();
    let key = "test_key";
    let value = b"test_value";

    // Test insert
    assert!(trie.insert(key, value.to_vec()).is_ok());

    // Test retrieve
    assert_eq!(trie.get(key), Some(value.to_vec()));
}
```

### Property-Based Testing

Use `proptest` for testing with random inputs:

```rust
proptest! {
    #[test]
    fn test_arbitrary_key_value_pairs(ref key in "\\PC*", ref value in prop::collection::vec(any::<u8>(), 0..1000)) {
        let mut trie = MerklePatriciaTrie::new();
        prop_assert!(trie.insert(key, value.clone()).is_ok());
        prop_assert_eq!(trie.get(key), Some(value.clone()));
    }
}
```

### Integration Tests

Add integration tests in `tests/` directory for end-to-end functionality.

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_mpt_insert_and_retrieve

# Run with coverage
cargo tarpaulin --out Html
```

## Documentation

### API Documentation

- Generate documentation locally:
  ```bash
  cargo doc --open
  ```

- Build docs for publishing:
  ```bash
  cargo doc
  ```

### README Updates

- Update README.md for significant new features
- Add examples and usage instructions
- Keep badges and status information current

### Changelog

Maintain a CHANGELOG.md file following [Keep a Changelog](https://keepachangelog.com/) format.

## Submitting Changes

### Pull Request Guidelines

1. **Title**: Use descriptive, concise titles
2. **Description**: Explain what and why
3. **Tests**: Include test coverage for new code
4. **Documentation**: Update relevant docs
5. **Breaking Changes**: Clearly mark breaking changes

### PR Template

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Changelog updated
```

### Review Process

1. Automated checks run (CI/CD)
2. Code review by maintainers
3. Requested changes addressed
4. Approval and merge

## Reporting Issues

### Bug Reports

Use the bug report template and include:

- **Expected behavior**
- **Actual behavior**
- **Steps to reproduce**
- **Environment details** (Rust version, OS, etc.)
- **Logs or error messages**

### Feature Requests

Use the feature request template and include:

- **Problem description**
- **Proposed solution**
- **Alternative solutions considered**
- **Additional context**

### Security Issues

For security vulnerabilities, please email security@yourproject.com instead of creating a public issue.

## Recognition

Contributors are recognized in:
- CHANGELOG.md for significant contributions
- GitHub's contributor insights
- Project documentation

Thank you for contributing to VCBC! 🚀
