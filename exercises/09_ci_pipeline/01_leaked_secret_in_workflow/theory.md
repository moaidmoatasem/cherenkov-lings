# Theoretical Context: Plaintext Credentials in CI Workflow Definitions

## Production Incident: The Codecov Bash Uploader Compromise (April 2021)

Codecov distributed a small shell script that thousands of CI pipelines piped straight into `bash` to upload coverage reports. Some time around late January 2021, an attacker obtained credentials from a flaw in Codecov's Docker image creation process and began modifying that script on the delivery server. The altered uploader did one extra thing: it collected the environment variables present on the CI runner and sent them to an attacker-controlled host. The change went undetected for roughly two months, and because CI runners routinely hold cloud keys, registry tokens, and deploy credentials in their environment, a coverage tool became a credential harvester for every pipeline that invoked it.

The lesson is not "avoid Codecov." It is that **a CI runner's environment is a blast radius**. Every secret materialised on that runner is exposed to every step that runs afterwards, to every third-party action in the job, and to anything that can read the build log. A credential written as a literal in the workflow file is worse still: it is exposed before the job even starts, to everyone with read access to the repository, and it persists in git history after the "fix" commit removes it.

## The Underlying Mechanism

1. **Workflow files are source code.** `.github/workflows/*.yml` is committed, mirrored to every fork and clone, and readable by anyone with repository read access — which on an open-source project means everyone. Deleting the line later does not remove it; `git log -p` still yields it, and the credential must be treated as burned from the moment it was pushed.
2. **Secret expressions are resolved late.** `${{ secrets.NAME }}` is not a value in the file — it is an expression the runner resolves at step-execution time from encrypted storage. The literal never exists in the repository, only in the runner's memory for the lifetime of that step.
3. **Log masking only covers registered secrets.** CI providers scrub values that came from the secret store out of log output. A hardcoded literal was never registered, so it is not masked — one `env | sort` in a debug step and the token is in a public build log.
4. **Detection is shape-based.** Credential formats are deliberately recognisable: `ghp_` + 36 characters for a GitHub PAT, `AKIA` + 16 for an AWS access key ID, `glpat-` for GitLab, `-----BEGIN … PRIVATE KEY-----` for PEM material. Scanners key off these prefixes, which is exactly what this drill's validator does.

```
[Anti-Pattern: Literal Credential in the Workflow File]

  workflow.yml (committed, world-readable, immortal in git history)
  └── env:
        CRUCIBLE_TOKEN: "ghp_C4f81a2Kx9Qw..."   ← the secret IS the file
              │
              ├──► every fork, clone, and mirror
              ├──► every CI log (unmasked — never registered)
              └──► git history, forever after the "removal" commit  ❌

[Resilient Pattern: Late-Bound Secret Reference]

  workflow.yml (committed)          Encrypted secret store
  └── env:                          ┌──────────────────────┐
        CRUCIBLE_TOKEN:  ───────────┤ CRUCIBLE_TOKEN=ghp_… │
          ${{ secrets.CRUCIBLE_TOKEN }}  └──────────┬───────────┘
              │                                 │
              ▼ resolved at step execution      │
        runner memory, this step only  ◄────────┘
              │
              └──► log output automatically masked  ✅
```

The remediation is two steps, and the second is the one people skip: reference the secret through the store, **and rotate the exposed credential**. A token that reached a commit is compromised regardless of whether the commit was later amended.

You will now simulate this in the Crucible: run `cherenkov-lings pipeline validate` against the workflow, read the `HARDCODED_SECRET` finding, and move the credential behind a `secrets` expression until the policy score reaches 100/100.
