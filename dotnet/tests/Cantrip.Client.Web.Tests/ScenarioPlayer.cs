using System.Text.RegularExpressions;
using Cantrip.Art;
using Cantrip.Protocol;
using Cantrip.Server.Host;
using Cantrip.Server.Rules;
using Microsoft.Playwright;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// Plays a <c>.cantrip</c> scenario with Bot1's seat a browser page (docs/scenarios-in-the-browser.md): a real table plays
/// the script, says for each of Bot1's questions what the script means it to answer, and this clicks its way there as a
/// person would - the card, the target, the row of a menu, the mana in the pool, the button - with real mouse events at
/// the places the page publishes. The table fails the scenario at the line when the page answers otherwise. The checks of
/// what the client shows (<c>Client.</c> keys) are judged here against the page.
/// <para>
/// The page is set up so that nothing answers for the player but the player: auto-mana off (a menu asks the colour),
/// every step stopped at and priority held, so the page never passes by itself where the script means an action.
/// </para>
/// </summary>
public sealed class ScenarioPlayer
{
    /// <summary>Every step of either player's turn, so the page asks the player at every priority.</summary>
    private static readonly string EveryStep = string.Join(
        ",",
        (TurnStepView[])[TurnStepView.Upkeep, TurnStepView.Draw, TurnStepView.PrecombatMain, TurnStepView.BeginCombat, TurnStepView.DeclareAttackers,
            TurnStepView.DeclareBlockers, TurnStepView.CombatDamage, TurnStepView.EndOfCombat, TurnStepView.PostcombatMain, TurnStepView.End]);

    private static readonly TimeSpan Patience = TimeSpan.FromSeconds(90);

    private readonly TablePage _page;
    private readonly ScenarioTable _table;
    private readonly List<string> _trace = [];

    /// <summary>The label of the button the last choice of cards was confirmed with; empty when the last pick answered itself.</summary>
    private string _lastConfirm = string.Empty;

    private ScenarioPlayer(TablePage page, ScenarioTable table)
    {
        _page = page;
        _table = table;
    }

    /// <summary>
    /// Plays the scenario at <paramref name="path"/> to its end. Null when it passed, page and table alike; else what
    /// failed, with the table's report or the page's, and the last steps taken.
    /// </summary>
    public static async Task<string?> PlayAsync(WebClient web, string path)
    {
        ScenarioSpec scenario = ScenarioSpec.Load(path, [1]);
        await using ScenarioTable table = await ScenarioTable.StartAsync(scenario, WebClient.Content);
        await using Microsoft.AspNetCore.Builder.WebApplication art = await ArtServer.StartAsync(
            new ArtStore(new ArtStoreOptions { ContentRoot = WebClient.Content, CacheDirectory = WebClient.ArtCache }), "127.0.0.1", 0);
        await using TablePage page = await TablePage.OpenAsync(
            web,
            $"server={Uri.EscapeDataString(table.Address.AbsoluteUri)}&art={Uri.EscapeDataString(art.Urls.First())}&seat=1&name=Bot1",
            WebClient.Pictures(Path.Combine(nameof(ScenarioPlayer), scenario.Name)),
            settings: new Dictionary<string, string> { ["auto-mana"] = "false" },
            storage: new Dictionary<string, string> { ["cantrip-stops-mine"] = EveryStep, ["cantrip-stops-theirs"] = EveryStep });

        var player = new ScenarioPlayer(page, table);
        try
        {
            return await player.PlayAsync();
        }
        catch (Exception error) when (error is TimeoutException or Xunit.Sdk.XunitException or PlaywrightException or NotDrivenException)
        {
            await page.PictureAsync("failed");
            return player.Failure(error.Message);
        }
    }

