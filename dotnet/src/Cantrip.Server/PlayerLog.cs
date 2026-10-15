using Cantrip.Projection;
using Cantrip.Protocol;

namespace Cantrip.Server;

/// <summary>
/// The log the players read: one list of lines for the whole table, every line tagged with the seats that read it.
/// Written from the engine's own log of events (<see cref="Engine.Events.GameLog"/>) as each batch is published.
/// <para>
/// Kept here, by the server, for the whole game, because the log is the record a player reads to find out what
/// happened, and it has to be complete whatever happened to their client. A seat that drops, reloads, or joins late is
/// sent every line it is entitled to, worded as it read at the time.
/// </para>
/// <para>
/// A line is written for each seat from what that seat was sent - its view and its events - at the moment it was sent
/// (<see cref="LogWriter"/>), so no line names what the seat could not see then. An event most seats read the same way,
/// such as "It becomes night.", is one line for all of them; one worded differently for different seats - "You draw
/// [Island]." to one, "Bot 1 draws a card." to the other - is a line per wording, each with its own seats. Seats that
/// share a line share its words, and the objects in it are the same objects, since an event fixes which object each
/// place in its line refers to; each seat is given them by the handles its own view uses.
/// </para>
/// </summary>
public sealed class PlayerLog
{
    private sealed class Line
    {
        public required long Number { get; init; }
        public required long Sequence { get; init; }
        public required int Turn { get; init; }
        public required int ActivePlayer { get; init; }
        public required TurnStepView? Step { get; init; }
        public required string Text { get; init; }
        public required LogEntryKind Kind { get; init; }

        /// <summary>The seats that read this line, each with the handles it knew the named objects by.</summary>
        public Dictionary<int, List<LogReferenceView>> Readers { get; } = [];

        /// <summary>Taken back: what it reported was undone. Kept, so a seat told of the retraction later can be told which.</summary>
        public bool Retracted { get; set; }

        public LogEntryView For(int seat) => new()
        {
            Number = Number,
            Sequence = Sequence,
            Turn = Turn,
            ActivePlayer = ActivePlayer,
            Step = Step,
            Text = Text,
            References = [.. Readers[seat]],
            Kind = Kind,
        };
    }

    private readonly List<Line> _lines = [];

    /// <summary>The lines written for each game event, by its sequence, for sharing a line and for taking it back.</summary>
    private readonly Dictionary<long, List<Line>> _byEvent = [];

    /// <summary>Where each seat's reading has got to: the turn, whose it is, and the step, for the lines' headings.</summary>
    private readonly Dictionary<int, (int Turn, int ActivePlayer, TurnStepView? Step)> _at = [];

    private long _lastNumber;

    /// <summary>What one seat has to be told after a batch: the lines it now reads, and the ones taken back.</summary>
    public sealed record Update(IReadOnlyList<LogEntryView> Added, IReadOnlyList<long> Removed)
    {
        public bool IsEmpty => Added.Count == 0 && Removed.Count == 0;
    }

