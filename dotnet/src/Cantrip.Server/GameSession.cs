using Cantrip.Core;
using Cantrip.Protocol;
using Cantrip.Server.Rules;

namespace Cantrip.Server;

/// <summary>
/// One game, with its clients.
/// <para>
/// The authority boundary. Clients send answers and receive views; everything else — what is legal, what the
/// rules do, what each player may know — happens on this side. A client could be hostile or simply stale and
/// the worst it can achieve is an <see cref="ErrorMessage"/>.
/// </para>
/// <para>
/// Transport-agnostic on purpose: it moves messages into and out of per-seat mailboxes and never touches a
/// socket. The same session therefore serves a dedicated server and a host-and-play client with no second code
/// path, which matters because a second path is a second set of bugs that only appear in one of the two modes.
/// </para>
/// <para>
/// Engine-agnostic too: the rules are an <see cref="IRulesHost"/>, which hands over views, events and prompts already
/// projected for each seat, so nothing here knows what a game state is. The native engine's sidecar is the host, and a
/// test may stand a double in its place.
/// </para>
/// </summary>
public sealed class GameSession : IDisposable
{
    private readonly IRulesHost _rules;
    private readonly List<PlayerId> _seats;
    private readonly Dictionary<PlayerId, Queue<ServerMessage>> _outbox = [];

    /// <summary>
    /// Requests published to clients, with the message each was sent as.
    /// <para>
    /// The message is kept, not just the seat, so a rejected answer can be met with the same prompt again. Without that
    /// the seat deadlocks: the server still holds the request, but the client cleared its copy when it answered, so
    /// neither side will ever move.
    /// </para>
    /// </summary>
    private readonly Dictionary<int, (PlayerId Seat, PromptMessage Message)> _published = [];

    /// <summary>How far through the rules' log every seat has been brought. One cursor, because all seats are
    /// published together.</summary>
    private long _sequence;
    private RulesStep? _latestStep;

    /// <summary>The game log for the whole table, kept for the whole game so any seat can be sent all of it again.</summary>
    private readonly PlayerLog _log = new();

    /// <summary>Each seat's deck as the game began, for its welcome: what it owned in the opening position.</summary>
    private readonly Dictionary<PlayerId, List<DeckCardView>> _decklists = [];

    /// <summary>
    /// The cards each seat has been sent as printed, in the order sent: its deck with its welcome, and every other card
    /// the first time it could see it. Sent again whole when the seat is brought back up to date.
    /// </summary>
    private readonly Dictionary<PlayerId, (HashSet<string> Ids, List<ObjectView> Cards)> _printed = [];

    /// <summary>
    /// A scenario's part of each step, for whatever drives the seats (docs/scenarios-in-the-browser.md); never sent to a
    /// client. Raised as the step is published, before its messages go out.
    /// </summary>
    public event Action<ScenarioProgress>? ScenarioProgressed;

    /// <summary>A session over whichever rules engine <paramref name="rules"/> is.</summary>
    public GameSession(IRulesHost rules)
    {
        ArgumentNullException.ThrowIfNull(rules);
        _rules = rules;
        _seats = [.. rules.Seats.Select(seat => new PlayerId(seat))];

        foreach (PlayerId id in _seats)
        {
            _outbox[id] = new Queue<ServerMessage>();
            _decklists[id] = [.. rules.Decklist(id.Value)];

            // A seat's own deck comes with its welcome, so it is never sent again as newly seen.
            _printed[id] = ([.. _decklists[id].Select(entry => entry.Card.CardId)], [.. _decklists[id].Select(entry => entry.Card)]);
        }
    }

    /// <summary>
    /// The oracle of each card in a view that the seat is seeing for the first time this game, noted as sent. A card is
    /// one the seat may identify: a face-down card it may not look at carries no card id, and is not in it.
    /// </summary>
    private List<ObjectView> NewlySeen(PlayerId seat, PlayerView view)
    {
        (HashSet<string> ids, List<ObjectView> cards) = _printed[seat];
        List<ObjectView> fresh = [];
        foreach (ObjectView seen in view.Objects)
        {
            if (seen.CardId.Length > 0 && ids.Add(seen.CardId) && _rules.Oracle(seen.CardId) is { } printed)
            {
                fresh.Add(printed);
                cards.Add(printed);
            }
        }

        return fresh;
    }

