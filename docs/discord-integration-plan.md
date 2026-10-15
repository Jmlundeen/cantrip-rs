# Discord authentication and organized-play integration

Status: the first slice is built. [discord-bot.md](discord-bot.md) describes it: `dotnet/src/Cantrip.Discord` connects
over the gateway with Discord.Net; a member with a host role runs `/cantrip host`, another presses **Join**, and
each gets an ephemeral seat ticket that opens the web client into a ticket-only table where both play RW Aggro.
`/cantrip draft` hosts a two-player Winston draft, kept as an event in the server's data directory with each
player's record ([events.md](events.md)), so it survives a restart and a player can return from any computer.
OAuth (stage 1), queues, leagues and everything after are still to come. Until OAuth lands, a seat is proven by
the ticket, which only its Discord account was shown, rather than by an OAuth login.

## Goals

Use Discord as the player identity provider and as the primary interface for
organized play while allowing every Cantrip server to be independently hosted.
The planned Discord features are:

- Draft queues that launch a draft when enough players have joined.
- Discord launch invitations that open the Cantrip web client and authenticate
  the player for the draft.
- Individual and team draft leagues, including membership, teams, schedules,
  results, and standings.
- Ladder profiles, ratings, and match history.
- Bug reports for both Cantrip and the Discord integration.
- Automated patch notes, maintenance notices, league deadlines, and results.
- Room for additional Discord-facing organized-play features.

## Core decisions

### Discord is the identity provider, not the session authority

Use Discord's OAuth2 authorization-code flow. After Discord authenticates a
player, the Cantrip server retrieves the player's Discord user ID and issues its
own expiring Cantrip session. The client must never be trusted to assert a
Discord ID or username directly.

The stable numeric Discord user ID is the account key. Usernames, global display
names, guild nicknames, and avatars are presentation data and may change. They
may be cached and refreshed, but must not identify an account.

Request only the scopes Cantrip needs. The initial requirement is `identify`.
Use PKCE and a cryptographically random OAuth `state`, validate the redirect,
and keep the Discord client secret off the game client. After retrieving the
identity, discard the Discord access token unless a concrete feature requires
continued delegated access.

Discord does not need to remain running after authentication. A running Discord
client or the Discord Social SDK may improve the sign-in experience, but it is
not the security boundary. Existing Cantrip sessions should survive a temporary
Discord outage.

### Each self-hosted installation owns its Discord application

The recommended fully self-hosted model is one Discord application per Cantrip
installation. A human operator creates the application in Discord's Developer
Portal; Discord creates its bot user automatically. The operator installs that
bot into the desired guild and supplies the Cantrip server with its credentials.

The same Discord application provides:

- OAuth identity for players.
- The bot identity for commands, queue controls, and messages.

Installation-specific values are configuration, never compiled constants:

- Discord application/client ID.
- OAuth client secret.
- Bot token.
- Registered OAuth redirect URI.
- Guild ID.
- Channel IDs for queues, results, announcements, and reports.
- Optional Discord role IDs and feature flags.

Secrets must be stored in environment variables or an appropriate local secret
store, never committed or sent to players. The client obtains the OAuth client
ID and authorization information from the Cantrip server it has chosen to join.

A centrally operated official Cantrip bot remains a possible future convenience
service, but it would introduce a hosted dependency and its bot token could not
be distributed to self-hosted operators.

### Discord is the interface; Cantrip is authoritative

The Discord component is an adapter for commands, buttons, modals, and
notifications. Cantrip owns queues, draft rosters, leagues, matches, results,
ratings, and administrative history. The bot must not independently calculate
or accept authoritative match results.

A deployment has three logical components, which may initially share a process:

1. Discord adapter: interactions and outbound Discord messages.
2. Cantrip application service and persistent organized-play data.
3. Draft and match servers, which remain authoritative for play and results.

## Draft queue and client launch flow

1. A player uses `/draft join` or presses a **Join Draft** button.
2. Discord's signed interaction supplies the player's Discord user ID.
3. Cantrip atomically adds that ID to the selected queue.
4. When the required player count is reached, Cantrip creates a draft with an
   immutable roster of Discord user IDs.
5. Cantrip creates one short-lived invitation for each rostered player.
6. The bot sends or exposes the player's launch invitation.
7. The invitation opens the web client in the player's browser, in their seat;
   there is nothing to install.
8. The client completes Discord OAuth for that player in the browser.
9. The draft server admits the player only when the authenticated Discord ID is
   on the roster and matches the invitation.

The launch invitation is not proof of identity. It should be opaque,
draft-specific, bound to the intended Discord user ID, short-lived, single-use,
and revocable. A leaked link therefore cannot let a different Discord account
take the player's seat.

Prefer an HTTPS link such as `https://host/join/<opaque-invitation>` as the
Discord-facing URL. It opens the web client, which the bot serves itself, so
nothing depends on Discord's handling of custom protocols.

Direct messages are a convenience rather than the only delivery path because a
player may block guild-member DMs. Provide a command such as `/draft launch`
that privately returns the caller's current invitation. A queue channel may
mention ready players without revealing invitation values.

Queue transitions must be transactional so simultaneous joins cannot overfill
the queue or launch multiple drafts. Cancellation, expiration, decline, and
replacement-player policies remain to be designed.

## Discord command surface

Prefer Discord application commands, buttons, select controls, and modals over
parsing ordinary messages. This avoids requiring the privileged Message Content
intent and provides verifiable structured interactions.

An initial command vocabulary might include:

