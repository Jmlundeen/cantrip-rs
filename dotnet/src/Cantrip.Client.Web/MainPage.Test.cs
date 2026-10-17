using System.Runtime.InteropServices.JavaScript;
using System.Text;
using System.Text.Json;
using Cantrip.Client;
using Cantrip.Client.Presentation;
using Cantrip.Protocol;
using Microsoft.UI.Xaml.Media;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// What a browser driven by Playwright, rather than a person, reads and does: the web client's tests
/// (Cantrip.Client.Web.Tests), and anyone picturing a scenario. Not a player feature, and none of it runs unless the
/// page is asked for it.
/// <para>
/// The table is drawn on canvases, so there is no DOM to find a card or a button in. <c>?test</c> publishes the page's
/// state as JSON on <c>globalThis.cantripTest</c> instead, ten times a second: whether the table is up, whether it is
/// still settling (faces being made, art arriving, cards in the air), whether the game is over, and where every card
/// and control is on the page, by name - which is what lets a test click a Forest or the Take button with real mouse
/// events without knowing the layout. <c>?auto</c> plays this seat itself, the one deterministic way, or at random
/// from <c>?bot-seed=N</c>, or as <c>?bot=tactical</c> says.
/// </para>
/// </summary>
public sealed partial class MainPage
{
    private IAutoPlayer? _selfPlayer;
    private DispatcherTimer? _testTimer;

    /// <summary>Answers this seat's question for it, under <c>?auto</c>.</summary>
    private void PlayMyself(GameClient client)
    {
        if (!ClientConfig.Auto || client.IsOver)
        {
            return;
        }

        _selfDrafter ??= new DraftPlay();
        if (_selfDrafter.Act(client))
        {
            _needsRedraw = true;
            return;
        }

        _selfPlayer ??= ClientConfig.NewBot(self: true);
        if (_selfPlayer.Act(client))
        {
            _needsRedraw = true;
        }
    }

