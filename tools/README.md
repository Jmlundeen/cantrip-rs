# tools

Everything here runs from a double-click or a prompt at the top of the checkout. Each `.cmd` says what it takes in its
header; most pass every argument on to the script behind them.

## Test

| | |
|---|---|
| `test.cmd` | builds everything and runs the fast tests: the C# solution (`dotnet\Cantrip.sln`), then the engine's (`cargo test` over the workspace at the root, which plays `scenarios\`); `/all`, `/noweb`, `/filter`, `/detailed` |
| `scenario-in-browser.cmd name` | plays one `.cantrip` scenario with Bot1 a browser page, in a window to watch (`/headless` without one); `test.cmd /filter "FullyQualifiedName~ScenarioInBrowser"` plays every scenario marked `# Browser` (`docs/scenarios-in-the-browser.md`) |
| `card-render.cmd` | draws a sheet of sample card faces with the real web client in a real browser (`?cardsheet`) and saves it as `TestResults\web\CardRenderTests\card-sheet.png`, to look at after a change to the card template |

## Play a game in the browser

| | |
|---|---|
| `play-game.cmd` | a local table: each seat's deck, and whether you, a bot or a language model plays it |
| `play-vs-bot.cmd` | you against the tactical bot, which your own browser tab plays (`docs/tactical-bot.md`) |
| `play-vs-llm.cmd` | you against a language model (OpenRouter) |
| `play-release-build.cmd` | you against a bot in the shipping Release build of the web client |

## Draft against bots

Each drafts, builds a deck from what was taken, and plays the games. Events are kept in `.cantrip-data`.

| | |
|---|---|
| `draft-winston-vs-bot.cmd` | Winston: 90 cards of the cube in three piles, you and one bot |
| `draft-grid-vs-bot.cmd` | grid: eighteen 3x3 grids, a row or a column each, you and one bot |
| `draft-booster-vs-bots.cmd` | booster: a pod of eight, you and seven bots, three packs then three Swiss rounds |
| `draft.cmd` | any of the three with every option (`-Cube`, `-Player2 Human`, `-Humans N`, ...), and `-Event ID` to host one again |

## Bots only, no browser

| | |
|---|---|
| `bots-random-games.cmd` | thousands of random-bot games at the session layer, reporting any that got stuck |
| `bots-deck-duel.cmd` | two decks against each other many times, and which won more |
| `bots-llm-games.cmd` | a language model against the random bot, with every decision it made |
| `bot-duel.cmd` | one way of playing a seat against another, many times; `watch` tells one game turn by turn |
| `bots-soak.cmd` | thousands of tactical-bot games over the every-card deck and any pool, reporting only those that did not finish, each with the command that replays it |
| `bot-train.cmd` | makes the tactical bot's weights: fits them to its own games, then tunes them by play |
| `bot-draft.cmd` | drafts pods of eight with nobody at the table, two ways of drafting in alternate seats: what their decks are and, with `--games`, which wins; `watch` tells one seat's draft pick by pick |

## Discord

| | |
|---|---|
| `discord-bot.cmd` | runs the Discord bot (`docs/discord-bot.md`) |
| `discord-bot-release.cmd` | stops the bot, builds the Release web client, and starts it again |
| `publish-server.cmd` | publishes to the server: builds the bot and the web client here, builds the engine host there, publishes the art, and restarts the bot once (`-Fast`, `-SkipClient`, `-SkipArt`); the server is in `tools\deploy.local.json` |
| `publish-art.cmd` | publishes card art alone, with the server up: brings the pre-sized art cache up to date here and sends what differs (`-DryRun`, `-SkipBuild`) |

## `maintenance/` - generated files and audits

| | |
|---|---|
| `bump-protocol.cmd` | moves `PlayerView.ProtocolVersion` on and records the wire digest |
| `push-public.cmd` | replays the commits on `origin/main` since the last publish onto the public `jschibel/cantrip-rs` (`-DryRun` lists them) |
| `card-coverage.py "Card"` (or `tools\card-coverage.cmd`) | the lines of a card's Rust file its scenarios never run; exits 1 if any (`cargo-llvm-cov`) |
| `new-card.py "Card"` | everything to read before adding a card, and its file to start from (`docs/adding-cards.md`) |
| `rebuild-card-test-deck.ps1` | regenerates the card-test deck from the card files |
| `card-list.py` | any list of cards (a decklist, a set, a Scryfall search, a text file, a pasted list) as the names the card files use, checked against Scryfall (the `import-cards` skill) |
| `missing-cards.py` | cards on the cube lists, or on one list or file (`--list`, `--file`), that have no card file yet |
| `format-scrape.py <format>` | the cards a format's winning decks play, from MTGGoldfish and MTGTop8, ranked, into `docs/formats/<format>.json` (pages cached in `target/format-cache`) |
| `card-issues.py` | GitHub issues for the cards a format plays that have no file: `sync <format>` opens them, `next <format>` lists the free ones, `claim`/`release "Name"` take and give back a card (a claim is a comment, good for 24 hours from the last PR commit or `card/<slug>` push), `status` lists live claims (the `add-cards` skill) |
| `cubecobra-sync.py` | syncs one cube list from its CubeCobra cube, or fetches a new cube whole by its CubeCobra name (the `import-cards` skill) |
| `missing-art.py` | the card faces and tokens of a cube list or file (`--list`, `--file`) that have no art yet, one a line |
| `pick-order.py` | ranks the cubes' cards into their `*.picks.json` pick orders |
| `art-credits.py` | fills `content/<dataset>/artists.json` with the original printing's artist for each picture not yet credited |
| `mtgpics-art/fetch_art.py` | downloads the original, non-promo art-only picture of each card name read from stdin from mtgpics.com, under its slug file name (the `fetch-card-art` skill) |
| `scryfall-art/fetch_art.py` | the fallback for what mtgpics lacks: downloads the earliest regular printing's Scryfall `art_crop` of each card name read from stdin, under its slug file name; never replaces a file (the `fetch-card-art` skill) |

## The rest

- `scripts/` - the PowerShell behind the launchers above; run the `.cmd`, not these.
- `deploy/` - the server's systemd units, kept for reference; the Discord token and OAuth secret live only on the server, in `/etc/cantrip/bot.env`. `dotnet\tools\Cantrip.ArtWarm` builds the pre-sized art cache `publish-art.cmd` sends.
- `dotnet\tools\Cantrip.Bot` - plays one seat of a real server without a window; every bot seat above is one of these.
- `dotnet\tools\Cantrip.Playtest`, `Cantrip.LlmGames` - the runners behind the `bots-*.cmd` launchers. `Cantrip.BotLab` - behind
  `bot-duel.cmd`, `bot-train.cmd` and `bot-draft.cmd`.
- `dotnet\tools\Cantrip.WebHost` - serves the Release client for `play-release-build.cmd`.
