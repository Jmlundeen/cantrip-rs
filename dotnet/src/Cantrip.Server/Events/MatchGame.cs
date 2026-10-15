using Cantrip.Protocol;
using Cantrip.Server.Rules;

namespace Cantrip.Server.Events;

/// <summary>
/// One game of a match of a <see cref="BoosterPod"/>, at a table of its own: the match's two players, seated by tickets
/// the pod gives them when it sends them here. The first game is dealt as soon as the table opens - whoever is not there
/// yet is brought up to date when they sit down - and every later one after both players have sideboarded, or their
/// time to has run out.
/// <para>
/// When the game is over its players are shown the result for a moment, then sent on: to the next game's table, or back
/// to the pod's. A player who does not come, or is gone too long, forfeits the match (<see cref="BoosterPod.Forfeited"/>).
/// </para>
/// <para>
/// All of its state is the pod's, kept under the pod's lock; the table is only somewhere to play it.
/// </para>
/// </summary>
public sealed class MatchGame : ITableEvent
{
    private readonly BoosterPod _pod;
    private readonly Dictionary<int, List<string>> _tickets = new() { [1] = [], [2] = [] };
    private readonly DateTimeOffset?[] _firstSeen = new DateTimeOffset?[3];
    private readonly DateTimeOffset?[] _awaySince = new DateTimeOffset?[3];
    private readonly Dictionary<int, int> _changes = [];
    private readonly List<(int Seat, ServerMessage Farewell)> _vacated = [];
    private int _version;
    private DateTimeOffset _opened;
    private DateTimeOffset? _endedAt;
    private bool _dealt;
    private bool _ended;

    internal MatchGame(BoosterPod pod, MatchRecord match, int number)
    {
        _pod = pod;
        Match = match;
        Number = number;
        Key = $"{match.Id}g{number}";
        SideboardDeadline = number > 1 ? pod.Now + pod.Timing.Sideboarding : null;
    }

    internal MatchRecord Match { get; }

    /// <summary>Which game of the match this is, from 1.</summary>
    public int Number { get; }

    /// <summary>The match and game, as the pod names the table: "r2t3g1".</summary>
    public string Key { get; }

    /// <summary>Where the table is, once the host has opened it.</summary>
    public Uri? Address { get; private set; }

    /// <summary>Whether everyone has gone on from this table, and the host can close it.</summary>
    public bool Done { get; private set; }

    private DateTimeOffset? SideboardDeadline { get; set; }

    private bool Sideboarding => Number > 1 && !_dealt && !_ended && Match.Players.Any(player => !Match.Decks[player].Submitted);

    /// <summary>The host has opened the table here: the game's players are sent to it.</summary>
    public void Opened(Uri address)
    {
        ArgumentNullException.ThrowIfNull(address);
        lock (_pod.Gate)
        {
            Address = address;
            _opened = _pod.Now;
            if (SideboardDeadline is not null)
            {
                SideboardDeadline = _opened + _pod.Timing.Sideboarding;
            }

            _pod.GameOpened(this);
        }
    }

    /// <summary>A ticket to this table for one of its players.</summary>
    internal string IssueTicket(string playerId)
    {
        int seat = Match.Players.IndexOf(playerId) + 1;
        string ticket = BoosterPod.NewTicket();
        _tickets[seat].Add(EventStore.HashTicket(ticket));
        return ticket;
    }

    /// <summary>The event is called off: nothing more happens here.</summary>
    internal void Close()
    {
        _ended = true;
        Done = true;
    }

    internal void Changed() => _version++;

    // --- The table. -------------------------------------------------------------------------------------------------

    public string EventId => _pod.EventId;

    public string Name
    {
        get
        {
            lock (_pod.Gate)
            {
                return $"Round {Match.Round}, game {Number}: {_pod.Record.Participant(Match.Players[0])!.Name} vs {_pod.Record.Participant(Match.Players[1])!.Name}";
            }
        }
    }

    public int Seats => 2;

    public TablePhase Phase
    {
        get
        {
            lock (_pod.Gate)
            {
                if (_ended) return TablePhase.Finished;
                return Sideboarding ? TablePhase.DeckBuilding : TablePhase.Setup;
            }
        }
    }

