using Cantrip.Protocol;

namespace Cantrip.Client;

/// <summary>
/// A client's whole understanding of a game: the last view it was sent, and whatever it has been asked.
/// <para>
/// Thin in the literal sense. It holds no rules knowledge, computes nothing about legality, and never predicts
/// what an action will do. Everything it displays came from the server, and everything it offers came from a
/// prompt. The one thing it is allowed to decide is which of the offered answers to send.
/// </para>
/// <para>
/// Free of any UI, so the client's behaviour can be tested without a browser or a window. The renderer reads these
/// properties; it does not interpret them.
/// </para>
/// </summary>
public sealed class GameClient
{
    private readonly IGameTransport _transport;
    private readonly List<ErrorMessage> _errors = [];

    public GameClient(IGameTransport transport)
    {
        ArgumentNullException.ThrowIfNull(transport);
        _transport = transport;
    }

    /// <summary>Which seat this client plays, or zero before the welcome arrives.</summary>
    public int Seat { get; private set; }

    public IReadOnlyList<string> PlayerNames { get; private set; } = [];

    /// <summary>This seat's own deck as the game began, each card with how many and as printed; empty until welcomed.</summary>
    public IReadOnlyList<DeckCardView> Decklist { get; private set; } = [];

    private readonly Dictionary<string, ObjectView> _printed = new(StringComparer.Ordinal);

    /// <summary>
    /// Every card this client has been sent as printed - its oracle - by card id: its own deck, and each card it has seen
    /// in the game. Kept for the game.
    /// </summary>
    public IReadOnlyDictionary<string, ObjectView> Printed => _printed;

    /// <summary>A card was sent as printed for the first time: one this client had not seen before.</summary>
    public event Action<ObjectView>? PrintedSeen;

    private void NotePrinted(ObjectView card)
    {
        if (card.CardId.Length > 0 && _printed.TryAdd(card.CardId, card))
        {
            PrintedSeen?.Invoke(card);
        }
    }

    /// <summary>Private proof for this seat, retained across transport loss. Never expose in the table UI or logs.</summary>
    public MembershipMessage? Membership { get; private set; }

    public TableMessage? Table { get; private set; }

    /// <summary>
    /// Why this seat was given up - the player left a forming table, or did not say ready in time - once it has been;
    /// the seat is gone, and is not to be taken back.
    /// </summary>
    public SeatReleasedMessage? Released { get; private set; }

    /// <summary>Where this seat has moved to - another table, and the ticket for it - once it has.</summary>
    public TableMoveMessage? Moved { get; private set; }

    /// <summary>The draft as this seat last saw it, while the table is drafting; null otherwise.</summary>
    public DraftMessage? Draft { get; private set; }

    /// <summary>Every seat's match clock as last sent, or null for a game with none.</summary>
    public ClockMessage? Clock { get; private set; }

    /// <summary>When <see cref="Clock"/> arrived, so a running clock can be counted down from it.</summary>
    public DateTime ClockReceived { get; private set; }

    /// <summary>How long a seat has left on its match clock now, counting a running one down; null with no clock.</summary>
    public (TimeSpan Left, bool Running)? ClockOf(int seat)
    {
        if (Clock?.Clocks.FirstOrDefault(clock => clock.Seat == seat) is not { } clock) return null;
        TimeSpan left = TimeSpan.FromMilliseconds(clock.LeftMs) - (clock.Running ? DateTime.UtcNow - ClockReceived : TimeSpan.Zero);
        return (left < TimeSpan.Zero ? TimeSpan.Zero : left, clock.Running);
    }

    /// <summary>An event's pairings and standings as this seat last saw them, while its table shows them; null otherwise.</summary>
    public PairingsMessage? Pairings { get; private set; }

    /// <summary>The deck this seat is building, as the server last had it, while decks are built; null otherwise.</summary>
    public DeckBuildingMessage? DeckBuilding { get; private set; }

    /// <summary>The most recent position, or null before the first one arrives.</summary>
    public PlayerView? View { get; private set; }

    /// <summary>What the server is waiting on from this client, or null when it is not this client's turn to answer.</summary>
    public PromptMessage? Prompt { get; private set; }

    public GameOverMessage? Outcome { get; private set; }

    /// <summary>
    /// What happened since the last pump, oldest first. Replaced each pump rather than accumulated: this is what a
    /// renderer animates, and holding a whole game's history would be a leak with no reader.
    /// </summary>
    public IReadOnlyList<GameEventView> RecentEvents { get; private set; } = [];