    private WelcomeMessage Welcome(PlayerId seat) => new()
    {
        Seat = seat.Value,
        PlayerNames = [.. _rules.SeatNames],
        Decklist = _decklists.TryGetValue(seat, out List<DeckCardView>? deck) ? deck : [],
    };

    /// <summary>The rules this session is played by.</summary>
    public IRulesHost Rules => _rules;

    public IReadOnlyList<PlayerId> Seats => _seats;

    public bool IsOver => _rules.IsOver || _forfeited;

    /// <summary>The turn the game is on, as the last position published says; zero before the first.</summary>
    public int Turn => AnyView?.Turn ?? 0;

    /// <summary>The step the game is in, as the last position published says.</summary>
    public TurnStepView Step => AnyView?.Step ?? TurnStepView.Untap;

    /// <summary>
    /// The seats that won, in seat order: the winning team's, once the game has been won by its rules, or none.
    /// </summary>
    public IReadOnlyList<int> WinningSeats => _rules.Outcome == GameOutcomeView.TeamWon
        ? [.. _rules.Seats.Where(seat => _rules.Team(seat) == _rules.WinningTeam).Order()]
        : [];

    /// <summary>Any seat's view of the last position published: what every seat sees of the turn and step.</summary>
    private PlayerView? AnyView => _latestStep?.Views.Values.FirstOrDefault();

    /// <summary>Ended from outside the rules - a match clock run out - rather than by the game itself.</summary>
    private bool _forfeited;

    /// <summary>The seats the game is waiting on: every seat with a question published and not yet answered.</summary>
    public IReadOnlyCollection<int> WaitingOn => [.. _published.Values.Select(published => published.Seat.Value).Distinct()];

    /// <summary>
    /// Ends the game from outside the rules, as a match clock that has run out does: the seats named win - one seat's
    /// team, or nobody for a draw - every seat is told the game is over, and nothing more is asked.
    /// </summary>
    public void Forfeit(IReadOnlyList<int> winners)
    {
        ArgumentNullException.ThrowIfNull(winners);
        _forfeited = true;
        _published.Clear();
        bool won = winners.Count > 0;
        int team = won ? _rules.Team(winners[0]) : 0;
        foreach (PlayerId id in Seats)
        {
            _outbox[id].Enqueue(new GameOverMessage { Outcome = won ? GameOutcomeView.TeamWon : GameOutcomeView.Draw, WinningTeam = team });
        }
    }

    /// <summary>Whether a seat can concede now: only when that ends the game (<see cref="IRulesHost.CanConcede"/>).</summary>
    public bool CanConcede(PlayerId seat) => !_forfeited && _latestStep is not null && _rules.CanConcede(seat.Value);

    /// <summary>
    /// A seat concedes (CR 104.3a): the game ends with everyone else winning, and every seat is sent the last position,
    /// the log's word that it was conceded, and that the game is over. Whatever was being asked is asked no more.
    /// </summary>
    public void Concede(PlayerId seat)
    {
        if (!CanConcede(seat))
        {
            throw new InvalidOperationException($"Seat {seat.Value} cannot concede now.");
        }

        RulesStep step = _rules.Concede(seat.Value);
        _published.Clear();
        Publish(step);
    }

    /// <summary>Seats everyone, starts the game, and publishes the opening position.</summary>
    public void Start()
    {
        foreach (PlayerId id in Seats)
        {
            _outbox[id].Enqueue(Welcome(id));
        }

        Publish(_rules.Start());
    }

    /// <summary>Takes everything queued for a seat.</summary>
    public IReadOnlyList<ServerMessage> Drain(PlayerId seat)
    {
        if (!_outbox.TryGetValue(seat, out Queue<ServerMessage>? queue) || queue.Count == 0)
        {
            return [];
        }

        List<ServerMessage> messages = [.. queue];
        queue.Clear();
        return messages;
    }