    /// <summary>Starts publishing the page's state, under <c>?test</c>.</summary>
    private void StartTestState()
    {
        if (!ClientConfig.Test || _testTimer is not null)
        {
            return;
        }

        _testTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(100) };
        _testTimer.Tick += (_, _) => JSHost.GlobalThis.SetProperty("cantripTest", TestState());
        _testTimer.Start();
    }

    /// <summary>How long after the last face was made the page counts as settled.</summary>
    private static readonly TimeSpan FacesSettle = TimeSpan.FromMilliseconds(400);

    private string TestState()
    {
        using var buffer = new MemoryStream();
        using (var json = new Utf8JsonWriter(buffer))
        {
            GameClient? client = _gameClient;
            // Ready once the table is up, or the draft or the deck being built before it.
            // A table forming from a queue, and a seat given up, are pages of their own too.
            bool ready = (client?.View is not null || EventShown || PairingsShown || client?.Table is { Forming: true } || client?.Released is not null)
                && _deckLoading.Visibility != Visibility.Visible;

            // What the page is still doing, by name, so a test that waited for it to finish can say what never did.
            List<string> busy = [];
            if (CardArt.Arriving)
            {
                busy.Add("art");
            }

            if (DateTime.UtcNow - Faces.LastDrawn < FacesSettle)
            {
                busy.Add("faces");
            }

            if (_flights.Flying.Count > 0)
            {
                busy.Add("flights");
            }

            if (_resolutions.Showing)
            {
                busy.Add("effects");
            }

            if (_selectionOverlay.Retirements > 0)
            {
                busy.Add("arrows");
            }

            if (CardBoards().Any(pair => pair.Board.Moving))
            {
                busy.Add("slides");
            }

            if (_needsRedraw)
            {
                busy.Add("redraw");
            }

            json.WriteStartObject();
            json.WriteBoolean("ready", ready);
            json.WriteBoolean("busy", busy.Count > 0);
            json.WriteString("doing", string.Join(" ", busy));
            json.WriteBoolean("over", client?.IsOver ?? false);
            json.WriteString("phase", client?.Released is not null ? "Released" : client?.Table is { Forming: true } ? "Forming" : client?.Table?.Phase.ToString() ?? string.Empty);
            json.WriteString("prompt", client?.Prompt?.Prompt?.GetType().Name ?? string.Empty);
            json.WriteNumber("requestId", client?.Prompt?.RequestId ?? 0);
            json.WriteNumber("seat", client?.Seat ?? 0);
            json.WriteString("fixture", _fixture ? _fixtureName : string.Empty);
            json.WriteString("answered", _fixture ? Answered(_fixtureTransport) : string.Empty);
            json.WriteString("art", ClientConfig.ArtTemplate);
            json.WriteString("outcome", client?.Outcome is { } outcome ? $"{outcome.Outcome} {outcome.WinningTeam}" : string.Empty);
            json.WriteNumber("flown", _flights.Launched);
            json.WriteNumber("selected", _pending.Count);
            json.WriteBoolean("graveyardsOpen", _graveyardsOpen);
            json.WriteString("title", _promptTitle.Text);
            json.WriteString("detail", _promptDetail.Text);
            json.WriteStartArray("logText");
            foreach (LogLine line in _journal.Lines)
            {
                json.WriteStringValue(line.Text);
            }

            json.WriteEndArray();
            json.WriteStartArray("modes");
            if (client?.Prompt?.Prompt is ChooseModesPromptMessage modesAsked)
            {
                foreach (string mode in modesAsked.Modes)
                {
                    json.WriteStringValue(mode);
                }
            }

            json.WriteEndArray();

            // What the dock says (its question and the line under it) and the buttons it offers, in order, each with whether it can
            // be pressed: a scenario's client checks read the wording a player is given (docs/scenarios-in-the-browser.md).
            json.WriteStartArray("dock");
            foreach (DockButton button in _buttons.Children.OfType<DockButton>())
            {
                if (button.Visibility != Visibility.Visible)
                {
                    continue;
                }

                json.WriteStartObject();
                json.WriteString("label", Microsoft.UI.Xaml.Automation.AutomationProperties.GetName(button) is { Length: > 0 } named ? named : string.Join(" ", Texts(button)));
                json.WriteBoolean("enabled", button.Enabled);
                json.WriteEndObject();
            }

            json.WriteEndArray();

            // Each ability shown resolving, newest last: the way it cut on the whole, and each arrow's.
            json.WriteStartArray("effects");
            foreach (ResolutionEffectsElement.Effect effect in _resolutions.History)
            {
                json.WriteStartObject();
                json.WriteString("feeling", effect.Overall.ToString());
                json.WriteBoolean("lit", effect.Origin is null);
                json.WriteStartArray("arrows");
                foreach ((ResolutionEffectsElement.Mark to, ResolutionEffectsElement.Feeling feeling) in effect.Arrows)
                {
                    json.WriteStartObject();
                    json.WriteString("to", to.IsPlayer ? $"player {to.Player}" : client?.Object(to.Card)?.Name ?? string.Empty);
                    json.WriteString("feeling", feeling.ToString());
                    json.WriteEndObject();
                }

                json.WriteEndArray();
                json.WriteEndObject();
            }

            json.WriteEndArray();

            // An event's pairings, as shown: the round, whether they are final, and each match in words.
            json.WriteStartObject("pairings");
            json.WriteNumber("round", client?.Pairings?.Round ?? 0);
            json.WriteBoolean("final", client?.Pairings?.Final ?? false);
            json.WriteNumber("startsIn", client?.Pairings?.StartsInMs ?? 0);
            json.WriteStartArray("matches");
            foreach (PairingView match in client?.Pairings?.Matches ?? [])
            {
                json.WriteStringValue($"{match.Table}: {match.First} {match.FirstWins}-{match.SecondWins} {match.Second}: {match.Status}");
            }

            json.WriteEndArray();
            json.WriteEndObject();
            json.WriteNumber("held", _inspector.FacesHeld);
            json.WriteString("tooltip", _tooltip.Showing);

            // The log's lines, and how many are made as controls: only those in view should be.
            json.WriteStartObject("log");
            json.WriteNumber("lines", _log.Extent.Lines);
            json.WriteNumber("shown", _log.Extent.Shown);
            json.WriteEndObject();

            json.WriteStartArray("cards");
            if (ready)
            {
                foreach ((CardBoardElement board, _) in CardBoards())
                {
                    if (board.XamlRoot is null || board.ActualWidth <= 0 || !IsShown(board))
                    {
                        continue;
                    }

                    GeneralTransform toPage = board.TransformToVisual(Root);
                    // An event's rows by name: a booster pack's cards are told from the pool's by it, not by where they are.
                    string row = board == _pileBoard ? "pack" : board == _gridBoard ? "grid" : board == _mainBoard ? "pool" : board == _sideBoard ? "sideboard" : string.Empty;
                    // The table's rows by name, as a scenario's client checks read them (docs/scenarios-in-the-browser.md).
                    string place = board == _stackBoard ? "stack"
                        : board == _handBoard ? "hand"
                        : board == _myHalf.Board ? "battlefield"
                        : board == _opponentHalf.Board ? "opponent-battlefield"
                        : board == _myHalf.Graveyard.Board ? "graveyard"
                        : board == _opponentHalf.Graveyard.Board ? "opponent-graveyard"
                        : board == _choiceBoard ? "choice"
                        : string.Empty;
                    foreach ((ObjectView card, Rect box, bool visible, Point? lower, Point? raise) in board.Targets())
                    {
                        json.WriteStartObject();
                        json.WriteString("name", card.Known || card.FaceDown ? card.Name : string.Empty);
                        json.WriteBoolean("visible", visible);
                        json.WriteString("row", row);
                        json.WriteString("place", place);
                        json.WriteNumber("handle", card.Handle.Value);
                        json.WriteBoolean("tapped", card.Tapped);
                        // A Room's doors as the face draws them, each by name and whether it is locked (CR 709.5c), and what the
                        // card offers its controller to do, as the menu words it (the unlock reads with its cost).
                        json.WriteString("doors", card.UnlockedDoors.Count > 0 && card.UnlockedDoors.Count == card.Halves.Count
                            ? string.Join("; ", card.Halves.Select((half, i) => $"{half.Name}:{(card.UnlockedDoors[i] ? "unlocked" : "locked")}"))
                            : string.Empty);
                        json.WriteString("halves", string.Join("; ", card.Halves.Select(half => half.Name)));
                        json.WriteString("actions", client is null
                            ? string.Empty
                            : string.Join(" | ", _actions.For(card.Handle).Select(option => CardActions.Describe(client, option))));

                        // What the face says, as a test reads it: the type line, the rules lines (granted and lost ones
                        // too), the status words at its foot, and the line written over its art.
                        json.WriteString("typeLine", card.Known && !card.FaceDown ? CardFace.TypeLine(card) : string.Empty);
                        json.WriteString("rules", string.Join(" / ", CardFace.RulesLines(card).Select(line => line.Spent ? $"(grey) {line.Text}" : line.Text)));
                        json.WriteString("status", string.Join(", ", CardFace.StatusWords(card)));
                        json.WriteString("counters", string.Join("; ", CardFace.ShownCounters(card).Select(counter => $"{counter.Count} × {counter.Name}")));
                        json.WriteString("damage", CardFace.MarkedDamage(card));
                        // Written over the art as the face writes it: what the look says, else a permanent's one chosen quality
                        // (a chosen name, color or player).
                        json.WriteString("annotation", _looks.TryGetValue(card.Handle, out CardLook? shown) && shown.Annotation is { Length: > 0 } written
                            ? written
                            : card.Choices.Count == 1 ? card.Choices[0].Value : string.Empty);
                        WriteBox(json, toPage.TransformBounds(box));
                        if (lower is { } down && raise is { } up)
                        {
                            WritePoint(json, "lower", toPage.TransformPoint(down));
                            WritePoint(json, "raise", toPage.TransformPoint(up));
                        }

                        if (_flights.FlewFrom.TryGetValue(card.Handle, out CardFlights.Pose from))
                        {
                            WritePoint(json, "flewFrom", new Point(from.X, from.Y));
                        }

                        json.WriteEndObject();
                    }
                }

                // The cards known in an opponent's hand, face up in the fan in their rail.
                if (_opponentHalf.Rail.HandFan is HandFanElement fan && fan.XamlRoot is not null && IsShown(fan))
                {
                    GeneralTransform toPage = fan.TransformToVisual(Root);
                    foreach ((ObjectView card, Rect box) in fan.Targets())
                    {
                        json.WriteStartObject();
                        json.WriteString("name", card.Name);
                        json.WriteBoolean("visible", true);
                        WriteBox(json, toPage.TransformBounds(box));
                        json.WriteEndObject();
                    }
                }
            }

            json.WriteEndArray();

            // Each player's box, which a click on targets them, and the mana in their pool, which a click spends.
            json.WriteStartArray("players");
            if (ready)
            {
                foreach (PlayerRail rail in (PlayerRail[])[_myHalf.Rail, _opponentHalf.Rail])
                {
                    if (rail.XamlRoot is null || rail.Seat == 0 || rail.Avatar.ActualWidth <= 0)
                    {
                        continue;
                    }

                    json.WriteStartObject();
                    json.WriteNumber("seat", rail.Seat);
                    WriteBox(json, rail.Avatar.TransformToVisual(Root).TransformBounds(new Rect(0, 0, rail.Avatar.ActualWidth, rail.Avatar.ActualHeight)));

                    // The facts beside the life total, and the buttons into exile and a sideboard, with whether they are lit.
                    json.WriteStartArray("facts");
                    foreach (string fact in rail.FactsShown())
                    {
                        json.WriteStringValue(fact);
                    }

                    json.WriteEndArray();
                    json.WriteStartArray("zones");
                    foreach ((string text, string tooltip, bool lit, DockButton button) in rail.ZoneButtonsShown())
                    {
                        if (button.ActualWidth <= 0)
                        {
                            continue;
                        }

                        json.WriteStartObject();
                        json.WriteString("text", text);
                        json.WriteString("tooltip", tooltip);
                        json.WriteBoolean("lit", lit);
                        json.WriteBoolean("open", button.On);
                        WriteBox(json, button.TransformToVisual(Root).TransformBounds(new Rect(0, 0, button.ActualWidth, button.ActualHeight)));
                        json.WriteEndObject();
                    }

                    json.WriteEndArray();
                    json.WriteBoolean("graveyardLit", rail.GraveyardLit);
                    json.WriteString("graveyardNote", rail.GraveyardNote ?? string.Empty);
                    if (rail.GraveyardPile.ActualWidth > 0)
                    {
                        Rect pile = rail.GraveyardPile.TransformToVisual(Root).TransformBounds(new Rect(0, 0, rail.GraveyardPile.ActualWidth, rail.GraveyardPile.ActualHeight));
                        json.WriteNumber("graveyardX", Math.Round(pile.X + (pile.Width / 2), 1));
                        json.WriteNumber("graveyardY", Math.Round(pile.Y + (pile.Height / 2), 1));
                    }

                    json.WriteStartArray("mana");
                    foreach ((PlayerRail.ManaChipSpec spec, FrameworkElement chip) in rail.Chips())
                    {
                        if (chip.ActualWidth <= 0)
                        {
                            continue;
                        }

                        json.WriteStartObject();
                        json.WriteString("type", spec.Type.ToString());
                        json.WriteNumber("count", spec.Count);
                        json.WriteBoolean("clickable", spec.Click is not null);
                        json.WriteString("caption", spec.Caption ?? string.Empty);
                        WriteBox(json, chip.TransformToVisual(Root).TransformBounds(new Rect(0, 0, chip.ActualWidth, chip.ActualHeight)));
                        json.WriteEndObject();
                    }

                    json.WriteEndArray();
                    json.WriteEndObject();
                }
            }

            json.WriteEndArray();

            json.WriteStartArray("controls");
            WriteControls(json, Root, inPopup: false);
            foreach (UIElement popup in Popups())
            {
                WriteControls(json, popup, inPopup: true);
            }

            json.WriteEndArray();

            // The cards named in lines of text - the game log, a draft's picks - by name, where the name is.
            json.WriteStartArray("mentions");
            if (ready)
            {
                WriteMentions(json, Root, clip: null);
            }

            json.WriteEndArray();
            json.WriteEndObject();
        }

        return Encoding.UTF8.GetString(buffer.ToArray());
    }

    /// <summary>
    /// The last answer a gallery fixture's dock sent, which nothing receives: its kind, and for a choice of modes or
    /// options the indexes chosen ("ModesChosenAnswer 42").
    /// </summary>
    private static string Answered(FixtureTransport? transport) =>
        transport?.Sent.OfType<AnswerMessage>().LastOrDefault()?.Answer switch
        {
            null => string.Empty,
            ModesChosenAnswer modes => $"{nameof(ModesChosenAnswer)} {string.Join(",", modes.Modes)}",
            { } answer => answer.GetType().Name,
        };

    /// <summary>Every popup open over the page: a card's menu and a drop-down's list are popups.</summary>
    private IEnumerable<UIElement> Popups()
    {
        if (XamlRoot is not null)
        {
            foreach (Microsoft.UI.Xaml.Controls.Primitives.Popup popup in VisualTreeHelper.GetOpenPopupsForXamlRoot(XamlRoot))
            {
                if (popup.Child is { } child)
                {
                    yield return child;
                }
            }
        }
    }

    /// <summary>
    /// Every control in view under an element, by what it says: a button, a toggle, a drop-down by its caption, a box to
    /// type in by its placeholder, and a row of an open menu or list (<see cref="CardMenu"/>) by its text.
    /// </summary>
    private void WriteControls(Utf8JsonWriter json, DependencyObject under, bool inPopup)
    {
        int count = VisualTreeHelper.GetChildrenCount(under);
        for (int i = 0; i < count; i++)
        {
            if (VisualTreeHelper.GetChild(under, i) is not FrameworkElement child || child.Visibility != Visibility.Visible || child.ActualWidth <= 0)
            {
                continue;
            }

            string? kind = child switch
            {
                DockButton or CollapseButton => "button",
                DockToggle => "toggle",
                Border { Tag: SidePanel.TabTag } => "tab",
                ChoiceDropdown => "dropdown",
                TextBox => "textbox",
                Border { Child: SymbolText } when inPopup => "item",
                _ => null,
            };

            if (kind is not null)
            {
                // A control named for what it does - a "+" that adds a Forest - is known by that name; a drop-down by its
                // caption, its first line; everything else by all it says.
                string label = Microsoft.UI.Xaml.Automation.AutomationProperties.GetName(child) is { Length: > 0 } named
                    ? named
                    : child is TextBox box ? box.PlaceholderText
                    : kind == "dropdown" ? Texts(child).FirstOrDefault() ?? string.Empty : string.Join(" ", Texts(child));
                FrameworkElement target = child is ChoiceDropdown dropdown ? dropdown.Button : child;

                json.WriteStartObject();
                json.WriteString("kind", kind);
                json.WriteString("label", label);
                json.WriteNumber("ability", kind == "item" && child is Border { Tag: int ability } ? ability : -1);
                WriteBox(json, target.TransformToVisual(Root).TransformBounds(new Rect(0, 0, target.ActualWidth, target.ActualHeight)));
                json.WriteEndObject();

                if (kind != "dropdown")
                {
                    continue;
                }
            }

            WriteControls(json, child, inPopup);
        }
    }

    /// <summary>
    /// Every card's name in a line of text under an element (<see cref="CardMention"/>) that the card can be read from,
    /// with whether it is in view: inside what the scrolling around it shows.
    /// </summary>
    private void WriteMentions(Utf8JsonWriter json, DependencyObject under, Rect? clip)
    {
        if (under is ScrollViewer { ActualWidth: > 0 } scroll)
        {
            Rect shown = scroll.TransformToVisual(Root).TransformBounds(new Rect(0, 0, scroll.ActualWidth, scroll.ActualHeight));
            if (clip is { } outer)
            {
                shown.Intersect(outer);
            }

            clip = shown;
        }

        int count = VisualTreeHelper.GetChildrenCount(under);
        for (int i = 0; i < count; i++)
        {
            if (VisualTreeHelper.GetChild(under, i) is not FrameworkElement child || child.Visibility != Visibility.Visible)
            {
                continue;
            }

            if (child is TextBlock { Tag: CardMention { Card: { } card } } name && name.ActualWidth > 0)
            {
                Rect box = name.TransformToVisual(Root).TransformBounds(new Rect(0, 0, name.ActualWidth, name.ActualHeight));
                Rect inside = box;
                if (clip is { } bounds)
                {
                    inside.Intersect(bounds);
                }

                json.WriteStartObject();
                json.WriteString("name", card.Name);
                json.WriteBoolean("visible", !inside.IsEmpty && inside.Height >= box.Height - 0.5);
                WriteBox(json, box);
                json.WriteEndObject();
                continue;
            }

            WriteMentions(json, child, clip);
        }
    }

    /// <summary>The words shown under an element, in order.</summary>
    private static IEnumerable<string> Texts(DependencyObject under)
    {
        if (under is SymbolText symbols)
        {
            yield return symbols.Text;
            yield break;
        }

        if (under is TextBlock { Text.Length: > 0 } text)
        {
            yield return text.Text;
            yield break;
        }

        int count = VisualTreeHelper.GetChildrenCount(under);
        for (int i = 0; i < count; i++)
        {
            foreach (string said in Texts(VisualTreeHelper.GetChild(under, i)))
            {
                yield return said;
            }
        }
    }

    private static void WriteBox(Utf8JsonWriter json, Rect box)
    {
        json.WriteNumber("x", Math.Round(box.X + (box.Width / 2), 1));
        json.WriteNumber("y", Math.Round(box.Y + (box.Height / 2), 1));
        json.WriteNumber("w", Math.Round(box.Width, 1));
        json.WriteNumber("h", Math.Round(box.Height, 1));
    }

    private static void WritePoint(Utf8JsonWriter json, string name, Point at)
    {
        json.WriteStartObject(name);
        json.WriteNumber("x", Math.Round(at.X, 1));
        json.WriteNumber("y", Math.Round(at.Y, 1));
        json.WriteEndObject();
    }
}