    private async Task<string?> PlayAsync()
    {
        bool holding = false;
        while (true)
        {
            ScenarioProgress progress = await _table.NextAsync(Patience);
            if (progress.Expected is { Seat: not 1 })
            {
                continue;
            }

            // The page is brought up to the question asked (or to the end) before anything on it is judged.
            if (progress.Expected is { } waiting)
            {
                _trace.Add($"table asks request {waiting.RequestId}");
            }

            // A page past the question answered it itself (a pass it would make for anyone): the table judged that answer
            // as it judges a click, so there is nothing to do but note it.
            TableState state = progress.Expected is { } asked
                ? await _page.SettledAsync(s => s.RequestId >= asked.RequestId)
                : await _page.SettledAsync();

            if (!holding)
            {
                await _page.ClickAsync("Hold Priority", "toggle");
                holding = true;
                state = await _page.SettledAsync();
            }

            foreach (ScenarioCheck check in progress.Checks)
            {
                if (await JudgeAsync(check, state) is { } wrong)
                {
                    await _page.PictureAsync("check-failed");
                    return Failure(wrong);
                }
            }

            if (progress.Verdict is { } verdict)
            {
                if (!verdict.Passed)
                {
                    return Failure(verdict.Report ?? "the table failed the scenario");
                }

                foreach (ScenarioCheck check in verdict.Client)
                {
                    if (await JudgeAsync(check, state) is { } wrong)
                    {
                        await _page.PictureAsync("check-failed");
                        return Failure(wrong);
                    }
                }

                return _page.Errors.Count > 0 ? Failure("the page threw: " + string.Join(" | ", _page.Errors)) : null;
            }

            if (progress.Expected is { } passed && state.RequestId > passed.RequestId)
            {
                _trace.Add($"{passed.Line ?? "(default)"}: {Describe(passed)}, answered by the page itself");
                continue;
            }

            if (progress.Expected is { } expected)
            {
                _trace.Add($"{expected.Line ?? "(default)"}: {Describe(expected)}");
                await AnswerAsync(expected, state);
            }
        }
    }

    private string Failure(string message) =>
        $"{message}\n\nBot1's last answers, clicked:\n  {string.Join("\n  ", _trace.TakeLast(15))}";

    private static string Describe(ScenarioExpected expected) => expected.Undo
        ? "Undo"
        : expected.Answer switch
        {
            TakeActionAnswer action => $"{action.Kind} {action.Source.Value}{(action.Color is { } c ? " " + c : string.Empty)}",
            { } answer => answer.GetType().Name,
            null => "nothing",
        };

    // --- Answers, as clicks ---

    private async Task AnswerAsync(ScenarioExpected expected, TableState state)
    {
        if (expected.Undo)
        {
            await _page.ClickAsync("Undo", "button");
            return;
        }

        switch (expected.Answer)
        {
            case PassAnswer:
                await _page.Page.Keyboard.PressAsync("Space");
                return;

            case TakeActionAnswer action:
                await TakeAsync(action, state);
                return;

            case AttackersDeclaredAnswer attack:
                foreach (AttackMessage one in attack.Attacks)
                {
                    await ClickHandleAsync(one.Attacker.Value);
                }

                await ClickButtonStartingAsync(attack.Attacks.Count == 0 ? "No attack" : "Attack with");
                return;

            case BlockersDeclaredAnswer block:
                foreach (BlockMessage one in block.Blocks)
                {
                    await ClickHandleAsync(one.Blocker.Value);
                    await ClickHandleAsync(one.Attacker.Value);
                }

                await _page.Page.Keyboard.PressAsync("Space");
                return;

            case MayChosenAnswer may:
                // A "you may" asks Yes or No; a mulligan, which is a may too, Keep or Mulligan.
                if (state.Controls.Any(c => c.Kind == "button" && c.Label == "Yes"))
                {
                    await _page.ClickAsync(may.Accept ? "Yes" : "No", "button");
                }
                else
                {
                    await ClickButtonStartingAsync(may.Accept ? "Mulligan" : "Keep");
                }

                return;

            case CardsChosenAnswer chosen:
            {
                foreach (ViewHandle card in chosen.Cards)
                {
                    await ClickHandleAsync(card.Value);
                }

                // An exact count answers itself on the last click; otherwise the dock's button, named for what it does
                // with that many ("Discard 1"), or the pass key where that is the button.
                TableState after = await _page.SettledAsync();
                _lastConfirm = string.Empty;
                if (after.RequestId == state.RequestId)
                {
                    ControlState? confirm = after.Controls.FirstOrDefault(c => c.Kind == "button" && (c.Label.EndsWith($" {chosen.Cards.Count}", StringComparison.Ordinal) || chosen.Cards.Count == 1 && c.Label.EndsWith(" it", StringComparison.Ordinal)));
                    if (confirm is not null)
                    {
                        _lastConfirm = confirm.Label;
                        await _page.Page.Mouse.ClickAsync(confirm.X, confirm.Y);
                    }
                    else
                    {
                        await _page.Page.Keyboard.PressAsync("Space");
                    }
                }

                return;
            }

            case ModesChosenAnswer modes:
            {
                // A choice of modes: a button for each, named by the mode's text; one mode answers on its click, several
                // are picked and then sent with Done.
                foreach (int mode in modes.Modes)
                {
                    string label = state.Modes is { } asked && mode < asked.Count
                        ? asked[mode]
                        : throw new NotDrivenException($"mode {mode}, which the page does not list", state);
                    await ClickButtonNamedAsync(label);
                }

                TableState after = await _page.SettledAsync();
                if (after.RequestId == state.RequestId && modes.Modes.Count != 1)
                {
                    await ClickButtonStartingAsync("Done");
                }

                return;
            }

            case TargetsChosenAnswer targets:
            {
                foreach (TargetMessage target in targets.Targets)
                {
                    await ClickTargetAsync(target);
                }

                // A target chosen as the spell is cast, after its additional cost, is confirmed with "Done with 1 target".
                TableState after = await _page.SettledAsync();
                if (after.RequestId == state.RequestId && after.Controls.Any(c => c.Kind == "button" && c.Label.StartsWith("Done with", StringComparison.Ordinal)))
                {
                    await ClickButtonStartingAsync("Done with");
                }

                return;
            }

            default:
                throw new NotDrivenException(expected.Answer?.GetType().Name ?? "nothing", state);
        }
    }