    /// <summary>Replaces an authenticated seat's undelivered mailbox with its current authorized view and choices.
    /// No rules step is taken and no historical animation is replayed.</summary>
    public void Resynchronize(PlayerId seat)
    {
        if (_latestStep is null || !_outbox.TryGetValue(seat, out Queue<ServerMessage>? queue))
        {
            throw new InvalidOperationException("Only a started session's member can be resynchronized.");
        }

        queue.Clear();
        queue.Enqueue(Welcome(seat));
        PlayerView view = _rules.View(seat.Value);
        ObjectFingerprint.Stamp(view);
        NewlySeen(seat, view);
        queue.Enqueue(new StateMessage
        {
            View = view,

            // Every card sent as printed so far, since a client brought back up to date may have lost them all.
            Printed = [.. _printed[seat].Cards],
        });
        queue.Enqueue(new EventsMessage { Sequence = _sequence });

        // The whole log, not just what is new: whatever this client had, and whatever it missed while it was away.
        queue.Enqueue(new LogMessage { Entries = [.. _log.For(seat.Value)], Complete = true });

        foreach ((PlayerId owner, PromptMessage prompt) in _published.Values)
        {
            if (owner == seat)
            {
                queue.Enqueue(prompt);
            }
        }

        if (IsOver)
        {
            queue.Enqueue(new GameOverMessage { Outcome = _rules.Outcome, WinningTeam = _rules.WinningTeam });
        }
    }

    /// <summary>
    /// Handles one message from a client.
    /// <para>
    /// Every rejection path returns an error to that client and leaves the game untouched. A client cannot
    /// desynchronise the game by sending nonsense, only by being ignored.
    /// </para>
    /// </summary>
    public void Submit(PlayerId from, ClientMessage message)
    {
        ArgumentNullException.ThrowIfNull(message);

        switch (message)
        {
            case JoinMessage join:
                if (join.ProtocolVersion != PlayerView.ProtocolVersion)
                {
                    Reject(from, ErrorCode.ProtocolMismatch,
                        $"server speaks protocol {PlayerView.ProtocolVersion}, client speaks {join.ProtocolVersion}.");
                }

                break;

            case AnswerMessage answer:
                Answer(from, answer);
                break;

            case ConcedeMessage when CanConcede(from):
                Concede(from);
                break;

            case ConcedeMessage:
                Reject(from, ErrorCode.IllegalAnswer, "a concession is taken only when it ends the game.");
                break;

            default:
                Reject(from, ErrorCode.IllegalAnswer, $"unrecognised message {message.GetType().Name}.");
                break;
        }
    }

    private void Answer(PlayerId from, AnswerMessage message)
    {
        if (IsOver)
        {
            Reject(from, ErrorCode.StaleReply, "the game is over.");
            return;
        }

        if (!_published.TryGetValue(message.RequestId, out (PlayerId Seat, PromptMessage Message) outstanding))
        {
            // Routine after a reconnect: the client is answering something already resolved. Nothing to re-ask, because
            // whatever was being asked has moved on.
            Reject(from, ErrorCode.StaleReply, $"request {message.RequestId} is not outstanding.");
            return;
        }

        if (outstanding.Seat != from)
        {
            Reject(
                from,
                ErrorCode.NotYourTurnToAnswer,
                $"request {message.RequestId} belongs to seat {outstanding.Seat.Value}.");

            return;
        }

        RulesStep step;
        try
        {
            step = _rules.Answer(message.RequestId, from.Value, message.Answer);
        }
        catch (RulesRejectedException refused)
        {
            // The rules are the authority on legality, so their refusal is reported rather than pre-empted by a
            // second, possibly disagreeing, check out here. The question still stands, so it is asked again — unless
            // the reply was stale, when there is nothing left to ask.
            Reject(from, refused.Code, refused.Detail);
            if (refused.Code != ErrorCode.StaleReply)
            {
                Reprompt(from, outstanding.Message);
            }

            return;
        }

        _published.Remove(message.RequestId);
        Publish(step);
    }

