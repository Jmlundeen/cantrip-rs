using Cantrip.Protocol;
using Cantrip.Protocol.InPerson;

namespace Cantrip.Client.Web;

/// <summary>
/// A phone's prompt on the table (docs/in-person.md, "The prompt on the table"): what the dock asks and the buttons that
/// answer it, sent to the relay whenever it changes, for the tablets to show on this player's side of the board and press
/// from there - a press reaching this page as a <see cref="SeatPressMessage"/>, which presses the same button of the dock.
/// At a table in paper what a player does is in plain view: a spell is revealed as it is cast (and may be taken back), a
/// target pointed at, a stop called, priority passed. So the prompt is public, and answered where the board is, with no
/// look back at the phone; only a choice among cards nobody else may see is not, and the tablets are told only that one
/// is being made.
/// </summary>
public sealed partial class MainPage
{
    /// <summary>The version of the prompt last sent, which a press must name.</summary>
    private int _tablePromptVersion;

    /// <summary>What the prompt last sent said, to send it again only when it changes.</summary>
    private string _tablePromptSent = string.Empty;

    /// <summary>What pressing each item of the prompt as it stands does, by its id; null for words.</summary>
    private List<Action?> _tablePromptPresses = [];

    /// <summary>The prompt as last sent, for the page's test state.</summary>
    private PromptCard? _tablePromptShown;

    /// <summary>Every press from a tablet this page acted on, newest last, for the page's test state.</summary>
    private readonly List<(int Version, int Item, bool Used)> _tablePressesTaken = [];

    /// <summary>How many presses from the tablets this page has taken, for the page's test state.</summary>
    private int _tablePressCount;

    /// <summary>A card's menu opened by a tap on a tablet: shown on the tablets, in place of the dock, while its question stands.</summary>
    private sealed record TableMenu(
        int RequestId,
        string Title,
        IReadOnlyList<(string Text, Action OnChosen)> Rows,
        IReadOnlyList<int> Abilities,
        IReadOnlyList<int> OptionIds);

    /// <summary>The menu the tablets are showing for this page, if any.</summary>
    private TableMenu? _tableMenu;

    /// <summary>Whether the click being made is a tap from a tablet (<see cref="TryTableTap"/>).</summary>
    private bool _takingTableTap;

    /// <summary>
    /// Opens a card's menu where the finger that asked for it is. A click on this page opens it here; a tap on a tablet shows
    /// it there, its rows the prompt's buttons, unless it says what the table may not know (a face-down card's real name) or
    /// is more than a prompt can hold, and then it opens here, with the tablets told a choice is being made on the phone.
    /// </summary>
    private void OpenCardMenu(GameClient client, ViewHandle source, List<(string Text, Action OnChosen)> items, List<int> abilities, List<int> optionIds)
    {
        if (_takingTableTap && client.Prompt is { } prompt)
        {
            var menu = new TableMenu(prompt.RequestId, client.Object(source)?.Name ?? string.Empty, items, abilities, optionIds);
            PromptCard card = TableMenuCard(client, menu, []);
            int words = card.Title.Length + card.Detail.Length + card.Rows.SelectMany(row => row.Items).Sum(item => item.Text.Length);
            if (card.Rows.Sum(row => row.Items.Count) <= PromptCard.MostItems && words <= PromptCard.MostText && !NamesHiddenCard(client, card))
            {
                _menu.Close();
                _tableMenu = menu;
                return;
            }
        }

        _menu.Open(Root, _pointer, items, abilities: abilities, optionIds: optionIds);
    }