    private async Task TakeAsync(TakeActionAnswer action, TableState state)
    {
        switch (action.Kind)
        {
            case ActionKindView.ConfirmCasting:
                await _page.ClickAsync("Okay", "button");
                return;

            case ActionKindView.PayCost:
                await ClickButtonStartingAsync("Pay");
                return;

            case ActionKindView.DeclineCost:
                await ClickButtonStartingAsync("Don't pay");
                return;

            case ActionKindView.CancelCasting:
                await _page.ClickAsync("Cancel", "button");
                return;

            // The value of X, a click at a time on the dock's plus and minus.
            case ActionKindView.IncreaseX:
                await _page.ClickAsync("+", "button");
                return;

            case ActionKindView.DecreaseX:
                await _page.ClickAsync("−", "button");
                return;

            case ActionKindView.AssignMana:
            {
                PlayerState me = state.Players?.FirstOrDefault(p => p.Seat == state.Seat)
                    ?? throw new NotDrivenException("AssignMana, with no pool shown", state);
                ManaChipState chip = me.Mana.FirstOrDefault(c => c.Type == action.Mana?.ToString() && c.Clickable)
                    ?? throw new Xunit.Sdk.XunitException($"No {action.Mana} in the pool to click: {string.Join(", ", me.Mana.Select(c => $"{c.Count} {c.Type}"))}");
                await _page.Page.Mouse.ClickAsync(chip.X, chip.Y);
                return;
            }

            case ActionKindView.PlayLand or ActionKindView.CastSpell or ActionKindView.ActivateAbility or ActionKindView.TurnFaceUp or ActionKindView.UnlockDoor:
            {
                await ClickHandleAsync(action.Source.Value);
                TableState after = await _page.SettledAsync();

                // A card with more than one thing to do opens a menu: the row the answer is.
                if (after.Controls.Any(c => c.Kind == "item"))
                {
                    await ChooseRowAsync(action, after);
                }

                foreach (TargetMessage target in action.Targets)
                {
                    await ClickTargetAsync(target);
                }

                if (action.Sacrifice is { } sacrifice)
                {
                    await ClickHandleAsync(sacrifice.Value);
                }

                return;
            }

            default:
                throw new NotDrivenException($"{nameof(TakeActionAnswer)} {action.Kind}", state);
        }
    }