    /// <summary>
    /// Writes one seat's lines for a batch of events, as projected for that seat with the view it was sent alongside
    /// them, and says what the seat has to be told.
    /// </summary>
    public Update Write(int seat, PlayerView view, IReadOnlyList<(long Sequence, GameEventView View)> events)
    {
        ArgumentNullException.ThrowIfNull(view);
        ArgumentNullException.ThrowIfNull(events);

        (int Turn, int ActivePlayer, TurnStepView? Step) at = _at.GetValueOrDefault(seat, (0, 0, null));
        List<Line> added = [];
        List<long> removed = [];
        (Dictionary<int, GameEventView> totals, Dictionary<int, int> foldedInto) = CombatDamageTotals(events, at.Step);

        for (int index = 0; index < events.Count; index++)
        {
            (long sequence, GameEventView entry) = events[index];

            // Damage dealt at the same moment as damage already written is that line's, and taken back with it.
            if (foldedInto.TryGetValue(index, out int first))
            {
                if (_byEvent.TryGetValue(events[first].Sequence, out List<Line>? into))
                {
                    _byEvent[sequence] = into;
                }

                continue;
            }

            entry = totals.GetValueOrDefault(index, entry);
            switch (entry)
            {
                case StepBeganEvent step:
                    at = (step.Turn, step.ActivePlayer, step.Step);
                    continue;

                case EventsRetractedEvent retracted:
                    foreach (Line line in retracted.Sequences.SelectMany(taken => _byEvent.GetValueOrDefault(taken) ?? []))
                    {
                        line.Retracted = true;
                        if (line.Readers.ContainsKey(seat) && !added.Remove(line))
                        {
                            removed.Add(line.Number);
                        }
                    }

                    continue;

                default:
                    break;
            }

            if (LogWriter.Write(seat, view, entry) is not { } written)
            {
                continue;
            }

            // The same words for the same event are the same line, read by one more seat.
            List<Line> forEvent = _byEvent.TryGetValue(sequence, out List<Line>? existing) ? existing : _byEvent[sequence] = [];
            Line? shared = forEvent.FirstOrDefault(line => line.Text == written.Text && !line.Readers.ContainsKey(seat));

            if (shared is null)
            {
                shared = new Line
                {
                    Number = ++_lastNumber,
                    Sequence = sequence,
                    Turn = at.Turn,
                    ActivePlayer = at.ActivePlayer,
                    Step = at.Step,
                    Text = written.Text,
                    Kind = written.Kind,
                };

                forEvent.Add(shared);
                _lines.Add(shared);
            }

            shared.Readers[seat] = [.. written.References];
            added.Add(shared);
        }

        _at[seat] = at;
        return new Update([.. added.Where(line => !line.Retracted).Select(line => line.For(seat))], removed);
    }

    /// <summary>
    /// Combat damage as one line for each thing dealt it: all combat damage in a step is dealt at once (CR 510.2), so two
    /// hundred attackers are one player losing four hundred life, not two hundred lines of two. Within a batch, in a
    /// combat damage step, each player's life lost - and, apart from it, gained - and each creature's damage marked are
    /// added together into the first such event, and the rest folded into it. First-strike damage is a combat damage step
    /// of its own (CR 510.4), with priority between the two, so it always comes in a batch of its own and keeps a line of
    /// its own.
    /// </summary>
    private static (Dictionary<int, GameEventView> Totals, Dictionary<int, int> FoldedInto) CombatDamageTotals(
        IReadOnlyList<(long Sequence, GameEventView View)> events, TurnStepView? step)
    {
        Dictionary<int, GameEventView> totals = [];
        Dictionary<int, int> foldedInto = [];
        Dictionary<(int Kind, long Who, bool Gained), int> firstOf = [];

        for (int index = 0; index < events.Count; index++)
        {
            GameEventView entry = events[index].View;
            if (entry is StepBeganEvent began)
            {
                step = began.Step;
                firstOf.Clear();
                continue;
            }

            if (step != TurnStepView.CombatDamage)
            {
                continue;
            }

            (int Kind, long Who, bool Gained)? key = entry switch
            {
                LifeChangedEvent { Delta: not 0 } life => (0, life.Player, life.Delta > 0),
                DamageMarkedEvent damage => (1, damage.Object.Value, false),
                _ => null,
            };

            if (key is not { } found)
            {
                continue;
            }

            if (!firstOf.TryGetValue(found, out int first))
            {
                firstOf[found] = index;
                continue;
            }

            foldedInto[index] = first;
            GameEventView total = totals.GetValueOrDefault(first, events[first].View);
            totals[first] = (total, entry) switch
            {
                (LifeChangedEvent sum, LifeChangedEvent life) => new LifeChangedEvent { Player = sum.Player, Delta = sum.Delta + life.Delta, NewTotal = life.NewTotal },
                (DamageMarkedEvent sum, DamageMarkedEvent damage) => new DamageMarkedEvent { Object = sum.Object, Amount = sum.Amount + damage.Amount },
                _ => total,
            };
        }

        return (totals, foldedInto);
    }

    /// <summary>Every line a seat reads, in the order it happened: the whole log, for a seat joining or coming back.</summary>
    public IReadOnlyList<LogEntryView> For(int seat) =>
    [
        .. _lines
            .Where(line => !line.Retracted && line.Readers.ContainsKey(seat))
            .OrderBy(line => line.Sequence)
            .ThenBy(line => line.Number)
            .Select(line => line.For(seat)),
    ];
}