    public int SeatFor(string ticket)
    {
        if (string.IsNullOrEmpty(ticket)) return 0;
        lock (_pod.Gate) return BoosterPod.Matching(ticket, _tickets.Select(pair => (pair.Key, (IEnumerable<string>)pair.Value)));
    }

    public string NameAt(int seat)
    {
        lock (_pod.Gate) return seat is 1 or 2 ? _pod.Record.Participant(Match.Players[seat - 1])!.Name : $"Seat {seat}";
    }

    public string PlayerAt(int seat)
    {
        lock (_pod.Gate) return seat is 1 or 2 ? Match.Players[seat - 1] : string.Empty;
    }

    public void Seated(int seat)
    {
        lock (_pod.Gate)
        {
            if (seat is 1 or 2) _firstSeen[seat] ??= _pod.Now;
        }
    }

    public void Connected(int seat, bool connected)
    {
        lock (_pod.Gate)
        {
            if (seat is not (1 or 2)) return;
            _awaySince[seat] = connected ? null : _pod.Now;
            if (connected) _firstSeen[seat] ??= _pod.Now;
        }
    }

    public int Version
    {
        get { lock (_pod.Gate) return _version; }
    }

    public ServerMessage? ViewFor(int seat)
    {
        lock (_pod.Gate)
        {
            if (!Sideboarding || seat is not (1 or 2)) return null;
            string player = Match.Players[seat - 1];
            EventParticipant me = _pod.Record.Participant(player)!;
            int mine = Swiss.GamesOf(Match, player).Won;
            int theirs = Swiss.GamesOf(Match, Match.Players[2 - seat]).Won;
            string standing = mine == theirs ? $"the match is {mine}-{theirs}" : mine > theirs ? $"you lead {mine}-{theirs}" : $"you trail {mine}-{theirs}";
            return _pod.Deck(me, Match.Decks[player], _changes.GetValueOrDefault(seat), SideboardDeadline, $"Sideboarding for game {Number}: {standing}",
                [.. Match.Players.Select((each, index) => new DeckBuilderSeatView
                {
                    Seat = index + 1, Name = _pod.Record.Participant(each)!.Name, Submitted = Match.Decks[each].Submitted,
                })]);
        }
    }

    public string? Receive(int seat, ClientMessage message)
    {
        lock (_pod.Gate)
        {
            if (message is not BuildDeckMessage build) return "That is not something to do at this table.";
            if (!Sideboarding || seat is not (1 or 2)) return "Decks are not being changed now.";
            string player = Match.Players[seat - 1];
            EventParticipant me = _pod.Record.Participant(player)!;
            _changes[seat] = build.Change;
            (EventDeck? deck, string? refusal) = PoolDecks.Built(Match.Decks[player], me.Pool.Count, build, _pod.BasicCards, _pod.Record.MinimumDeck);
            if (deck is null) return refusal;
            Match.Decks[player] = deck;
            _version++;
            _pod.Save();
            return null;
        }
    }

    public bool ReadyToDeal
    {
        get
        {
            lock (_pod.Gate) return Address is not null && !_dealt && !_ended && !Match.Complete && !Sideboarding;
        }
    }

    public IRulesHost Deal(IReadOnlyList<string> names, IRulesHostFactory? rules = null)
    {
        lock (_pod.Gate)
        {
            // A game cut off by a restart is dealt again from its own seed; a new one is recorded as it is dealt.
            GameRecord? record = Match.Games.FirstOrDefault(game => game.Number == Number);
            ulong seed = record?.Seed ?? _pod.SeedFor(Match, Number);
            IRulesHost game = _pod.NewGame(Match, seed, names, rules);
            if (record is null)
            {
                Match.Games.Add(new GameRecord { Number = Number, Seed = seed, Started = _pod.Now });
            }

            _dealt = true;
            Match.Detail = $"Game {Number} is being played.";
            _pod.Changed();
            _pod.Save();
            return game;
        }
    }

