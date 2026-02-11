# skill-issue

Static security analyzer for Claude skill directories.

Scans skill files for hidden instructions, credential exfiltration, prompt injection, and other security risks before you install or publish a skill.

## Install

```bash
cargo install skill-issue
```

Or download a binary from [Releases](https://github.com/daviddrummond95/skill-issue-cli/releases).

## Usage

```bash
# Scan the current directory
skill-issue .

# JSON output
skill-issue ./my-skill --format json

# Only show warnings and above
skill-issue ./my-skill --severity warning

# Ignore specific rules
skill-issue ./my-skill --ignore SL-NET-001 SL-FS-002
```

## Documentation

Full documentation is available at **[skill-issue.sh](https://skill-issue.sh)**.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