    /// <summary>A menu as the prompt the tablets show: a button for each row, and Cancel, which is the way out a phone's menu has in a tap beside it.</summary>
    private PromptCard TableMenuCard(GameClient client, TableMenu menu, List<Action?> presses)
    {
        // The panel is short and narrow, so a cost every row shares is not said on each: a land's colours fit side by side.
        int shared = menu.Rows.Count > 1 ? menu.Rows[0].Text.IndexOf(": ", StringComparison.Ordinal) + 2 : 0;
        string cost = shared >= 2 ? menu.Rows[0].Text[..shared] : string.Empty;
        if (!menu.Rows.All(row => row.Text.StartsWith(cost, StringComparison.Ordinal)))
        {
            cost = string.Empty;
        }

        List<PromptItem> rows = [];
        foreach ((string text, Action onChosen) in menu.Rows)
        {
            rows.Add(new PromptItem { Id = presses.Count, Kind = PromptItemKind.Button, Text = text[cost.Length..] });
            presses.Add(() =>
            {
                // A press on its way as the question moved on answers nothing: what it chose was offered by that question.
                bool current = client.Prompt?.RequestId == menu.RequestId;
                _tableMenu = null;
                if (current)
                {
                    onChosen();
                }
            });
        }

        rows.Add(new PromptItem { Id = presses.Count, Kind = PromptItemKind.Button, Text = "Cancel" });
        presses.Add(() =>
        {
            _tableMenu = null;
            _needsRedraw = true;
        });

        return new PromptCard
        {
            Asking = true,
            Title = menu.Title,
            Rows = [new PromptRow { Items = rows }],
        };
    }

    /// <summary>
    /// The dock as the table may see it, sent when it changes: its toggles, its title and detail, an error, Undo and the dock's
    /// buttons and steppers - or, while the choice is among cards only this player may see, only that one is being made.
    /// Called on the page's timer; the presses are taken from the dock as it stands each time, since a redraw makes the
    /// dock's buttons anew without changing what they say.
    /// </summary>
    private void SendTablePrompt()
    {
        if (_phoneLink is null)
        {
            return;
        }

        List<Action?> presses = [];
        PromptCard card = TablePrompt(presses);
        _tablePromptPresses = presses;
        string said = Said(card);
        if (said == _tablePromptSent)
        {
            return;
        }

        _tablePromptSent = said;
        card.Version = ++_tablePromptVersion;
        _tablePromptShown = card;
        _phoneLink.SendPrompt(card);
    }

    /// <summary>An item of the prompt pressed on a tablet: pressed here, if the prompt it was pressed on is still the one shown.</summary>
    private void TakeTablePress(int version, int item)
    {
        bool used = version == _tablePromptVersion && item >= 0 && item < _tablePromptPresses.Count && _tablePromptPresses[item] is not null;
        if (used)
        {
            _tablePromptPresses[item]!();
            RedrawNow();
        }

        _tablePressCount++;
        _tablePressesTaken.Add((version, item, used));
        if (_tablePressesTaken.Count > 64)
        {
            _tablePressesTaken.RemoveAt(0);
        }
    }

    /// <summary>The dock read into a card for the table, each pressable item's action added to <paramref name="presses"/> at its id.</summary>
    private PromptCard TablePrompt(List<Action?> presses)
    {
        if (_tableMenu is { } menu)
        {
            if (_gameClient is { } asked && asked.Prompt?.RequestId == menu.RequestId && !asked.IsOver)
            {
                return TableMenuCard(asked, menu, presses);
            }

            _tableMenu = null;
        }

        var card = new PromptCard();

        // A card's menu open on this page is being answered here, whatever it says.
        bool hidden = _choicesOpen && _choicePrompted || _menu.IsOpen;
        List<PromptItem> row = [];

        void Item(PromptItemKind kind, string text, Action? press, bool primary = false, bool enabled = true, bool on = false)
        {
            row.Add(new PromptItem { Id = presses.Count, Kind = kind, Text = text, Primary = primary, Enabled = enabled, On = on });
            presses.Add(enabled ? press : null);
        }

        void EndRow()
        {
            if (row.Count > 0)
            {
                card.Rows.Add(new PromptRow { Items = row });
                row = [];
            }
        }

        // Auto Pass, Pass Turn and Hold Priority are the player's own way of playing, set on their phone: the tablet's one
        // prompt has room only for what is asked.
        if (_undoButton.Visibility == Visibility.Visible)
        {
            Item(PromptItemKind.Button, "Undo", _undoButton.Press, enabled: _undoButton.Enabled);
        }

        foreach (UIElement child in _dockButtons.Children)
        {
            if (child.Visibility != Visibility.Visible)
            {
                continue;
            }

            switch (child)
            {
                case DockButton button:
                    Item(PromptItemKind.Button, button.Caption, button.Press, button.Primary, button.Enabled, button.On);
                    break;

                case SymbolText words:
                    Item(PromptItemKind.Words, words.Text, null);
                    break;

                case TextBlock words:
                    Item(PromptItemKind.Words, words.Text, null);
                    break;

                // A card named by typing it is a choice among names, which says what this player is thinking of.
                case TextBox:
                    hidden = true;
                    break;

                // A stepper's "-", its value and "+", in a row of their own.
                case Panel stepper:
                    EndRow();
                    foreach (UIElement part in stepper.Children)
                    {
                        switch (part)
                        {
                            case DockButton step:
                                Item(PromptItemKind.Button, step.Caption, step.Press, step.Primary, step.Enabled, step.On);
                                break;
                            case SymbolText value:
                                Item(PromptItemKind.Words, value.Text, null);
                                break;
                            case TextBlock value:
                                Item(PromptItemKind.Words, value.Text, null);
                                break;
                        }
                    }

                    EndRow();
                    break;
            }
        }

        EndRow();
        card.Title = _promptTitle.Text;
        card.Detail = _promptDetail.Visibility == Visibility.Visible ? _promptDetail.Text : string.Empty;
        card.Error = _errorLabel.Visibility == Visibility.Visible ? _errorLabel.Text : string.Empty;

        // Whether the table is waiting on this player: the tablets show the prompt of the player being asked.
        bool asking = _gameClient is { Prompt: not null, IsOver: false };
        card.Asking = asking;
        if (hidden || _gameClient is { } client && NamesHiddenCard(client, card))
        {
            presses.Clear();
            return new PromptCard { Private = true, Asking = asking };
        }

        return card;
    }