    public void DealFailed(string why)
    {
        lock (_pod.Gate)
        {
            // Nothing the players can change at this table would deal it: the match is ended without a result.
            _ended = true;
            _endedAt = _pod.Now;
            _pod.GameFinished(this, [], abandoned: true, $"{DraftPod.DealFailedLine}: {why}");
        }
    }

    public void GameEnded(bool abandoned, IReadOnlyList<int> winningSeats, string detail)
    {
        lock (_pod.Gate)
        {
            if (_ended) return;
            _ended = true;
            _endedAt = _pod.Now;
            _pod.GameFinished(this, [.. winningSeats.Where(seat => seat is 1 or 2).Select(seat => Match.Players[seat - 1])], abandoned, detail);
        }
    }

    public void Tick(DateTimeOffset now)
    {
        lock (_pod.Gate)
        {
            if (Done || Address is null) return;

            if (!_ended)
            {
                if (Sideboarding && SideboardDeadline is { } deadline && now >= deadline)
                {
                    foreach (string player in Match.Players.Where(player => !Match.Decks[player].Submitted))
                    {
                        Match.Decks[player] = PoolDecks.Submitted(_pod.Record.Participant(player)!.Pool, Match.Decks[player], _pod.Record.MinimumDeck, _pod.ColorsOfCard);
                    }

                    _version++;
                    _pod.Save();
                }

                List<string> absent = [.. Enumerable.Range(1, 2)
                    .Where(seat => _firstSeen[seat] is null ? now - _opened >= _pod.Timing.NoShow : _awaySince[seat] is { } away && now - away >= _pod.Timing.Away)
                    .Select(seat => Match.Players[seat - 1])];
                if (absent.Count > 0)
                {
                    _ended = true;
                    _endedAt = now - _pod.Timing.AfterGame;
                    _pod.Forfeited(this, absent);
                }

                return;
            }

            if (_endedAt is { } ended && now - ended >= _pod.Timing.AfterGame)
            {
                SendOn();
            }
        }
    }

    public TimeSpan? ClockAtDeal(int seat)
    {
        lock (_pod.Gate)
        {
            if (_pod.Timing.MatchClock <= TimeSpan.Zero || seat is not (1 or 2)) return null;
            return Match.ClockMs.TryGetValue(Match.Players[seat - 1], out long left) ? TimeSpan.FromMilliseconds(left) : _pod.Timing.MatchClock;
        }
    }

    public void ClockUsed(int seat, TimeSpan left)
    {
        lock (_pod.Gate)
        {
            if (seat is not (1 or 2)) return;
            Match.ClockMs[Match.Players[seat - 1]] = Math.Max(0, (long)left.TotalMilliseconds);
            _pod.Save();
        }
    }

    public void ClockRanOut(int seat)
    {
        lock (_pod.Gate)
        {
            if (_ended || seat is not (1 or 2)) return;
            _ended = true;
            _endedAt = _pod.Now;
            _pod.TimedOut(this, Match.Players[seat - 1]);
        }
    }

    public IReadOnlyList<(int Seat, ServerMessage Farewell)> TakeVacated()
    {
        lock (_pod.Gate)
        {
            if (_vacated.Count == 0) return [];
            List<(int, ServerMessage)> vacated = [.. _vacated];
            _vacated.Clear();
            return vacated;
        }
    }

    /// <summary>
    /// The game is over and its result has been shown: both players go on to the next game's table once it is open, or
    /// back to the pod's when the match is decided.
    /// </summary>
    private void SendOn()
    {
        MatchGame? next = null;
        if (!Match.Complete)
        {
            next = _pod.Next(this);
            if (next?.Address is null)
            {
                // Not open yet: they wait here, with the result in front of them.
                return;
            }
        }

        for (int seat = 1; seat <= 2; seat++)
        {
            string player = Match.Players[seat - 1];
            TableMoveMessage? move = next is not null
                ? new TableMoveMessage { Server = next.Address!.AbsoluteUri, Ticket = next.IssueTicket(player) }
                : _pod.BackToPod(player);
            if (move is not null)
            {
                _vacated.Add((seat, move));
            }
        }

        Done = true;
        _version++;
    }
}
