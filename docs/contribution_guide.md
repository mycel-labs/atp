# ATP Contribution Guide

This guide provides information for developers who want to contribute to the ATP project.

## Development Setup

1. **Prerequisites**
   - [DFINITY SDK (dfx)](https://internetcomputer.org/docs/current/developer-docs/setup/install)
   - [Rust](https://www.rust-lang.org/tools/install)
   - [Git](https://git-scm.com/downloads)

2. **Fork and Clone the Repository**
   ```bash
   # Fork the repository on GitHub first
   # Then clone your fork
   git clone https://github.com/YOUR_USERNAME/atp.git
   cd atp
   # Add the original repository as upstream
   git remote add upstream https://github.com/mycel-labs/atp.git
   ```

3. **Install Dependencies**
   ```bash
   # Install Rust dependencies
   cargo build
   ```

4. **Start the Local Replica**
   ```bash
   dfx start --background
   ```

5. **Deploy Locally**
   ```bash
   dfx deploy
   ```

## Development Workflow

### Branching Strategy

- `main` - The main branch containing stable code
- `develop` - Development branch for integrating features
- `feature/feature-name` - Feature branches for new development
- `fix/bug-name` - Bug fix branches

### Making Changes

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Your Changes**
   - Follow the code style and architecture patterns in the existing codebase
   - Add tests for new functionality
   - Update documentation as needed

3. **Run Tests**
   ```bash
   cargo test
   ```

4. **Format Your Code**
   ```bash
   cargo fmt
   ```

5. **Check for Linting Issues**
   ```bash
   cargo clippy
   ```

6. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

   Follow the [Conventional Commits](https://www.conventionalcommits.org/) format:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation changes
   - `test:` for adding or modifying tests
   - `refactor:` for code refactoring
   - `chore:` for routine tasks, maintenance, etc.

7. **Push Your Changes**
   ```bash
   git push -u origin feature/your-feature-name
   ```

8. **Create a Pull Request**
   - Go to your fork on GitHub
   - Click "Compare & pull request"
   - Create a new pull request from your fork's feature branch to the original repository's `develop` branch
   - Fill in the PR template with details about your changes

## Code Standards

### Rust Style Guide

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use meaningful variable and function names
- Add documentation comments (`///`) for public functions and types
- Keep functions small and focused on a single responsibility
- Use proper error handling with `Result` types

### Testing

- Write unit tests for all new functionality
- Ensure tests are deterministic and don't depend on external state
- Use mocks for external dependencies when appropriate
- Aim for high test coverage, especially for critical components

### Documentation

- Update documentation when making changes to the API
- Document complex algorithms or business logic
- Keep the README and other documentation up to date
- Use clear and concise language

## Canister Development Guidelines

### State Management

- Use stable memory for persistent state
- Implement proper upgrade hooks (`pre_upgrade` and `post_upgrade`)
- Be mindful of memory usage and cycles consumption

### Security Considerations

- Validate all inputs from users and other canisters
- Implement proper access control for sensitive operations
- Be careful with inter-canister calls and their failure modes
- Follow the principle of least privilege

### Performance

- Optimize for cycles usage
- Use query calls when possible instead of update calls
- Be mindful of the size of messages passed between canisters
- Consider batching operations when appropriate

### Build Configuration

ATP uses cargo features to control which key ID is used during compilation:

```bash
# For local development (default)
cargo build --features local

# For test environment  
cargo build --features test

# For production environment
cargo build --features production
```

The CI/CD pipeline automatically builds all three variants and publishes them to GitHub releases:
- `atp-local.wasm` - Built with `--features local` (uses `dfx_test_key`)
- `atp-test.wasm` - Built with `--features test` (uses `test_key_1`)
- `atp-production.wasm` - Built with `--features production` (uses `key_1`)

## Getting Help

If you need help or have questions about contributing:

- Open an issue on GitHub
- Join the community discussions
- Check the existing documentation and code comments

Thank you for contributing to ATP!
