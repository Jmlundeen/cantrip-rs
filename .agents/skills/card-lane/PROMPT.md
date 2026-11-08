# Role handoffs

Generate after the manifest is ready and immediately after minting a token:
`python tools/maintenance/card-lane.py prompt --queue Q --id W --role implementor --token TOKEN --agent-id AGENT`.
For review use `--role reviewer --card Name`. Bind the agent to the manifest's model; text cannot select runtime settings.

The first command is `tools/worktree.cmd join TOKEN --agent-id AGENT`. Continue in the path join returns. Read this skill and the
relevant rules skills. Implementors run next, fill the packet, use fast checks while editing, and end with finish.
Reviewers use the durable handoff's exact SHA and frozen packet, run their own proof and restore every break;
their final command is `leave MEMBER`. Neither role releases its lease or edits after leaving.

Pass no slot path or lease. Mint a fresh token for a resumed role. Recover lost final responses through ready state.