    private readonly Dictionary<GameEventView, long> _sequences = new(ReferenceEqualityComparer.Instance);

    /// <summary>
    /// The server's log sequence for one of <see cref="RecentEvents"/>, or -1 for an event not in the latest batch. What
    /// an <see cref="EventsRetractedEvent"/> names events by.
    /// </summary>
    public long SequenceOf(GameEventView entry) => _sequences.TryGetValue(entry, out long sequence) ? sequence : -1;

    /// <summary>How far through the server's log this client has been brought.</summary>
    public long EventSequence { get; private set; }

    public bool IsOver => Outcome is not null;

    /// <summary>
    /// Rejections from the server, newest last. Kept rather than thrown: a stale reply being refused is a
    /// normal consequence of reconnecting, not a fault.
    /// </summary>
    public IReadOnlyList<ErrorMessage> Errors => _errors;

    /// <summary>True when the server has asked something this client has not yet answered.</summary>
    public bool IsWaitingOnMe => Prompt is not null;

    /// <summary>
    /// Applies everything the transport has received. Called once per frame by the renderer.
    /// </summary>
    /// <returns>How many messages were applied, so a caller can tell whether anything changed.</returns>
    public int Pump()
    {
        IReadOnlyList<ServerMessage> messages = _transport.Receive();

        List<GameEventView>? events = null;
        _sequences.Clear();

        foreach (ServerMessage message in messages)
        {
            if (message is EventsMessage batch)
            {
                // Batches from one pump are concatenated, so a renderer that draws once per frame still sees
                // everything in order rather than only the last batch.
                events ??= [];
                events.AddRange(batch.Events);
                EventSequence = batch.Sequence;

                for (int i = 0; i < batch.Events.Count && i < batch.Sequences.Count; i++)
                {
                    _sequences[batch.Events[i]] = batch.Sequences[i];
                }

                continue;
            }

            Apply(message);
        }

        RecentEvents = events ?? [];
        return messages.Count;
    }

    /// <summary>Whether the transport still has a connection. Always true for a game hosted in this process.</summary>
    public bool IsConnected => _transport.IsConnected;

    /// <summary>Asks for a seat. A <paramref name="ticket"/> from the host claims the seat it was issued for.</summary>
    public void Join(string playerName, int preferredSeat = 0, string? ticket = null) =>
        _transport.Send(new JoinMessage { PlayerName = playerName, PreferredSeat = preferredSeat, Ticket = ticket ?? string.Empty });

    public void Resume(MembershipMessage membership)
    {
        ArgumentNullException.ThrowIfNull(membership);
        _transport.Send(new ResumeMessage { MembershipId = membership.MembershipId, ReconnectProof = membership.ReconnectProof });
    }

    public void ConfigurePlayer(string name, string deck) =>
        _transport.Send(new ConfigurePlayerMessage { Name = name, Deck = deck });

    public void ConfigureTable(IEnumerable<string> sets) =>
        _transport.Send(new ConfigureTableMessage { Sets = [.. sets] });

    public void SetReady(bool ready) => _transport.Send(new ReadyMessage { Ready = ready });

    /// <summary>Takes or passes the pile this seat is looking at in the draft shown.</summary>
    public void Pick(bool take)
    {
        if (Draft is not { } draft || draft.Active != Seat)
        {
            throw new InvalidOperationException("It is not this seat's turn to pick.");
        }

        _transport.Send(new DraftPickMessage { Decision = draft.Decision, Take = take });
    }

    /// <summary>
    /// Takes the card at <paramref name="place"/> in the pack in front of this seat, in a booster draft, and with it the
    /// cards at <paramref name="also"/>: each an additional card, paid for by a card of the pool going into the pack
    /// (<see cref="DraftMessage.AdditionalPicks"/>).
    /// </summary>
    public void PickCard(int place, IReadOnlyList<int>? also = null)
    {
        if (Draft is not { } draft || draft.Pile.Count == 0)
        {
            throw new InvalidOperationException("There is no pack in front of this seat.");
        }

        if (place < 0 || place >= draft.Pile.Count)
        {
            throw new ArgumentOutOfRangeException(nameof(place), "That card is not in the pack.");
        }

        also ??= [];
        if (also.Count > draft.AdditionalPicks.Count || also.Any(other => other < 0 || other >= draft.Pile.Count || other == place) || also.Distinct().Count() != also.Count)
        {
            throw new ArgumentOutOfRangeException(nameof(also), "Those cards cannot be taken with the pick.");
        }

        _transport.Send(new DraftPickMessage { Decision = draft.Decision, Take = true, Card = place, Also = [.. also] });
    }

