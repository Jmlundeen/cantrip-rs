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
                if (Judge(check, state) is { } wrong)
                {
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
                    if (Judge(check, state) is { } wrong)
                    {
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
                if (after.RequestId == state.RequestId)
                {
                    ControlState? confirm = after.Controls.FirstOrDefault(c => c.Kind == "button" && c.Label.EndsWith($" {chosen.Cards.Count}", StringComparison.Ordinal));
                    if (confirm is not null)
                    {
                        await _page.Page.Mouse.ClickAsync(confirm.X, confirm.Y);
                    }
                    else
                    {
                        await _page.Page.Keyboard.PressAsync("Space");
                    }
                }

                return;
            }

            case TargetsChosenAnswer targets:
                foreach (TargetMessage target in targets.Targets)
                {
                    await ClickTargetAsync(target);
                }

                return;

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

            case ActionKindView.CancelCasting:
                await _page.ClickAsync("Cancel", "button");
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

            case ActionKindView.PlayLand or ActionKindView.CastSpell or ActionKindView.ActivateAbility or ActionKindView.TurnFaceUp:
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
        ControlState? row = action.Color is { } color
            ? rows.FirstOrDefault(r => r.Label.Contains($"{{{Symbol(color)}}}", StringComparison.Ordinal))
            : rows.Count == 1 ? rows[0] : null;
        if (row is null)
        {
            throw new NotDrivenException($"which row of the menu is {action.Kind}{(action.Color is { } c ? " " + c : string.Empty)}", state);
        }

        await _page.Page.Mouse.ClickAsync(row.X, row.Y);
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

    private async Task ClickHandleAsync(int handle)
    {
        TableState state = await _page.SettledAsync();
        CardState card = state.Cards.FirstOrDefault(c => c.Visible && c.Handle == handle)
            ?? throw new Xunit.Sdk.XunitException(
                $"Handle {handle} is not in view. In view: {string.Join(", ", state.Cards.Where(c => c.Visible).Select(c => $"{c.Name} #{c.Handle} ({c.Place})"))}");
        await _page.Page.Mouse.ClickAsync(card.X, card.Y);
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
    private static string? Judge(ScenarioCheck check, TableState state)
    {
        string[] parts = check.Key.Split('.', 4);
        if (parts.Length < 3 || parts[1] != "Bot1")
        {
            return $"line {check.Line}: {check.Key} is not something the page for Bot1 can say (Client.Bot1.<what>)";
        }

        string what = string.Join('.', parts.Skip(2));
        string? shown = Read(what, state);
        if (shown is null)
        {
            return $"line {check.Line}: {check.Key} is not a client key: {ClientKeys}";
        }

        return Same(shown, check.Value) ? null : $"line {check.Line}: {check.Key} expected \"{check.Value}\", the page shows \"{shown}\"";
    }

    private const string ClientKeys =
        "Stack, Hand, Battlefield, OpponentBattlefield, OpponentHand (the cards known in it), Button.<label> (shown or absent), Arrows (the latest resolution's targets), Over";

    private static string? Read(string what, TableState state)
    {
        string Names(string place) => string.Join("; ", state.Cards.Where(c => c.Place == place).Select(c => c.Name));
        return what switch
        {
            "Stack" => Names("stack"),
            "Hand" => Names("hand"),
            "Battlefield" => Names("battlefield"),
            "OpponentBattlefield" => Names("opponent-battlefield"),
            "OpponentHand" => string.Join("; ", state.Cards.Where(c => c.Place == string.Empty && c.Row == string.Empty && c.Name.Length > 0 && c.Handle == 0).Select(c => c.Name)),
            "Arrows" => state.Effects is { Count: > 0 } effects ? string.Join("; ", effects[^1].Arrows.Select(a => a.To)) : string.Empty,
            "Over" => state.Over ? "true" : "false",
            _ when what.StartsWith("Button.", StringComparison.Ordinal) =>
                state.Controls.Any(c => c.Kind == "button" && c.Label.Contains(what["Button.".Length..], StringComparison.Ordinal)) ? "shown" : "absent",
            _ => null,
        };
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