    /// <summary>The menu's row for the answer: the one adding its colour, for a mana ability of any colour.</summary>
    private async Task ChooseRowAsync(TakeActionAnswer action, TableState state)
    {
        List<ControlState> rows = [.. state.Controls.Where(c => c.Kind == "item")];

        // A card with several abilities lists them: the rows of the one the answer is. A menu of one ability's rows (a mana
        // ability's colours) is the same menu whole.
        if (rows.Any(r => r.Ability == action.AbilityIndex))
        {
            rows = [.. rows.Where(r => r.Ability == action.AbilityIndex)];
        }

        ControlState? row = action.Color is { } color
            ? rows.FirstOrDefault(r => r.Label.Contains($"{{{Symbol(color)}}}", StringComparison.Ordinal))
            : DoorOf(action, state) is { } door ? rows.FirstOrDefault(r => r.Label.Contains(door, StringComparison.Ordinal))
            : rows.Count == 1 ? rows[0] : null;
        if (row is null)
        {
            throw new NotDrivenException($"which row of the menu is {action.Kind}{(action.Color is { } c ? " " + c : string.Empty)}", state);
        }

        await _page.Page.Mouse.ClickAsync(row.X, row.Y);
    }

    /// <summary>
    /// The words of the menu row for the door an action is about (" — Unholy Annex" ends a cast of that half, "Unlock Ritual Chamber" starts its unlock): the half of a split card that a cast is
    /// of (the left way is the half printed first, CR 709.3), or the locked door of a Room that an unlock opens - the only
    /// locked one, or, with both locked, the left's before the right's (CR 709.5e). Null for any other action.
    /// </summary>
    private static string? DoorOf(TakeActionAnswer action, TableState state)
    {
        CardState? card = state.Cards.FirstOrDefault(c => c.Visible && c.Handle == action.Source.Value);
        if (card is null)
        {
            return null;
        }

        if (action.Kind == ActionKindView.CastSpell && action.AbilityIndex is 238 or 239)
        {
            string[] halves = card.Halves.Split("; ", StringSplitOptions.RemoveEmptyEntries);
            return halves.Length == 2 ? " — " + halves[action.AbilityIndex == 239 ? 0 : 1] : null;
        }

        if (action.Kind == ActionKindView.UnlockDoor)
        {
            string[] locked = [.. card.Doors.Split("; ", StringSplitOptions.RemoveEmptyEntries)
                .Where(door => door.EndsWith(":locked", StringComparison.Ordinal))
                .Select(door => door[..^":locked".Length])];
            return locked.Length switch { 0 => null, 1 => "Unlock " + locked[0], _ => "Unlock " + locked[action.AbilityIndex == 0 ? 0 : 1] };
        }

        return null;
    }

    private static char Symbol(ColorView color) => color switch
    {
        ColorView.White => 'W',
        ColorView.Blue => 'U',
        ColorView.Black => 'B',
        ColorView.Red => 'R',
        ColorView.Green => 'G',
        _ => '?',
    };

    private async Task ClickTargetAsync(TargetMessage target)
    {
        if (target.Object.Value != 0)
        {
            await ClickHandleAsync(target.Object.Value);
            return;
        }

        TableState state = await _page.SettledAsync();
        PlayerState box = state.Players?.FirstOrDefault(p => p.Seat == target.Player)
            ?? throw new Xunit.Sdk.XunitException($"Seat {target.Player} is not shown to click.");
        await _page.Page.Mouse.ClickAsync(box.X, box.Y);
    }

    /// <summary>
    /// A card in a graveyard or in exile is drawn only while that zone is open: a person opens the pile, or the Exile button,
    /// and clicks it there. The zones are tried in turn - each opened, looked into, shut again if it was not there - and the
    /// card in the one that holds it is returned, its zone left open. Null when none holds it.
    /// </summary>
    private async Task<CardState?> OpenZoneHoldingAsync(int handle)
    {
        TableState state = await _page.SettledAsync();
        List<(float X, float Y)> openers = [];
        foreach (PlayerState player in state.Players ?? [])
        {
            openers.AddRange((player.Zones ?? []).Where(z => z.Text.StartsWith("Exile", StringComparison.Ordinal)).Select(z => (z.X, z.Y)));
            if (player.GraveyardX > 0)
            {
                openers.Add((player.GraveyardX, player.GraveyardY));
            }
        }

        foreach ((float x, float y) in openers)
        {
            await _page.Page.Mouse.ClickAsync(x, y);
            TableState open = await _page.SettledAsync();
            if (open.Cards.FirstOrDefault(c => c.Visible && c.Handle == handle) is { } found)
            {
                return found;
            }

            // Not in this one: shut it, where its opener is now (the board moves over when a graveyard opens).
            await _page.Page.Mouse.ClickAsync(x, y);
            await _page.SettledAsync();
        }

        return null;
    }