    /// <summary>
    /// Takes a row (0 to 2, from the top) or a column (3 to 5, from the left) of the grid in a grid draft, on this seat's
    /// pick.
    /// </summary>
    public void PickLine(int line)
    {
        if (Draft is not { Format: "grid" } draft || draft.Active != Seat)
        {
            throw new InvalidOperationException("It is not this seat's pick from the grid.");
        }

        if (line is < 0 or > 5)
        {
            throw new ArgumentOutOfRangeException(nameof(line), "A line is a row or a column of the grid.");
        }

        _transport.Send(new DraftPickMessage { Decision = draft.Decision, Take = true, Card = line });
    }

    /// <summary>
    /// Answers a booster pick that is due (<see cref="DraftMessage.PickDue"/>) with no card chosen, so the server takes one
    /// at random without waiting any longer.
    /// </summary>
    public void PickNothing()
    {
        if (Draft is not { } draft || draft.Pile.Count == 0)
        {
            throw new InvalidOperationException("There is no pack in front of this seat.");
        }

        _transport.Send(new DraftPickMessage { Decision = draft.Decision, Take = false });
    }

    /// <summary>
    /// Sends this seat's deck as it now stands - the places in its pool of the main deck's cards and the basic lands
    /// added - and with <paramref name="submit"/> says it is the deck to play.
    /// </summary>
    public void BuildDeck(IEnumerable<int> main, IReadOnlyDictionary<string, int> basics, bool submit)
    {
        ArgumentNullException.ThrowIfNull(main);
        ArgumentNullException.ThrowIfNull(basics);
        if (DeckBuilding is not { } building)
        {
            throw new InvalidOperationException("No deck is being built.");
        }

        _transport.Send(new BuildDeckMessage
        {
            Change = ++DeckChangesSent,
            Main = [.. main],
            Basics = [.. basics.Select(land => new BasicLandCount { Name = land.Key, Count = land.Value })],
            Submit = submit,
        });
    }

    /// <summary>
    /// How many deck changes this client has sent. A deck view whose <see cref="DeckBuildingMessage.Change"/> is below
    /// it is from before the latest of them, and shows a deck this client has already changed.
    /// </summary>
    public int DeckChangesSent { get; private set; }

    /// <summary>Whether every deck change this client has sent has come back in the deck view.</summary>
    public bool DeckCaughtUp => DeckBuilding is not { } building || building.Change >= DeckChangesSent;

    /// <summary>Explicit departure revokes membership; unlike Disconnect, it cannot be resumed.</summary>
    public void Leave() => _transport.Send(new LeaveMessage());

    /// <summary>Concedes the game being played: this seat loses it, and it is over.</summary>
    public void Concede() => _transport.Send(new ConcedeMessage());

    /// <summary>Drops from the event this table is part of: this seat is paired no more.</summary>
    public void Drop() => _transport.Send(new DropMessage());

    /// <summary>
    /// Drops only the connection. Membership remains reserved for the server's reconnect grace period.
    /// </summary>
    public void Disconnect()
    {
        Prompt = null;
        _transport.Dispose();
    }

    /// <summary>Answers the outstanding prompt and clears it, so the same answer cannot be sent twice.</summary>
    public void Answer(AnswerPayload answer)
    {
        ArgumentNullException.ThrowIfNull(answer);

        if (Prompt is null)
        {
            throw new InvalidOperationException("Nothing has been asked.");
        }

        var message = new AnswerMessage { RequestId = Prompt.RequestId, Answer = answer };

        // Cleared before sending. With a loopback transport the send runs the engine synchronously and can
        // publish the next prompt inside this call, which would otherwise be overwritten on the way out.
        Prompt = null;
        _transport.Send(message);
    }

    public void Pass() => Answer(new PassAnswer());

    public void Discard(IEnumerable<ViewHandle> cards) =>
        Answer(new CardsChosenAnswer { Cards = [.. cards] });

    // --- Reading the view. Convenience only: no interpretation, no rules. ---

    public PlayerSummaryView? Me =>
        View?.Players.FirstOrDefault(p => p.Seat == Seat);