- `/draft join`, `/draft leave`, `/draft status`, `/draft launch`.
- `/league create`, `/league join`, `/league team`, `/league standings`.
- `/ladder`, `/profile`, `/match history`.
- `/report bug`.
- `/admin queue cancel`, `/admin result correct`, and league configuration.

Common queue actions should also be buttons on a persistent queue message.
Discord command permissions can hide administrative commands, but the Cantrip
service must independently authorize every action. Never treat Discord UI
visibility as the security boundary.

## Leagues, results, and ladders

Use Discord user IDs for league membership, team membership, draft seats,
ratings, suspensions, and administrative assignments. Historical records may
snapshot the display name used at the time for presentation only.

When a match finishes, the authoritative game server emits a result with a
globally unique match ID. Organized-play processing must be idempotent:

1. Persist the authoritative result.
2. Apply league standings or ladder rating changes exactly once.
3. Record the change in an audit trail.
4. Enqueue Discord announcements.

Retries must not apply the same result twice. Administrative corrections,
invalidations, membership changes, team changes, queue removals, and rating
adjustments need an append-only audit history containing the actor and reason.

Rules still to decide include rating system, season boundaries, tie breakers,
team scoring, result disputes, drops, and corrections.

## Bug reports

Use a Discord modal to collect structured reports:

- Component: client, server, Discord adapter, or card/rules behavior.
- Summary and description.
- Reproduction steps.
- Optional draft, match, or game ID.
- Consent to attach sanitized diagnostics.

Client or server logs should be uploaded through a Cantrip-controlled reporting
path and referenced by an opaque report ID. Do not automatically paste raw logs
into Discord; they may contain account identifiers, addresses, or secrets.

The configured destination may be a private Discord thread/forum or an external
issue tracker. Add rate limiting, duplicate handling, and an abuse policy before
making reporting broadly available.

## Announcements and guild configuration

Store guild configuration in Cantrip's database rather than source or static
code. Likely mappings include:

- Draft queue channel.
- Match result channel.
- League announcement channel.
- Bug report destination.
- Administrator, league-player, or team role IDs.
- Enabled announcement subscriptions.

Announcements may include releases, patch notes, maintenance, draft schedules,
league deadlines, and results. Official Cantrip releases could be consumed from
a signed release feed or release service; operators should be able to publish
installation-specific announcements separately.

## Discord permissions

Begin with the minimum permissions needed by enabled features:

- View Channels.
- Send Messages.
- Embed Links.
- Create Threads, only when reports or league discussions use them.
- Manage Messages, only if maintaining queue panels requires it.
- Manage Roles, only when automatic role synchronization is enabled.

Do not request Administrator. Avoid presence and message-content access. Guild
member access may be necessary for continuous role synchronization or member
enumeration and should be added only for a concrete feature, since Discord
treats member, presence, and message-content access as privileged intents at
scale.

## Reliability and security requirements

- Verify Discord interaction signatures and reject replayed interactions.
- Never expose the bot token, OAuth client secret, or Discord access tokens.
- Encrypt sensitive stored credentials where the deployment supports it.
- Use least-privilege bot permissions and OAuth scopes.
- Rate-limit commands, reports, and invitation generation.
- Make result processing and outbound notifications idempotent.
- Queue and retry Discord messages around API outages and rate limits.
- Continue active Cantrip sessions during temporary Discord failures.
- Provide explicit token rotation and Discord-disconnect procedures.
- Treat a self-hosted server as trusted by its players; make requested OAuth
  scopes visible and minimal.
- Plan account recovery for deleted, disabled, or compromised Discord accounts.

Discord authentication proves control of a Discord account at login time. It
does not establish a real-world identity and does not prevent alternate Discord
accounts.

## Suggested implementation stages

1. **Identity foundation:** per-installation Discord configuration, OAuth
   login, stable Discord-ID accounts, and Cantrip sessions.
2. **Bot foundation:** installation setup, interaction verification, command
   registration, permissions, configuration storage, and notification queue.
3. **Draft queues:** queue panel and commands, transactional launch, rostered
   invitations, web client link, OAuth-to-seat verification, and DM fallback.
4. **Authoritative results:** unique match results, idempotent processing,
   result announcements, and audit history.
5. **Leagues and ladders:** membership, teams, seasons, standings, ratings,
   administrative correction, and dispute workflows.
6. **Reports and announcements:** structured bug reporting, diagnostics,
   release feed, subscriptions, and operator announcements.

## Open questions before implementation

- Whether the Discord adapter runs inside the Cantrip server or as a separately
  deployable process sharing an API and database.
- The exact OAuth callback into the web client, and how it hands the login to
  the seat.
- Database ownership and schema for queues, leagues, ratings, and audit events.
- Draft no-show, cancellation, substitution, and reconnection policies.
- The first supported league formats and rating algorithm.
- Whether Discord roles are merely decorative or grant Cantrip permissions.
- The bug-report destination and diagnostic-retention policy.
- Whether an optional official hosted bot should complement independent bots.
- Backup, migration, and secret-rotation workflows for self-hosted operators.

## References

- [Discord OAuth2](https://discord.com/developers/docs/topics/oauth2)
- [Discord application commands](https://docs.discord.com/developers/docs/interactions/slash-commands)
- [Discord Social SDK authentication](https://discord.com/developers/docs/social-sdk/authentication.html)
- [Discord privileged intents](https://support-dev.discord.com/hc/en-us/articles/6207308062871-What-are-Privileged-Intents)