    /// <summary>
    /// Asks again after refusing an answer.
    /// <para>
    /// A refusal leaves the request outstanding on this side while the client has already discarded its copy, so
    /// without this the seat would simply stop answering and the game would stall — which is how a rejection shows up
    /// in practice: not as an error a player can act on, but as a game that quietly stops.
    /// </para>
    /// <para>
    /// The identical message goes back out. Nothing changed when the answer was refused, so every handle in it still
    /// resolves, and re-deriving it could only introduce a difference from what the client was first shown.
    /// </para>
    /// </summary>
    private void Reprompt(PlayerId seat, PromptMessage prompt)
    {
        if (_outbox.TryGetValue(seat, out Queue<ServerMessage>? queue))
        {
            queue.Enqueue(prompt);
        }
    }

    /// <summary>
    /// Publishes the position to every seat, then any prompt to the seat that owes it.
    /// <para>
    /// The view goes out first so a client always has the state its prompt refers to, and the handles in a
    /// prompt always resolve against a view it has already received.
    /// </para>
    /// </summary>
    private void Publish(RulesStep step)
    {
        _latestStep = step;
        if (step.Scenario is { } progress)
        {
            ScenarioProgressed?.Invoke(progress);
        }

        foreach (PlayerId id in Seats)
        {
            PlayerView view = step.Views[id.Value];

            // Each object stamped with a digest of itself, so a client can tell what is unchanged since the last view
            // without working it out for every object again.
            ObjectFingerprint.Stamp(view);
            _outbox[id].Enqueue(new StateMessage { View = view, Printed = NewlySeen(id, view) });

            IReadOnlyList<(long Sequence, GameEventView View)> events =
                step.Events.TryGetValue(id.Value, out IReadOnlyList<(long Sequence, GameEventView View)>? batch) ? batch : [];
            if (events.Count > 0)
            {
                _outbox[id].Enqueue(new EventsMessage
                {
                    Events = [.. events.Select(e => e.View)],
                    Sequences = [.. events.Select(e => e.Sequence)],
                    Sequence = step.Sequence,
                });
            }

            // The log's lines for these events, written now from the view and events this seat was just sent, so
            // they name exactly what the seat could see.
            PlayerLog.Update written = _log.Write(id.Value, view, events);
            if (!written.IsEmpty)
            {
                _outbox[id].Enqueue(new LogMessage { Entries = [.. written.Added], Removed = [.. written.Removed] });
            }
        }

        _sequence = step.Sequence;

        foreach (RulesRequest request in step.Requests)
        {
            // Only what has not already gone out. The rules report every outstanding request, not just new ones,
            // so when several are outstanding at once — each defending player declaring blocks, for instance — every
            // answer would otherwise re-send the prompts still waiting. The client would answer the duplicate and be
            // told its reply was stale, through no fault of its own.
            if (_published.ContainsKey(request.RequestId))
            {
                continue;
            }

            var seat = new PlayerId(request.Seat);
            var prompt = new PromptMessage
            {
                RequestId = request.RequestId,
                Prompt = request.Prompt,
                MayCancel = request.MayCancel,
                Presentation = request.Presentation,
            };

            _published[request.RequestId] = (seat, prompt);
            _outbox[seat].Enqueue(prompt);
        }

        if (step.Finished)
        {
            foreach (PlayerId id in Seats)
            {
                _outbox[id].Enqueue(new GameOverMessage
                {
                    Outcome = _rules.Outcome,
                    WinningTeam = _rules.WinningTeam,
                });
            }
        }
    }

    private void Reject(PlayerId seat, ErrorCode code, string detail)
    {
        if (_outbox.TryGetValue(seat, out Queue<ServerMessage>? queue))
        {
            queue.Enqueue(new ErrorMessage { Code = code, Detail = detail });
        }
    }

    /// <summary>Lets the rules go: the sidecar's game is closed.</summary>
    public void Dispose() => _rules.Dispose();
}