    public IEnumerable<PlayerSummaryView> Opponents =>
        View?.Players.Where(p => p.Seat != Seat) ?? [];

    public ObjectView? Object(ViewHandle handle)
    {
        if (View is not { } view)
        {
            return null;
        }

        // Indexed once per view rather than searched per call: a library search looks up every one of hundreds of cards,
        // and a search per card made that hundreds of thousands of comparisons. The first object under a handle wins, as
        // the search it replaces found it.
        Index(view);
        return _byHandle.GetValueOrDefault(handle);
    }

    private readonly Dictionary<ViewHandle, ObjectView> _byHandle = [];
    private readonly Dictionary<(ZoneKindView Kind, int Owner), IReadOnlyList<ObjectView>> _byZone = [];
    private PlayerView? _indexed;

    /// <summary>
    /// Puts back, in a view just received, every object the last view had unchanged - the same handle with the same
    /// fingerprint (<see cref="ObjectView.Fingerprint"/>) - so that whatever was worked out from an object and kept with it
    /// holds for as long as the object does not change, rather than for one view. A board of two hundred permanents where
    /// one card tapped is one new object, not two hundred.
    /// </summary>
    private void KeepUnchanged(PlayerView next)
    {
        if (View is not { } last)
        {
            return;
        }

        Index(last);
        for (int i = 0; i < next.Objects.Count; i++)
        {
            ObjectView received = next.Objects[i];
            if (received.Fingerprint != 0
                && _byHandle.TryGetValue(received.Handle, out ObjectView? kept)
                && kept.Fingerprint == received.Fingerprint)
            {
                next.Objects[i] = kept;
            }
        }
    }

    /// <summary>Starts this view's indexes over when it is a new view: a view is never changed once received.</summary>
    private void Index(PlayerView view)
    {
        if (ReferenceEquals(_indexed, view))
        {
            return;
        }

        _byHandle.Clear();
        _byZone.Clear();
        foreach (ObjectView item in view.Objects)
        {
            _byHandle.TryAdd(item.Handle, item);
        }

        _indexed = view;
    }

    /// <summary>
    /// Cards in a zone, in the order the server sent them. Empty for a zone this client cannot see into.
    /// <para>
    /// Listed once per view and handed out again after that: a redraw asks for the battlefield once for every permanent
    /// on it, and a list per ask made a board of two hundred permanents cost forty thousand lookups a redraw.
    /// </para>
    /// </summary>
    public IReadOnlyList<ObjectView> Zone(ZoneKindView kind, int owner)
    {
        if (View is not { } view)
        {
            return [];
        }

        Index(view);
        if (_byZone.TryGetValue((kind, owner), out IReadOnlyList<ObjectView>? listed))
        {
            return listed;
        }

        ZoneView? zone = view.Zones.FirstOrDefault(z => z.Kind == kind && z.Owner == owner);
        List<ObjectView> contents = [];
        if (zone is { ContentsVisible: true })
        {
            foreach (ViewHandle handle in zone.Contents)
            {
                if (_byHandle.GetValueOrDefault(handle) is { } card)
                {
                    contents.Add(card);
                }
            }
        }

        _byZone[(kind, owner)] = contents;
        return contents;
    }

    /// <summary>
    /// Cards this client knows in a zone it cannot see into, such as what is left of a hand Duress revealed. A hand's come
    /// in name order, a library's top first. Empty for a zone it can see into, where <see cref="Zone"/> has everything.
    /// </summary>
    public IReadOnlyList<KnownCardView> KnownCards(ZoneKindView kind, int owner) =>
        View?.Zones.FirstOrDefault(z => z.Kind == kind && z.Owner == owner)?.Known ?? [];

    /// <summary>
    /// Every card this client can name in a zone it cannot see into: the ones it remembers and the ones it is being
    /// shown this moment, in name order so nothing about the order says which card is which. Right for a hand, where
    /// the cards are not held by position; a library's are, and <see cref="Shown"/> says which of those is its top.
    /// <para>
    /// The two arrive by different channels and a card crosses between them as a reveal comes and goes: while it is
    /// being revealed it is in the view with a handle, and from then on it is remembered without one. Reading only one
    /// of them turns a card face down for exactly the message that announces it was revealed.
    /// </para>
    /// </summary>
    public IReadOnlyList<ObjectView> NamedIn(ZoneKindView kind, int owner) =>
        [.. KnownCards(kind, owner).Select(known => known.Card).Concat(Shown(kind, owner))
            .OrderBy(card => card.Name, StringComparer.Ordinal)
            .ThenBy(card => card.CardId, StringComparer.Ordinal)];