    /// <summary>
    /// Whether the prompt's words name a card the table may not know of: one in a hand, a library or a sideboard, or face
    /// down, that not every other player knows and that no card in plain view shares a name with. "Cast Brainstorm for its
    /// miracle cost?" says what was drawn, and is the player's to keep to themselves until they reveal it. The card being
    /// cast or activated - its targets chosen, a cost being paid - is in plain view already, as a spell revealed from a hand
    /// is at a table in paper, and may still be taken back.
    /// </summary>
    private bool NamesHiddenCard(GameClient client, PromptCard card)
    {
        if (client.View is not { } view)
        {
            return false;
        }

        HashSet<ViewHandle> playing = [];
        if (_targeting is { } targeting)
        {
            playing.Add(targeting.Source);
        }

        foreach (ActionOptionMessage costing in _selection.CostChoice ?? [])
        {
            playing.Add(costing.Source);
        }

        HashSet<string> inView = [.. view.Objects
            .Where(each => each.Known && !each.FaceDown && each.Zone is ZoneKindView.Battlefield or ZoneKindView.Stack or ZoneKindView.Graveyard or ZoneKindView.Exile or ZoneKindView.Command)
            .Select(each => each.Name)];
        int others = Math.Max(0, view.Players.Count - 1);
        string words = string.Join("\n", [card.Title, card.Detail, card.Error, .. card.Rows.SelectMany(row => row.Items).Select(item => item.Text)]);
        return view.Objects.Any(each =>
            each.Known
            && each.Name.Length > 0
            && (each.FaceDown || each.Zone is ZoneKindView.Hand or ZoneKindView.Library or ZoneKindView.Sideboard)
            && !playing.Contains(each.Handle)
            && each.KnownBy.Count < others
            && !inView.Contains(each.Name)
            && NamesWord(words, each.Name));
    }

    /// <summary>Whether a name appears in some words as a whole, not inside a longer word ("Opt" in "Options").</summary>
    private static bool NamesWord(string words, string name)
    {
        for (int at = words.IndexOf(name, StringComparison.Ordinal); at >= 0; at = words.IndexOf(name, at + 1, StringComparison.Ordinal))
        {
            bool before = at == 0 || !char.IsLetterOrDigit(words[at - 1]);
            int end = at + name.Length;
            bool after = end >= words.Length || !char.IsLetterOrDigit(words[end]);
            if (before && after)
            {
                return true;
            }
        }

        return false;
    }

    /// <summary>Everything a card says, as one string, to tell whether it changed.</summary>
    private static string Said(PromptCard card) =>
        $"{card.Private}|{card.Asking}\u0001{card.Title}\u0001{card.Detail}\u0001{card.Error}\u0001"
        + string.Join("\u0002", card.Rows.Select(row => string.Join("\u0003", row.Items.Select(item => $"{(int)item.Kind}|{item.Text}|{item.Primary}|{item.Enabled}|{item.On}"))));
}