    private async Task ClickHandleAsync(int handle)
    {
        TableState state = await _page.SettledAsync();
        CardState card = state.Cards.FirstOrDefault(c => c.Visible && c.Handle == handle)
            ?? await OpenZoneHoldingAsync(handle)
            ?? throw new Xunit.Sdk.XunitException(
                $"Handle {handle} is not in view. In view: {string.Join(", ", state.Cards.Where(c => c.Visible).Select(c => $"{c.Name} #{c.Handle} ({c.Place})"))}");
        // The stack fans its cards, each covering most of the one beneath: the strip of the card's left edge is the part of
        // it that is always in view.
        float x = card.Place == "stack" ? card.X - card.W / 2 + 6 : card.X;
        await _page.Page.Mouse.ClickAsync(x, card.Y);
    }

    private async Task ClickButtonNamedAsync(string label)
    {
        TableState state = await _page.SettledAsync();
        ControlState button = state.Controls.FirstOrDefault(c => c.Kind == "button" && c.Label == label)
            ?? throw new Xunit.Sdk.XunitException($"No button '{label}'. There are: {string.Join(", ", state.Controls.Where(c => c.Kind == "button").Select(c => c.Label))}");
        await _page.Page.Mouse.ClickAsync(button.X, button.Y);
    }

    private async Task ClickButtonStartingAsync(string start)
    {
        TableState state = await _page.SettledAsync();
        ControlState button = state.Controls.FirstOrDefault(c => c.Kind == "button" && c.Label.StartsWith(start, StringComparison.Ordinal))
            ?? throw new Xunit.Sdk.XunitException($"No button starting '{start}'. There are: {string.Join(", ", state.Controls.Where(c => c.Kind == "button").Select(c => c.Label))}");
        await _page.Page.Mouse.ClickAsync(button.X, button.Y);
    }

    // --- What the client shows ---

    /// <summary>A <c>Client.</c> line judged against the page; null when it holds, else what was wrong.</summary>
    private async Task<string?> JudgeAsync(ScenarioCheck check, TableState state)
    {
        string[] parts = check.Key.Split('.', 4);
        if (parts.Length < 3 || parts[1] != "Bot1")
        {
            return $"line {check.Line}: {check.Key} is not something the page for Bot1 can say (Client.Bot1.<what>)";
        }

        string what = string.Join('.', parts.Skip(2));

        // What the face of one card says: Client.Bot1.Battlefield.Firdoch Core.TypeLine. A card in exile or a graveyard is
        // only drawn while that zone is open, so the zone is opened to read it, as a player opens it to look.
        Match card = CardKey.Match(what);
        string? shown;
        if (card.Success)
        {
            string row = card.Groups["row"].Value;
            string name = card.Groups["card"].Value;
            string attribute = card.Groups["attr"].Value;
            if (row is "Exile" or "Graveyard")
            {
                shown = await ReadInZoneAsync(row, name, attribute);
            }
            else
            {
                string place = row switch { "OpponentBattlefield" => "opponent-battlefield", _ => row.ToLowerInvariant() };
                CardState? found = state.Cards.FirstOrDefault(c => c.Place == place && c.Name == name);
                shown = found is null
                    ? $"(no {name} in {row}: {string.Join("; ", state.Cards.Where(c => c.Place == place).Select(c => c.Name))})"
                    : Face(found, attribute);
            }
        }
        else
        {
            // A list's .Count, as the dump's lists have one.
            shown = what.EndsWith(".Count", StringComparison.Ordinal) && Read(what[..^".Count".Length], state) is { } list
                ? list.Split(';', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries).Length.ToString(System.Globalization.CultureInfo.InvariantCulture)
                : Read(what, state);

            // The button the last choice of cards was confirmed with.
            if (what == "Confirm")
            {
                shown = _lastConfirm;
            }

            // A line of the game log, by its whole text: the value if the log has it, nothing if not.
            if (what == "Log.Has")
            {
                shown = state.LogText?.Contains(check.Value) == true ? check.Value : string.Empty;
            }
        }

        if (shown is null)
        {
            return $"line {check.Line}: {check.Key} is not a client key: {ClientKeys}";
        }

        return Same(shown, check.Value) ? null : $"line {check.Line}: {check.Key} expected \"{check.Value}\", the page shows \"{shown}\"" +
            (what == "Log.Has" ? "; its log ends: " + string.Join(" | ", (state.LogText ?? []).TakeLast(8)) : string.Empty);
    }

    private static readonly Regex CardKey = new(
        @"^(?<row>Stack|Hand|Battlefield|OpponentBattlefield|Exile|Graveyard)\.(?<card>.+)\.(?<attr>TypeLine|Rules|Status|Annotation)$",
        RegexOptions.CultureInvariant);

    /// <summary>One thing a card's face says, with its lines on one line.</summary>
    private static string Face(CardState card, string attribute) => attribute switch
    {
        "TypeLine" => card.TypeLine,
        "Rules" => card.Rules,
        "Status" => card.Status,
        _ => card.Annotation.ReplaceLineEndings(" / "),
    };

    /// <summary>
    /// What a card's face says in a zone drawn only while it is open: Bot1's exile (the button over the piles) or graveyard
    /// (the pile), opened as a player opens it and shut again after.
    /// </summary>
    private async Task<string> ReadInZoneAsync(string zone, string name, string attribute)
    {
        TableState state = await _page.SettledAsync();
        string place = zone == "Exile" ? "choice" : "graveyard";
        (float X, float Y)? opener = ZoneOpener(state, zone);
        if (opener is null)
        {
            return $"(no {zone} to open)";
        }

        await _page.Page.Mouse.ClickAsync(opener.Value.X, opener.Value.Y);
        TableState open = await _page.SettledAsync();
        CardState? found = open.Cards.FirstOrDefault(c => c.Place == place && c.Visible && c.Name == name);
        string shown = found is null
            ? $"(no {name} in the open {zone}: {string.Join("; ", open.Cards.Where(c => c.Place == place).Select(c => c.Name))})"
            : Face(found, attribute);

        // Shut again, where the zone's opener now is.
        (float X, float Y)? closer = ZoneOpener(open, zone);
        if (closer is not null)
        {
            await _page.Page.Mouse.ClickAsync(closer.Value.X, closer.Value.Y);
            await _page.SettledAsync();
        }

        return shown;
    }

    /// <summary>Where a click opens Bot1's exile or graveyard: the Exile button, or the graveyard pile.</summary>
    private static (float X, float Y)? ZoneOpener(TableState state, string zone)
    {
        PlayerState? me = state.Players?.FirstOrDefault(p => p.Seat == state.Seat);
        if (zone == "Exile")
        {
            ZoneButtonState? button = me?.Zones?.FirstOrDefault(z => z.Text.StartsWith("Exile", StringComparison.Ordinal));
            return button is null ? null : (button.X, button.Y);
        }

        return me is { GraveyardX: > 0 } ? (me.GraveyardX, me.GraveyardY) : null;
    }

    private const string ClientKeys =
        "Stack, Hand, Battlefield, OpponentBattlefield, OpponentHand (the cards known in it), each with .Count; <Row>.<Card>.TypeLine, .Rules, .Status or .Annotation (rows Stack, Hand, Battlefield, OpponentBattlefield, Exile, Graveyard); Facts, OpponentFacts (beside the life total); Exile.Lit, .Caption, .Tooltip and OpponentExile.*; Graveyard.Lit, .Note; Button.<label> (shown or absent), Arrows (the latest resolution's targets), Over, Prompt.Title, Prompt.Detail, Prompt.Modes, Log.Has (a line of the game log, by its text), Confirm (the button the last choice of cards was confirmed with)";

    private static string? Read(string what, TableState state)
    {
        string Names(string place) => string.Join("; ", state.Cards.Where(c => c.Place == place).Select(c => c.Name));
        PlayerState? me = state.Players?.FirstOrDefault(p => p.Seat == state.Seat);
        PlayerState? them = state.Players?.FirstOrDefault(p => p.Seat != state.Seat);
        static ZoneButtonState? Exile(PlayerState? player) => player?.Zones?.FirstOrDefault(z => z.Text.StartsWith("Exile", StringComparison.Ordinal));
        return what switch
        {
            "Facts" => string.Join("; ", me?.Facts ?? []),
            "OpponentFacts" => string.Join("; ", them?.Facts ?? []),
            "Exile.Lit" => (Exile(me)?.Lit ?? false) ? "true" : "false",
            "Exile.Caption" => Exile(me)?.Text ?? string.Empty,
            "Exile.Tooltip" => Exile(me)?.Tooltip ?? string.Empty,
            "OpponentExile.Lit" => (Exile(them)?.Lit ?? false) ? "true" : "false",
            "OpponentExile.Caption" => Exile(them)?.Text ?? string.Empty,
            "OpponentExile.Tooltip" => Exile(them)?.Tooltip ?? string.Empty,
            "Graveyard.Lit" => (me?.GraveyardLit ?? false) ? "true" : "false",
            "Graveyard.Note" => me?.GraveyardNote ?? string.Empty,
            "Stack" => Names("stack"),
            "Hand" => Names("hand"),
            "Battlefield" => Names("battlefield"),
            "OpponentBattlefield" => Names("opponent-battlefield"),
            "OpponentHand" => string.Join("; ", state.Cards.Where(c => c.Place == string.Empty && c.Row == string.Empty && c.Name.Length > 0 && c.Handle == 0).Select(c => c.Name)),
            "Arrows" => state.Effects is { Count: > 0 } effects ? string.Join("; ", effects[^1].Arrows.Select(a => a.To)) : string.Empty,
            "Over" => state.Over ? "true" : "false",
            // A Room's doors as its face shows them: `Battlefield.<Room name>.Doors` is "Unholy Annex:unlocked; Ritual Chamber:locked", and
            // `.Actions` what the card offers to do, as its menu words it. A Room with no door unlocked has no name: `Battlefield..Doors`.
            _ when RoomKey(what, state) is { } shown => shown,
            "Prompt.Title" => state.Title,
            "Prompt.Detail" => state.Detail,
            "Prompt.Modes" => state.Modes is { } modes ? string.Join("; ", modes) : string.Empty,
            _ when what.StartsWith("Button.", StringComparison.Ordinal) =>
                state.Controls.Any(c => c.Kind == "button" && c.Label.Contains(what["Button.".Length..], StringComparison.Ordinal)) ? "shown" : "absent",
            _ => null,
        };
    }

    private static string? RoomKey(string what, TableState state)
    {
        foreach ((string prefix, string place) in new[] { ("Battlefield.", "battlefield"), ("OpponentBattlefield.", "opponent-battlefield") })
        {
            foreach (string field in new[] { "Doors", "Actions" })
            {
                string suffix = "." + field;
                if (what.StartsWith(prefix, StringComparison.Ordinal) && what.EndsWith(suffix, StringComparison.Ordinal) && what.Length >= prefix.Length + suffix.Length)
                {
                    string name = what[prefix.Length..^suffix.Length];
                    CardState? card = state.Cards.FirstOrDefault(c => c.Place == place && c.Name == name);
                    return card is null ? null : field == "Doors" ? card.Doors : card.Actions;
                }
            }
        }

        return null;
    }

    /// <summary>Lists compare as sets, as the dump's hand and battlefield do; the value names the empty list by nothing.</summary>
    private static bool Same(string shown, string expected)
    {
        static List<string> Items(string text) => [.. text.Split(';', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries).Order(StringComparer.Ordinal)];
        return Items(shown).SequenceEqual(Items(expected), StringComparer.Ordinal);
    }
}

/// <summary>An answer this driver cannot click its way to yet, naming it and what the page offered.</summary>
public sealed class NotDrivenException(string what, TableState state)
    : Exception($"The browser driver cannot click its way to {what} yet. The page offers: {string.Join(", ", state.Controls.Select(c => $"{c.Kind} '{c.Label}'"))}");
