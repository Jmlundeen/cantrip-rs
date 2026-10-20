# A card that adds something to the client

Read this only when a card puts into the game something the client has never shown, asks something it has never asked,
or offers an action or cost it has never offered. A card that reuses existing prompts, actions and state needs none of
it: its headless scenarios prove it.

Ask what a player needs from the client to play the card, and check each against what the client already presents
(`docs/web-client.md`; `crates/cantrip-host` builds the views, prompts and actions the client draws;
`dotnet/src/Cantrip.Client.Presentation` and `Cantrip.Client.Web` draw them):

- **State:** anything new on an object, a player or a zone (doors, a chosen name or type, counters of a new kind, a
  face, a designation, a revealed card, a new zone or pile) reaches the host view (`crates/cantrip-host/src/view.rs`)
  and is drawn - legibly, at table size, for both the owner and the opponent as the rules let each see it.
- **Questions:** every `g.ask(..)` the card makes has a prompt the client shows, with wording true to what the choice
  does. A new `Purpose` or `RequestKind` gets its own view and caption; never map it onto an existing one that means
  something else.
- **Actions:** every ability, special action and cost the card offers is clickable, with its cost and what it does
  shown, and distinguishable from the card's other actions.

The client change is made in the same work as the card - it is part of the card, not a follow-up - and proven with a
**browser scenario**: `# Browser` at the top and `? Client.` checks of what the page shows
(`docs/scenarios-in-the-browser.md`), run with `tools\test.cmd /filter "ScenarioInBrowser"` (or
`tools\scenario-in-browser.cmd name` to watch) after `cargo build --profile cards -p cantrip-host` - the table runs the
last-built `cantrip-native-host.exe`, and one that predates the card publishes nothing. If the browser driver cannot
click its way through the new prompt or action, extend `ScenarioPlayer`/`TablePage`; never drop the check to fit the
driver. A protocol change bumps the version and passes the wire-shape test, in the same commit as the change. Card
agents run browser tests only for such an interaction, one agent at a time, and only that interaction's scenarios.