    /// <summary>
    /// Cards this client is being shown, with their handles, in a zone it cannot see into: a library being searched, or
    /// the top of a library under Bolas's Citadel. In id order, which says nothing about where each one is. Empty for a
    /// zone it can see into.
    /// </summary>
    public IReadOnlyList<ObjectView> Shown(ZoneKindView kind, int owner) =>
        View is null || View.Zones.FirstOrDefault(z => z.Kind == kind && z.Owner == owner) is not { ContentsVisible: false }
            ? []
            : [.. View.Objects.Where(o => o.Zone == kind && o.ZoneOwner == owner)];

    /// <summary>How many cards are in a zone, whether or not this client can see into it.</summary>
    public int ZoneCount(ZoneKindView kind, int owner) =>
        View?.Zones.FirstOrDefault(z => z.Kind == kind && z.Owner == owner)?.Count ?? 0;

    public IReadOnlyList<ObjectView> MyHand() => Zone(ZoneKindView.Hand, Seat);

    public IReadOnlyList<ObjectView> Battlefield() => Zone(ZoneKindView.Battlefield, 0);

    public IEnumerable<ObjectView> BattlefieldOf(int seat) =>
        Battlefield().Where(o => o.Controller == seat);

    /// <summary>
    /// The game log as the server has written it for this seat, oldest first. The server keeps it and sends all of it
    /// again when this client is seated or resumed, so it is complete however long this client was away.
    /// </summary>
    public IReadOnlyList<LogEntryView> Log => _log;

    /// <summary>Bumped whenever lines are taken away or replaced rather than added, so a view that appends knows to start over.</summary>
    public int LogRevision { get; private set; }

    private readonly List<LogEntryView> _log = [];

    private void ApplyLog(LogMessage log)
    {
        if (log.Complete)
        {
            _log.Clear();
            LogRevision++;
        }

        if (log.Removed.Count > 0)
        {
            HashSet<long> gone = [.. log.Removed];
            if (_log.RemoveAll(entry => gone.Contains(entry.Number)) > 0)
            {
                LogRevision++;
            }
        }

        _log.AddRange(log.Entries);
    }

    private void Apply(ServerMessage message)
    {
        switch (message)
        {
            case MembershipMessage membership:
                Membership = membership;
                Seat = membership.Seat;
                break;

            case TableMessage table:
                Table = table;
                if (table.Phase != TablePhase.Drafting)
                {
                    Draft = null;
                }

                if (table.Phase != TablePhase.DeckBuilding)
                {
                    DeckBuilding = null;
                }

                if (table.Phase != TablePhase.Pairings)
                {
                    Pairings = null;
                }

                if (table.Phase == TablePhase.Abandoned)
                {
                    Prompt = null;
                }
                break;

            case WelcomeMessage welcome:
                Seat = welcome.Seat;
                PlayerNames = welcome.PlayerNames;
                Decklist = welcome.Decklist;
                foreach (DeckCardView entry in welcome.Decklist)
                {
                    NotePrinted(entry.Card);
                }

                break;

            case StateMessage state:
                foreach (ObjectView printed in state.Printed)
                {
                    NotePrinted(printed);
                }

                KeepUnchanged(state.View);
                View = state.View;
                break;

            case PromptMessage prompt:
                if (Table?.Phase != TablePhase.Abandoned) Prompt = prompt;
                break;

            case GameOverMessage over:
                Outcome = over;
                Prompt = null;
                break;

            case ErrorMessage error:
                _errors.Add(error);
                break;

            case LogMessage log:
                ApplyLog(log);
                break;

            case DraftMessage draft:
                Draft = draft;
                break;

            case SeatReleasedMessage released:
                Released = released;
                Prompt = null;
                break;

            case TableMoveMessage moved:
                Moved = moved;
                Prompt = null;
                break;

            case DeckBuildingMessage building:
                DeckBuilding = building;
                break;

            case PairingsMessage pairings:
                Pairings = pairings;
                break;

            case ClockMessage clock:
                Clock = clock;
                ClockReceived = DateTime.UtcNow;
                break;

            default:
                // An unrecognised message from a newer server is not a reason to fall over; the handshake is
                // what catches a real mismatch.
                break;
        }
    }
}
