# Configuration & Guardrails

RTK provides customizable configurations to control what commands an AI agent can execute and to protect your codebase from exposing credentials or secrets.

---

## Configuration Files

RTK merges configurations from two levels:
1. **Global Configuration**: Stored at `~/.config/rtk/config.json`. Applies to all commands run by the user.
2. **Local Workspace Configuration**: Stored at `.rtk.json` in the root of your project directory. Overrides global settings for project-specific configurations.

To inspect the active configuration configuration merged from both files, run:
```bash
rtk config show
```

### Environment variables

| Variable | Purpose |
|----------|---------|
| `RTK_DB_PATH` | Global tracking DB (command logs, stats); read by [claude-statusline-pro](https://github.com/andreafinazziinfo/claude-statusline-pro) `💾` indicator when set in its `config.sh` |
| `RTK_PROJECT_DB_PATH` | Project-local DB (memory, session) override |
| `RTK_INDEX_DB_PATH` | Code index SQLite path override |

---

## Command Guardrails (Deny Lists)

Autonomous agents can sometimes hallucinate destructive commands. RTK intercepts these commands via the `rtk rewrite` shell hook before they are executed.

### How Guardrails Work
If a command matches one of the patterns configured in the deny list, RTK blocks it immediately, exits with a non-zero status code, and prints a warning to the agent.

### Default Blocked Commands
By default, RTK blocks commands like:
* `git push --force` or `git push -f`
* `git reset --hard` (when running in untracked workspace states)
* `rm -rf /` (or similar destructive file system commands)

### Adding Custom Guardrails
You can add custom regex patterns to your deny list. For example, to prevent an agent from deleting databases:
```bash
rtk config deny add "drop database"
```

To prevent resetting Kubernetes clusters:
```bash
rtk config deny add "kubectl delete namespace"
```

### Strict chained commands

By default, chained commands (`&&`, `;`, `|`, `||`) bypass rewrite but **deny rules still apply per segment**. Set in `.rtk.json`:

```json
{ "strict_chained": true }
```

When enabled, any chained command is blocked by `rtk rewrite` (exit code 2) unless you split into single commands.

---

## Data Loss Prevention (DLP)

To prevent LLM providers from storing your private credentials in their training logs or caching databases, RTK filters command outputs through a strict Data Loss Prevention scanner.

### Redacted Secrets
RTK scans all command stdout/stderr for the following:
* **API Keys**: Patterns matching Stripe, AWS, Slack, OpenAI, GitHub, and generic tokens.
* **Credentials**: DB connection strings containing passwords (e.g. `postgres://user:password@host/db`).
* **Private Keys**: Blocks containing `-----BEGIN PRIVATE KEY-----` or `-----BEGIN RSA PRIVATE KEY-----`.
* **High-Entropy Tokens**: Dynamic Shannon entropy checks to detect random credentials that don't match specific static regexes.

Any detected secrets are replaced in the command's final output with a marker like `[REDACTED_API_KEY]` or `[REDACTED_SECRET]`.

### Adding Custom DLP Regexes
If your organization uses custom token formats (e.g., `CORP_SEC_[0-9a-zA-Z]{24}`), you can register them in your RTK settings:

```bash
rtk config dlp add "CORP_SEC_[0-9a-zA-Z]{24}"
```
Once added, any shell output matching this regular expression is automatically scrubbed before it reaches the AI agent.
