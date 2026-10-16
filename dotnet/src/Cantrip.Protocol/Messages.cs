using Cantrip.Core.Decisions;
using MessagePack;

namespace Cantrip.Protocol;

/// <summary>
/// Anything the server sends a client.
/// <para>
/// A MessagePack union with explicit keys, so adding a message type is backward compatible and a client
/// built from a different commit fails at the handshake rather than misreading a frame.
/// </para>
/// </summary>
[Union(0, typeof(WelcomeMessage))]
[Union(1, typeof(StateMessage))]
[Union(2, typeof(PromptMessage))]
[Union(3, typeof(GameOverMessage))]
[Union(4, typeof(ErrorMessage))]
[Union(5, typeof(EventsMessage))]
[Union(6, typeof(MembershipMessage))]
[Union(7, typeof(TableMessage))]
[Union(8, typeof(LogMessage))]
[Union(9, typeof(DraftMessage))]
[Union(10, typeof(DeckBuildingMessage))]
[Union(11, typeof(SeatReleasedMessage))]
[Union(12, typeof(TableMoveMessage))]
[Union(13, typeof(PairingsMessage))]
[Union(14, typeof(ClockMessage))]
public abstract class ServerMessage
{
    public abstract string Describe();
}

/// <summary>Sent once when a client is seated.</summary>
[MessagePackObject]
public sealed class WelcomeMessage : ServerMessage
{
    [Key(0)]
    public int ProtocolVersion { get; set; } = PlayerView.ProtocolVersion;

    /// <summary>Which seat this client is playing. Seats are numbered from one.</summary>
    [Key(1)]
    public int Seat { get; set; }

    [Key(2)]
    public List<string> PlayerNames { get; set; } = [];

    /// <summary>
    /// This seat's own deck as the game began: every card it owned, by name, with how many, in name order. Only its own -
    /// what another player brought is theirs to reveal. Tokens and copies are made in the game, so are not in it.
    /// </summary>
    [Key(3)]
    public List<DeckCardView> Decklist { get; set; } = [];

    public override string Describe() => $"welcome, seat {Seat}";
}

/// <summary>A card in a decklist, by name, how many of it, and the card as printed.</summary>
[MessagePackObject]
public sealed class DeckCardView
{
    [Key(0)]
    public string Name { get; set; } = string.Empty;

    [Key(1)]
    public int Count { get; set; }

    /// <summary>
    /// The card as printed, its oracle, as <see cref="StateMessage.Printed"/> sends a card: for showing the deck, and for
    /// drawing its faces before any of it is dealt. It has no handle, so it names no object in the game - which copy is
    /// where in a library stays hidden.
    /// </summary>
    [Key(2)]
    public ObjectView Card { get; set; } = new();
}

/// <summary>
/// The current position as this client is entitled to see it.
/// <para>
/// A full snapshot for now. Deltas and a visibility-filtered semantic event log — which the client needs to
/// know <em>why</em> something changed in order to animate it — are an addition on top, not a replacement:
/// a periodic authoritative snapshot is still what makes reconnection, late joining, and spectating work.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class StateMessage : ServerMessage
{
    [Key(0)]
    public PlayerView View { get; set; } = new();

    /// <summary>
    /// Every card this seat can see in <see cref="View"/> for the first time this game, as printed: its oracle, the card
    /// before anything in the game has touched it, with the other face of a double-faced card. Each is sent once per
    /// game, and all of them again when a seat is brought back up to date. The client draws a card's face from it
    /// before the card is shown, and tells a card the game has changed from it.
    /// </summary>
    [Key(1)]
    public List<ObjectView> Printed { get; set; } = [];

    public override string Describe() => $"state: {View}";
}

/// <summary>
/// What has happened since this client's last update, in terms it can animate.
/// <para>
/// Sent alongside the snapshot rather than instead of it. A structural diff of two views can say a card is no longer
/// on the battlefield and is now in a graveyard, but not whether it was destroyed, sacrificed, or bounced — and those
/// look different on screen. Working the reason out client-side would mean the client inferring rules.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class EventsMessage : ServerMessage
{
    [Key(0)]
    public List<GameEventView> Events { get; set; } = [];

    /// <summary>How far through the log this client has now been brought.</summary>
    [Key(1)]
    public long Sequence { get; set; }

    /// <summary>
    /// The log sequence of each event, in the same order as <see cref="Events"/>. Sent so a later
    /// <see cref="EventsRetractedEvent"/> can name the events it withdraws: events a viewer may not see are left out of
    /// the batch, so position alone could not.
    /// </summary>
    [Key(2)]
    public List<long> Sequences { get; set; } = [];

    public override string Describe() => $"{Events.Count} event(s) to sequence {Sequence}";
}

[Union(0, typeof(ObjectMovedEvent))]
[Union(1, typeof(LifeChangedEvent))]
[Union(2, typeof(DamageMarkedEvent))]
[Union(3, typeof(TapChangedEvent))]
[Union(4, typeof(StepBeganEvent))]
[Union(5, typeof(SpellCastEvent))]
[Union(6, typeof(SpellFizzledEvent))]
[Union(7, typeof(LibraryShuffledEvent))]
[Union(8, typeof(CardsRevealedEvent))]
[Union(9, typeof(EventsRetractedEvent))]
[Union(10, typeof(LibraryArrangedEvent))]
[Union(11, typeof(AbilityActivatedEvent))]
[Union(12, typeof(TokenCreatedEvent))]
[Union(13, typeof(CountersPutEvent))]
[Union(14, typeof(ExtraTurnAddedEvent))]
[Union(15, typeof(PlayerLostEvent))]
[Union(16, typeof(GameEndedEvent))]
[Union(17, typeof(TransformedEvent))]
[Union(18, typeof(DieRolledEvent))]
[Union(19, typeof(ExtraPhaseAddedEvent))]
[Union(20, typeof(DayNightBecameEvent))]
[Union(21, typeof(MonarchBecameEvent))]
[Union(22, typeof(RandomPlayerChosenEvent))]
[Union(23, typeof(StartingPlayerChosenEvent))]
[Union(24, typeof(MulliganTakenEvent))]
[Union(25, typeof(HandKeptEvent))]
[Union(26, typeof(AbilityResolvedEvent))]
[Union(27, typeof(CoinFlippedEvent))]
[Union(28, typeof(TurnEndedEvent))]
public abstract class GameEventView;

/// <summary>The seat chosen at random decided who takes the first turn (CR 103.1).</summary>
[MessagePackObject]
public sealed class StartingPlayerChosenEvent : GameEventView
{
    [Key(0)]
    public int Chooser { get; set; }

    /// <summary>Who plays first: the chooser, unless they chose to draw first.</summary>
    [Key(1)]
    public int Player { get; set; }
}

/// <summary>A seat took a mulligan (CR 103.5). Public, though neither hand is.</summary>
[MessagePackObject]
public sealed class MulliganTakenEvent : GameEventView
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>How many mulligans they have taken, counting this one.</summary>
    [Key(1)]
    public int Count { get; set; }
}

/// <summary>
/// A coin was flipped (CR 705). Public. <see cref="Won"/> is whether <see cref="Player"/> won the flip, the way a card
/// that says "if you lose the flip" reads it.
/// </summary>
[MessagePackObject]
public sealed class CoinFlippedEvent : GameEventView
{
    [Key(0)]
    public int Player { get; set; }

    [Key(1)]
    public bool Won { get; set; }
}

/// <summary>
/// An effect ended <see cref="Player"/>'s turn (CR 724.1): every spell and ability on the stack is exiled and the turn goes
/// straight to its cleanup step. Public.
/// </summary>
[MessagePackObject]
public sealed class TurnEndedEvent : GameEventView
{
    [Key(0)]
    public int Player { get; set; }
}

/// <summary>A seat kept its opening hand (CR 103.5), of this many cards.</summary>
[MessagePackObject]
public sealed class HandKeptEvent : GameEventView
{
    [Key(0)]
    public int Player { get; set; }

    [Key(1)]
    public int Cards { get; set; }
}

/// <summary>A public random player selection, announced as the effect resolves before priority resumes.</summary>
[MessagePackObject]
public sealed class RandomPlayerChosenEvent : GameEventView
{
    [Key(0)]
    public int Controller { get; set; }

    [Key(1)]
    public int SelectedPlayer { get; set; }

    [Key(2)]
    public ViewHandle Source { get; set; }
}

/// <summary>
/// A player became the monarch (CR 725.1). Public: who the monarch is, is a fact about the game that everyone at
/// the table can see, and the view already carries it. Sent whenever it moves, whatever moved it — an effect saying
/// so, a creature getting through to the monarch (CR 725.2), or the monarch leaving the game (CR 725.4).
/// </summary>
[MessagePackObject]
public sealed class MonarchBecameEvent : GameEventView
{
    /// <summary>The player who is now the monarch.</summary>
    [Key(0)]
    public int Player { get; set; }
}

/// <summary>
/// It became day or night (CR 731.1). Public: the designation is a fact about the game, which everyone at the table
/// can see. Sent whenever it changes, whatever changed it — the untap step's own turn-based action (CR 502.2), a
/// permanent with daybound or nightbound arriving in a game that is neither (CR 702.145d, 702.145g), or an effect
/// that says so outright.
/// </summary>
[MessagePackObject]
public sealed class DayNightBecameEvent : GameEventView
{
    /// <summary>What it now is. Never <see cref="DayNightView.Neither"/>: nothing takes a designation away.</summary>
    [Key(0)]
    public DayNightView Designation { get; set; }
}

/// <summary>
/// A double-faced permanent turned over (CR 701.27a). Public, like the permanent. Both names, since the view already
/// shows the new face by the time this is read.
/// </summary>
[MessagePackObject]
public sealed class TransformedEvent : GameEventView
{
    [Key(0)]
    public ViewHandle Object { get; set; }

    [Key(1)]
    public string From { get; set; } = string.Empty;

    [Key(2)]
    public string To { get; set; } = string.Empty;
}

/// <summary>
/// A player lost the game, and what did it (CR 104.3). Public: everyone at the table sees a player leave and knows why.
/// </summary>
[MessagePackObject]
public sealed class PlayerLostEvent : GameEventView
{
    [Key(0)]
    public int Player { get; set; }

    [Key(1)]
    public LossReasonView Reason { get; set; }
}

/// <summary>The game ended (CR 104.2a, 104.4), and how.</summary>
[MessagePackObject]
public sealed class GameEndedEvent : GameEventView
{
    /// <summary>Never <see cref="GameOutcomeView.InProgress"/>.</summary>
    [Key(0)]
    public GameOutcomeView Outcome { get; set; }

    /// <summary>The team left standing, or zero for a draw.</summary>
    [Key(1)]
    public int WinningTeam { get; set; }
}

/// <summary>
/// An activated ability was put on the stack (CR 602.2), and what it was aimed at. The counterpart of
/// <see cref="SpellCastEvent"/>: the ability has no card of its own, so its source is named too.
/// </summary>
[MessagePackObject]
public sealed class AbilityActivatedEvent : GameEventView
{
    /// <summary>The ability on the stack. None once it has already resolved by the time this is read.</summary>
    [Key(0)]
    public ViewHandle Ability { get; set; }

    /// <summary>The permanent whose ability it is.</summary>
    [Key(1)]
    public ViewHandle Source { get; set; }

    [Key(2)]
    public int Controller { get; set; }

    /// <summary>How the ability reads.</summary>
    [Key(3)]
    public string Text { get; set; } = string.Empty;

    [Key(4)]
    public List<TargetMessage> Targets { get; set; } = [];

    /// <summary>A triggered ability put on the stack (CR 603.3) rather than one a player activated.</summary>
    [Key(5)]
    public bool Triggered { get; set; }

    /// <summary>
    /// A modal ability's chosen modes, as their text, in the order they happen (CR 700.2d), as
    /// <see cref="SpellCastEvent.Modes"/> is for a spell; empty for an ability that is not modal.
    /// </summary>
    [Key(6)]
    public List<string> Modes { get; set; } = [];
}

/// <summary>
/// An ability finished resolving (CR 608.2m): the counterpart of <see cref="AbilityActivatedEvent"/>. The events its
/// effects made are the ones from <see cref="First"/> up to this one, so a client can show what it did - the damage, the
/// life, the counters - as coming from its source.
/// </summary>
[MessagePackObject]
public sealed class AbilityResolvedEvent : GameEventView
{
    /// <summary>The ability, which has ceased to exist; none when this viewer was never given a handle for it.</summary>
    [Key(0)]
    public ViewHandle Ability { get; set; }

    /// <summary>The object whose ability it was; none when this viewer cannot see it any more.</summary>
    [Key(1)]
    public ViewHandle Source { get; set; }

    [Key(2)]
    public int Controller { get; set; }

    /// <summary>The log sequence of the first event its effects could have made.</summary>
    [Key(3)]
    public long First { get; set; }

    /// <summary>What it was aimed at, of what this viewer can see.</summary>
    [Key(4)]
    public List<TargetMessage> Targets { get; set; } = [];

    /// <summary>The creatures whose power or toughness it changed, and by how much, of those this viewer can see.</summary>
    [Key(5)]
    public List<PowerToughnessChangeView> PowerToughness { get; set; } = [];
}

/// <summary>How far an effect raised or lowered one creature's power and toughness.</summary>
[MessagePackObject]
public sealed class PowerToughnessChangeView
{
    [Key(0)]
    public ViewHandle Object { get; set; }

    [Key(1)]
    public int Power { get; set; }

    [Key(2)]
    public int Toughness { get; set; }
}

/// <summary>A token was created and put onto the battlefield (CR 111.1).</summary>
[MessagePackObject]
public sealed class TokenCreatedEvent : GameEventView
{
    [Key(0)]
    public ViewHandle Object { get; set; }

    [Key(1)]
    public int Controller { get; set; }

    [Key(2)]
    public string Name { get; set; } = string.Empty;
}

/// <summary>Counters were put on a permanent (CR 122.6).</summary>
[MessagePackObject]
public sealed class CountersPutEvent : GameEventView
{
    [Key(0)]
    public ViewHandle Object { get; set; }

    /// <summary>Which kind, by name (CR 122.1): <c>+1/+1</c>, <c>lore</c>, <c>mining</c>.</summary>
    [Key(1)]
    public string Name { get; set; } = string.Empty;

    /// <summary>How many were put on it.</summary>
    [Key(2)]
    public int Count { get; set; }
}

/// <summary>A player was given extra turns after this one (CR 500.7).</summary>
[MessagePackObject]
public sealed class ExtraTurnAddedEvent : GameEventView
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>How many extra turns.</summary>
    [Key(1)]
    public int Count { get; set; }
}

/// <summary>A phase was added directly after this one, in the turn currently running (CR 500.8), as by an additional combat phase.</summary>
[MessagePackObject]
public sealed class ExtraPhaseAddedEvent : GameEventView
{
    [Key(0)]
    public PhaseView Phase { get; set; }
}

/// <summary>
/// A player looked at the top of their library and put the cards back (CR 701.22), or shuffled instead. Counts only:
/// which cards went where is known to nobody but the player who looked.
/// </summary>
[MessagePackObject]
public sealed class LibraryArrangedEvent : GameEventView
{
    [Key(0)]
    public int Player { get; set; }

    [Key(1)]
    public int OnTop { get; set; }

    [Key(2)]
    public int OnBottom { get; set; }

    [Key(3)]
    public bool Shuffled { get; set; }
}

/// <summary>
/// Earlier events are withdrawn because what they reported was undone, as a mana ability taken back is.
/// <para>
/// A client removes whatever those events produced — a line in the log — rather than showing the undo as something
/// that happened. Sequences this client was never sent are simply not found.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class EventsRetractedEvent : GameEventView
{
    [Key(0)]
    public List<long> Sequences { get; set; } = [];
}

/// <summary>
/// Cards were revealed (CR 701.20), such as a hand shown by Duress.
/// <para>
/// Only cards this viewer can see while the reveal lasts are listed, so it never names anything the viewer was not
/// shown. The handles last as long as the reveal: once the choice it was made for is answered they are dropped.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class CardsRevealedEvent : GameEventView
{
    /// <summary>Whose cards they are.</summary>
    [Key(0)]
    public int Player { get; set; }

    [Key(1)]
    public List<ViewHandle> Cards { get; set; } = [];
}

/// <summary>
/// A spell was cast, and what it was aimed at (CR 601.2c).
/// <para>
/// Sent alongside the move that put it on the stack rather than instead of it: the move is what a renderer
/// animates, and this is what a log reads out. Targets are public from the moment they are announced, so they are
/// carried here — by the time anyone reads the log the spell has resolved and the stack object is gone, so there
/// is nothing left to recover them from.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class SpellCastEvent : GameEventView
{
    [Key(0)]
    public ViewHandle Spell { get; set; }

    [Key(1)]
    public int Controller { get; set; }

    [Key(2)]
    public List<TargetMessage> Targets { get; set; } = [];

    /// <summary>The value chosen for X, for a spell with {X} in its cost; null otherwise.</summary>
    [Key(3)]
    public int? X { get; set; }

    /// <summary>A modal spell's chosen modes, as their text, in the order they happen; empty for any other spell.</summary>
    [Key(4)]
    public List<string> Modes { get; set; } = [];

    /// <summary>How many times the spell was kicked (CR 702.33); zero when it was not.</summary>
    [Key(5)]
    public int Kicked { get; set; }

    /// <summary>Whether that payment was a buyback (CR 702.27) rather than a kicker, which the log words differently.</summary>
    [Key(6)]
    public bool Buyback { get; set; }

    /// <summary>The seat its gift was promised to (CR 702.174k), which is announced with the spell; null when none was.</summary>
    [Key(7)]
    public int? GiftPromisedTo { get; set; }
}

/// <summary>
/// A spell did not resolve because every target it had chosen had become illegal (CR 608.2b).
/// <para>
/// Reported rather than left to be inferred from "it went to a graveyard and nothing happened". A player who has
/// just lost a spell to a removed target wants to be told why, and the client has no way to work it out.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class SpellFizzledEvent : GameEventView
{
    [Key(0)]
    public ViewHandle Spell { get; set; }

    [Key(1)]
    public int Controller { get; set; }
}

/// <summary>A library was shuffled (CR 701.22).</summary>
[MessagePackObject]
public sealed class LibraryShuffledEvent : GameEventView
{
    [Key(0)]
    public int Player { get; set; }
}

/// <summary>A die was rolled (CR 706). Public: everyone at the table sees what a die comes up as.</summary>
[MessagePackObject]
public sealed class DieRolledEvent : GameEventView
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>How many faces the die had (CR 706.1a).</summary>
    [Key(1)]
    public int Sides { get; set; }

    /// <summary>The face that came up, from 1 to <see cref="Sides"/>.</summary>
    [Key(2)]
    public int Result { get; set; }
}

/// <summary>
/// An object changed zones.
/// <para>
/// <see cref="Object"/> is <see cref="ViewHandle.None"/> when the viewer is not entitled to know which card moved —
/// an opponent drawing, for instance. The zones and the reason are still sent, because that a card was drawn is
/// public; which card it was is not.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class ObjectMovedEvent : GameEventView
{
    [Key(0)]
    public ViewHandle Object { get; set; }

    [Key(1)]
    public ZoneKindView From { get; set; }

    [Key(2)]
    public int FromOwner { get; set; }

    [Key(3)]
    public ZoneKindView To { get; set; }

    [Key(4)]
    public int ToOwner { get; set; }

    [Key(5)]
    public ZoneChangeReasonView Reason { get; set; }
}

[MessagePackObject]
public sealed class LifeChangedEvent : GameEventView
{
    [Key(0)]
    public int Player { get; set; }

    [Key(1)]
    public int Delta { get; set; }

    [Key(2)]
    public int NewTotal { get; set; }
}

[MessagePackObject]
public sealed class DamageMarkedEvent : GameEventView
{
    [Key(0)]
    public ViewHandle Object { get; set; }

    [Key(1)]
    public int Amount { get; set; }
}

[MessagePackObject]
public sealed class TapChangedEvent : GameEventView
{
    [Key(0)]
    public ViewHandle Object { get; set; }

    [Key(1)]
    public bool Tapped { get; set; }
}

[MessagePackObject]
public sealed class StepBeganEvent : GameEventView
{
    [Key(0)]
    public int Turn { get; set; }

    [Key(1)]
    public TurnStepView Step { get; set; }

    [Key(2)]
    public int ActivePlayer { get; set; }
}

/// <summary>Why an object moved. Mirrors the engine's reasons; the drift test covers it.</summary>
public enum ZoneChangeReasonView
{
    Draw = 0,
    Play = 1,
    Cast = 2,
    Resolve = 3,
    Discard = 4,
    Destroyed = 5,
    Sacrificed = 6,
    Exiled = 7,
    Returned = 8,
    Milled = 9,
    StateBasedAction = 10,

    /// <summary>Countered (CR 701.5). Not destroyed, and none of its effects happen.</summary>
    Countered = 11,

    /// <summary>Found by a library search (CR 701.23).</summary>
    Searched = 12,

    /// <summary>Put on top of a library from somewhere else, as Brainstorm puts cards back.</summary>
    PutOnLibrary = 13,

    /// <summary>A token put onto the battlefield as it is created (CR 111.1).</summary>
    Created = 14,

    /// <summary>Put into a graveyard from the top of a library by surveil (CR 701.25a).</summary>
    Surveilled = 15,

    /// <summary>Shuffled into a library from a hand or a graveyard, as Timetwister shuffles both in (CR 701.24).</summary>
    ShuffledIn = 16,

    /// <summary>
    /// Manifested (CR 701.40a): onto the battlefield face down, or for manifest dread's other card, into the graveyard.
    /// </summary>
    Manifested = 17,

    /// <summary>Put into a hand from a library without drawing or searching, as Dark Confidant does.</summary>
    PutIntoHand = 18,
}

/// <summary>The server needs this client to decide something.</summary>
[MessagePackObject]
public sealed class PromptMessage : ServerMessage
{
    /// <summary>Echoed back in the answer, so a reply that has been overtaken can be rejected.</summary>
    [Key(0)]
    public int RequestId { get; set; }

    [Key(1)]
    public PromptPayload Prompt { get; set; } = new PriorityPromptMessage();

    /// <summary>
    /// Asked while this client is still announcing a spell or ability, so it may answer with a
    /// <see cref="CancelAnnouncementAnswer"/> instead and give the whole thing up, leaving the game as it was before.
    /// </summary>
    [Key(2)]
    public bool MayCancel { get; set; }

    /// <summary>Structured wording context, scoped to this prompt's authorized recipient; never executable UI.</summary>
    [Key(3)]
    public PromptPresentationContext? Presentation { get; set; }

    public override string Describe() => $"prompt {RequestId}: {Prompt.GetType().Name}";
}

[MessagePackObject]
public sealed class GameOverMessage : ServerMessage
{
    [Key(0)]
    public GameOutcomeView Outcome { get; set; }

    [Key(1)]
    public int WinningTeam { get; set; }

    public override string Describe() => $"game over: {Outcome}";
}

/// <summary>
/// Something the client sent could not be accepted. Never fatal on its own: a rejected stale reply is a
/// normal consequence of reconnecting, not a protocol violation.
/// </summary>
[MessagePackObject]
public sealed class ErrorMessage : ServerMessage
{
    [Key(0)]
    public ErrorCode Code { get; set; }

    [Key(1)]
    public string Detail { get; set; } = string.Empty;

    public override string Describe() => $"error {Code}: {Detail}";
}

public enum ErrorCode
{
    /// <summary>The reply referred to a request that is no longer outstanding. Expected after a reconnect.</summary>
    StaleReply = 0,

    /// <summary>The answer was not something the prompt allows.</summary>
    IllegalAnswer = 1,

    /// <summary>A handle in the answer does not name anything this client can currently see.</summary>
    UnknownHandle = 2,

    ProtocolMismatch = 3,

    NotYourTurnToAnswer = 4,
    MembershipRequired = 5,
    MembershipRefused = 6,
    InvalidSetup = 7,
    TableClosed = 8,
}

/// <summary>Anything a client sends the server.</summary>
[Union(0, typeof(JoinMessage))]
[Union(1, typeof(AnswerMessage))]
[Union(2, typeof(ResumeMessage))]
[Union(3, typeof(ConfigurePlayerMessage))]
[Union(4, typeof(ConfigureTableMessage))]
[Union(5, typeof(ReadyMessage))]
[Union(6, typeof(LeaveMessage))]
[Union(7, typeof(DraftPickMessage))]
[Union(8, typeof(BuildDeckMessage))]
[Union(9, typeof(ConcedeMessage))]
[Union(10, typeof(DropMessage))]
public abstract class ClientMessage
{
    public abstract string Describe();
}

[MessagePackObject]
public sealed class JoinMessage : ClientMessage
{
    [Key(0)]
    public int ProtocolVersion { get; set; } = PlayerView.ProtocolVersion;

    [Key(1)]
    public string PlayerName { get; set; } = string.Empty;

    /// <summary>A preference for an unoccupied lobby seat, never proof of membership.</summary>
    [Key(2)]
    public int PreferredSeat { get; set; }

    /// <summary>
    /// A seat ticket the host issued for this player, or empty. Unlike <see cref="PreferredSeat"/> it is a bearer
    /// credential: it claims the one seat it was issued for, under the name and deck fixed when it was issued. Never
    /// logged or described.
    /// </summary>
    [Key(3)]
    public string Ticket { get; set; } = string.Empty;

    public override string Describe() => $"join as '{PlayerName}'";
}

[MessagePackObject]
public sealed class AnswerMessage : ClientMessage
{
    [Key(0)]
    public int RequestId { get; set; }

    [Key(1)]
    public AnswerPayload Answer { get; set; } = new PassAnswer();

    public override string Describe() => $"answer {RequestId}: {Answer.GetType().Name}";
}

// --- Prompts ---

[Union(0, typeof(PriorityPromptMessage))]
[Union(1, typeof(DiscardPromptMessage))]
[Union(2, typeof(DeclareAttackersPromptMessage))]
[Union(3, typeof(DeclareBlockersPromptMessage))]
[Union(4, typeof(AssignCombatDamagePromptMessage))]
[Union(5, typeof(SearchLibraryPromptMessage))]
[Union(6, typeof(PayCostsPromptMessage))]
[Union(7, typeof(ChooseCardsPromptMessage))]
[Union(8, typeof(UnlessPaysPromptMessage))]
[Union(9, typeof(DistributeCardsPromptMessage))]
[Union(10, typeof(ChooseTargetPromptMessage))]
[Union(11, typeof(MayPromptMessage))]
[Union(12, typeof(ChooseModesPromptMessage))]
[Union(13, typeof(ChooseReplacementPromptMessage))]
[Union(14, typeof(OrderTriggersPromptMessage))]
[Union(15, typeof(NameCardPromptMessage))]
[Union(16, typeof(ChoosePlayerPromptMessage))]
[Union(17, typeof(ChooseColorsPromptMessage))]
[Union(18, typeof(ChooseDividedTargetsPromptMessage))]
[Union(19, typeof(ChooseStartingPlayerPromptMessage))]
[Union(20, typeof(MulliganPromptMessage))]
public abstract class PromptPayload
{
    /// <summary>
    /// The simplest legal answer to this question, composed only out of what the prompt itself carries.
    /// <para>
    /// The mirror of the engine prompt's own default, and abstract for the same reason: a new kind of question cannot
    /// reach the wire without saying how a client that does not understand it should answer. Every automatic
    /// answerer on this side — the client's bot, the transport tests' bots, and the dock the client shows for a prompt
    /// it has no screen for — falls back to this, so a client built before a prompt existed keeps the game moving
    /// instead of sitting on a question it cannot read.
    /// </para>
    /// <para>
    /// Built from the prompt's own lists, so it can never name a card, target, or option that was not offered — the
    /// same guarantee that keeps a bot from killing a game with an answer the server then refuses.
    /// </para>
    /// </summary>
    public abstract AnswerPayload Default();
}

/// <summary>
/// This client chooses the modes of a modal spell it is casting (CR 601.2b, 700.2), while the card is still in its hand.
/// Answered with a <see cref="ModesChosenAnswer"/>. Each chosen mode that targets is asked a
/// <see cref="ChooseTargetPromptMessage"/> next, whose text is the mode's.
/// </summary>
[MessagePackObject]
public sealed class ChooseModesPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The card being cast.</summary>
    [Key(1)]
    public ViewHandle Source { get; set; }

    /// <summary>Every mode's text, in printed order. Answers name modes by index into this list.</summary>
    [Key(2)]
    public List<string> Modes { get; set; } = [];

    /// <summary>The indexes that may be chosen: a mode with no legal target cannot be.</summary>
    [Key(3)]
    public List<int> Eligible { get; set; } = [];

    /// <summary>How many modes to choose, or the fewest of them where <see cref="AtMost"/> opens a range.</summary>
    [Key(4)]
    public int Count { get; set; }

    /// <summary>Whether the same mode may be chosen more than once.</summary>
    [Key(5)]
    public bool MayRepeat { get; set; }

    /// <summary>The choices are additional costs of which one is paid, as "discard a card or pay 3 life", not modes.</summary>
    [Key(6)]
    public bool AdditionalCost { get; set; }

    /// <summary>
    /// What is being chosen, when it is neither modes nor an additional cost and there may be no card to name: "Venture
    /// into Undercity: choose the next room". Empty otherwise.
    /// </summary>
    [Key(7)]
    public string Heading { get; set; } = string.Empty;

    /// <summary>
    /// The most modes that may be chosen, for "choose one or more" (CR 700.2, escalate). Zero where the count is
    /// exact; already narrowed to what the caster can pay the escalate cost for.
    /// </summary>
    [Key(8)]
    public int AtMost { get; set; }

    /// <summary>
    /// The eligible modes whose own cost (spree's) the caster could pay for together with the spell's. Empty where modes
    /// have no cost of their own. A mode outside it may still be chosen, but the cast will be given up while paying.
    /// </summary>
    [Key(9)]
    public List<int> Affordable { get; set; } = [];

    /// <summary>
    /// For an additional cost that is a keyword action, which: <c>Behold</c> or <c>Blight</c>. Empty otherwise.
    /// </summary>
    [Key(10)]
    public string CostKind { get; set; } = string.Empty;

    /// <summary>
    /// What the question adds to the choice, in words: that each mode past the first has a cost (escalate, spree), or that the
    /// modes are the faces a card may be cast as. Empty when there is nothing to add.
    /// </summary>
    [Key(11)]
    public string Detail { get; set; } = string.Empty;

    /// <summary>Whether choosing more modes than the fewest costs something extra (escalate, spree): a range of free modes is "one or both".</summary>
    [Key(12)]
    public bool ExtraModesCost { get; set; }

    /// <summary>
    /// The rules text of each mode, index for index, where a mode is a thing to weigh and not only to read: the faces a card may be
    /// cast as (CR 709.3) are each a card's printed text. Empty for modes the question's own text says it all of.
    /// </summary>
    [Key(13)]
    public List<string> Texts { get; set; } = [];

    /// <summary>
    /// The eligible modes in the order worth trying them: those <see cref="Affordable"/> names first, then the rest, each
    /// group in printed order.
    /// </summary>
    [IgnoreMember]
    public IEnumerable<int> Preferred => Eligible.OrderBy(mode => Affordable.Count > 0 && !Affordable.Contains(mode));

    /// <summary>The most modes an answer may name: the range's top where there is one, <see cref="Count"/> otherwise.</summary>
    [IgnoreMember]
    public int Most => AtMost > Count ? AtMost : Count;

    /// <summary>
    /// The first eligible modes, affordable ones first, repeating the last one it may when repeats are allowed and too
    /// few are eligible. The fewest a range will take, since the modes past the first are what escalate and spree charge
    /// for.
    /// </summary>
    public override AnswerPayload Default() => new ModesChosenAnswer
    {
        Modes = MayRepeat
            ? [.. Enumerable.Range(0, Count).Select(i => Eligible[Math.Min(i, Eligible.Count - 1)])]
            : [.. Preferred.Take(Count)],
    };
}

/// <summary>
/// Somewhere the cards of a <see cref="DistributeCardsPromptMessage"/> may go, and how many of them it takes.
/// <para>
/// A destination says which cards it will accept rather than why, so a client can tell whether a placement is legal
/// without knowing any rules: see <c>CardDistribution</c> in <c>Cantrip.Core</c>, which is the same check the engine
/// makes when the answer arrives.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class CardDestinationMessage
{
    /// <summary>Where cards put here go, which is what a client labels the pile with.</summary>
    [Key(0)]
    public CardDestinationKindView Kind { get; set; }

    /// <summary>Whose zone it is. Usually the player being asked.</summary>
    [Key(1)]
    public int Owner { get; set; }

    /// <summary>The fewest cards that may be put here, already reduced to what the cards there are can meet.</summary>
    [Key(2)]
    public int Min { get; set; }

    /// <summary>The most that may be, or <see cref="int.MaxValue"/> for "and the rest".</summary>
    [Key(3)]
    public int Max { get; set; }

    /// <summary>Which of the cards looked at may be put here. All of them, unless the effect narrowed it.</summary>
    [Key(4)]
    public List<ViewHandle> Eligible { get; set; } = [];

    /// <summary>Whether the order cards are put here in matters, as it does for either end of a library.</summary>
    [Key(5)]
    public bool Ordered { get; set; }

    /// <summary>Whether they are placed in a random order instead, so there is no order for anyone to choose.</summary>
    [Key(6)]
    public bool Randomized { get; set; }

    /// <summary>
    /// The kinds this takes one card of each, as Atraxa, Grand Unifier's "a card of each card type": every card put
    /// here has to count as a different one of these lists. Empty when it does not care.
    /// </summary>
    [Key(7)]
    public List<List<ViewHandle>> Groups { get; set; } = [];

    /// <summary>What each group is called, index for index: "Creature".</summary>
    [Key(8)]
    public List<string> GroupNames { get; set; } = [];
}

/// <summary>
/// This client looks at some cards and says where each one goes: scrying, which may send some to the bottom;
/// surveilling, which may put some into the graveyard; Ponder's "in any order", which may shuffle instead; Stock Up's
/// two into the hand and the rest on the bottom.
/// <para>
/// All of them are one message because they are one question, differing only in <see cref="Destinations"/>. Answered
/// with a <see cref="CardsDistributedAnswer"/> naming every card exactly once across lists that run index for index
/// with the destinations. The handles last as long as the prompt: once it is answered they are dropped, so nothing can
/// be tracked afterwards.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class DistributeCardsPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The spell or ability doing the looking.</summary>
    [Key(1)]
    public ViewHandle Source { get; set; }

    /// <summary>The cards looked at, top card of the library first.</summary>
    [Key(2)]
    public List<ViewHandle> Cards { get; set; } = [];

    /// <summary>Where they may go, in the order the card's rules text names them.</summary>
    [Key(3)]
    public List<CardDestinationMessage> Destinations { get; set; } = [];

    /// <summary>Whether shuffling instead is allowed.</summary>
    [Key(4)]
    public bool MayShuffle { get; set; }

    /// <summary>
    /// Requirements met in order, then everything left over into the first destination that will take it: the same
    /// placement the engine would fall back to, so a client that draws no dock for this still answers legally.
    /// </summary>
    public override AnswerPayload Default() => new CardsDistributedAnswer
    {
        Placements = CardDistribution.Greedy(Cards, Limits(), Accepted(), Groups()),
    };

    /// <summary>
    /// Why this placement is not one the server will take, or null when it is — the same question
    /// <c>DistributeCardsPrompt.Accepts</c> asks, from the same code, so a client can grey its Done button on exactly
    /// what would be refused rather than on a guess at the rule.
    /// </summary>
    public string? Problem(CardsDistributedAnswer answer)
    {
        ArgumentNullException.ThrowIfNull(answer);

        return CardDistribution.Problem(
            Cards,
            Limits(),
            Accepted(),
            [.. answer.Placements.Select(placement => (IReadOnlyList<ViewHandle>)placement)],
            Groups());
    }

    private IReadOnlyList<IReadOnlyList<ViewHandle>>[] Groups() =>
        [.. Destinations.Select(destination =>
            (IReadOnlyList<IReadOnlyList<ViewHandle>>)[.. destination.Groups.Select(group => (IReadOnlyList<ViewHandle>)group)])];

    private DestinationLimit[] Limits() =>
        [.. Destinations.Select(destination => new DestinationLimit(destination.Min, destination.Max))];

    private IReadOnlyList<ViewHandle>[] Accepted() =>
        [.. Destinations.Select(destination => (IReadOnlyList<ViewHandle>)destination.Eligible)];
}

/// <summary>
/// This client chooses cards from a hand as a spell resolves: its own discards, or a card from an opponent's hand
/// that has been revealed to it, as Duress does.
/// <para>
/// Every card in the hand is listed, and <see cref="Eligible"/> says which may be picked. Exactly
/// <see cref="Count"/> have to be picked, which is already reduced to what is possible — zero when nothing is eligible,
/// in which case the answer is an empty choice once the hand has been looked at.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class ChooseCardsPromptMessage : PromptPayload
{
    /// <summary>Who is choosing.</summary>
    [Key(0)]
    public int Player { get; set; }

    /// <summary>Whose hand the cards are in.</summary>
    [Key(1)]
    public int Owner { get; set; }

    /// <summary>The spell asking.</summary>
    [Key(2)]
    public ViewHandle Source { get; set; }

    [Key(3)]
    public CardChoicePurposeView Purpose { get; set; }

    /// <summary>The whole hand, in hand order.</summary>
    [Key(4)]
    public List<ViewHandle> Cards { get; set; } = [];

    [Key(5)]
    public List<ViewHandle> Eligible { get; set; } = [];

    /// <summary>How many to choose, or with <see cref="UpTo"/> the most that may be chosen.</summary>
    [Key(6)]
    public int Count { get; set; }

    /// <summary>Whether <see cref="Count"/> is a maximum rather than an exact number ("up to three lands").</summary>
    [Key(7)]
    public bool UpTo { get; set; }

    /// <summary>What kind of card is being chosen ("artifact"), where the purpose alone does not say. Empty otherwise.</summary>
    [Key(8)]
    public string Kind { get; set; } = string.Empty;

    /// <summary>
    /// The total power the chosen cards must add up to, as saddle's "total power N or greater" (CR 702.171a); zero
    /// for a choice that asks nothing of it. <c>Cantrip.Core.Decisions.TotalPower</c> is the rule.
    /// </summary>
    [Key(9)]
    public int MinimumPower { get; set; }

    /// <summary>The power of each card in <see cref="Eligible"/>, index for index, when <see cref="MinimumPower"/> is set.</summary>
    [Key(10)]
    public List<int> Powers { get; set; } = [];

    /// <summary>
    /// The total mana value the chosen cards must add up to, as collect evidence's "total mana value N or greater"
    /// (CR 701.59a); zero for a choice that asks nothing of it. <c>Cantrip.Core.Decisions.TotalManaValue</c> is the rule.
    /// </summary>
    [Key(11)]
    public int MinimumManaValue { get; set; }

    /// <summary>The mana value of each card in <see cref="Eligible"/>, index for index, when <see cref="MinimumManaValue"/> is set.</summary>
    [Key(12)]
    public List<int> ManaValues { get; set; } = [];

    /// <summary>
    /// The most a cast chosen here may be worth: the mana value of the spell cast must be at most this, as what is left of Uldaros
    /// Theorix's "total mana value 6 or less". Zero when there is no limit.
    /// </summary>
    [Key(13)]
    public int MaximumManaValue { get; set; }

    /// <summary>For <see cref="CardChoicePurposeView.PutCounters"/>, the kind of counter the chosen permanent gets, as Oracle text names it ("loyalty", "+1/+1"). Empty otherwise.</summary>
    [Key(14)]
    public string Counter { get; set; } = string.Empty;

    /// <summary>For <see cref="CardChoicePurposeView.PutCounters"/>, how many counters the chosen permanent gets.</summary>
    [Key(15)]
    public int CounterCount { get; set; }

    /// <summary>
    /// The first eligible cards, never anything outside <see cref="Eligible"/>. A choice that must add up to a power is
    /// offered "up to" every candidate, so taking them all meets it whenever anything can.
    /// </summary>
    public override AnswerPayload Default() => new CardsChosenAnswer { Cards = [.. Eligible.Take(Count)] };
}

/// <summary>
/// This client chooses the target of a triggered ability it controls, as the ability goes on the stack (CR 603.3d), or of
/// one chosen mode of a modal spell it is casting, when <see cref="Ability"/> is the card in its hand.
/// </summary>
[MessagePackObject]
public sealed class ChooseTargetPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The ability, already on the stack; or the modal spell being cast, still in hand.</summary>
    [Key(1)]
    public ViewHandle Ability { get; set; }

    /// <summary>The permanent whose ability it is. None if it has already left the battlefield.</summary>
    [Key(2)]
    public ViewHandle Source { get; set; }

    [Key(3)]
    public string Text { get; set; } = string.Empty;

    /// <summary>Every legal target. Answer with one of these, or none when <see cref="Optional"/>.</summary>
    [Key(4)]
    public List<TargetMessage> Candidates { get; set; } = [];

    /// <summary>"Up to one target": answering with no target is allowed.</summary>
    [Key(5)]
    public bool Optional { get; set; }

    /// <summary>
    /// How many targets to choose: one for almost everything, and more for a slot counted by X, as Candelabra of
    /// Tawnos' "Untap X target lands". Never more than <see cref="Candidates"/> offers.
    /// </summary>
    [Key(6)]
    public int Count { get; set; } = 1;

    /// <summary>
    /// Whether these are targets (CR 115.1) or things merely chosen as an effect resolves (CR 608.2d), as Comet,
    /// Stellar Pup's "to a creature or player" chooses. The answer is the same shape either way; only what the dock
    /// calls it differs, because a player told "target" would read the wrong rule off the screen.
    /// </summary>
    [Key(7)]
    public bool Targeted { get; set; } = true;

    /// <summary>
    /// The fewest targets a printed range takes — "one or two target creatures" is a <see cref="Count"/> of two and a
    /// minimum of one (CR 601.2c). Zero when there is no range: exactly <see cref="Count"/>, or none where <see cref="Optional"/>.
    /// </summary>
    [Key(8)]
    public int Minimum { get; set; }

    /// <summary>
    /// "Any number of" rather than "up to <see cref="Count"/>": every candidate or none may be chosen, as proliferate's
    /// "choose any number of permanents and/or players" (CR 701.34a). Comes with <see cref="Optional"/> and a
    /// <see cref="Count"/> of every candidate; only the dock's wording differs.
    /// </summary>
    [Key(9)]
    public bool AnyNumber { get; set; }

    /// <summary>
    /// The opponent whose slot this is, when the ability has one slot for each opponent (The Theorist's "for each opponent, return
    /// up to one target ..."): a seat, or zero. <see cref="Text"/> then names where it goes with "{player}".
    /// </summary>
    [Key(10)]
    public int ForPlayer { get; set; }

    /// <summary>No target where that is allowed, and otherwise the first candidates the server offered.</summary>
    public override AnswerPayload Default() =>
        new TargetsChosenAnswer { Targets = Optional ? [] : [.. Candidates.Take(Count)] };
}

/// <summary>
/// This client chooses the targets of one slot marked "divided" and, together with them, how much of a total each
/// gets (CR 601.2d): "damage divided as you choose among any number of target creatures". Answered with a
/// <see cref="TargetsDividedAnswer"/>.
/// <para>
/// Not the same question as <see cref="AssignCombatDamagePromptMessage"/>, whose recipients are already fixed
/// blockers or attackers and only their shares are being asked for. Here the recipients themselves are the choice.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class ChooseDividedTargetsPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The spell being cast, or the ability being activated or put on the stack.</summary>
    [Key(1)]
    public ViewHandle Ability { get; set; }

    /// <summary>The permanent whose ability it is, or the card being cast.</summary>
    [Key(2)]
    public ViewHandle Source { get; set; }

    [Key(3)]
    public string Text { get; set; } = string.Empty;

    /// <summary>Every legal target this slot could take. At most this many may be chosen.</summary>
    [Key(4)]
    public List<TargetMessage> Candidates { get; set; } = [];

    /// <summary>The total to divide among however many of <see cref="Candidates"/> are chosen.</summary>
    [Key(5)]
    public int Total { get; set; }

    /// <summary>Maximum chosen targets, or zero when only the total limits them.</summary>
    [Key(6)]
    public int MaxTargets { get; set; }

    /// <summary>
    /// Whether the slot reads "up to" (CR 601.2c): then choosing no target at all, and dividing nothing, is legal.
    /// </summary>
    [Key(7)]
    public bool Optional { get; set; }

    /// <summary>The whole total on the first candidate, or nothing when there is no total to divide.</summary>
    public override AnswerPayload Default() => new TargetsDividedAnswer
    {
        Shares = Total > 0 && Candidates.Count > 0
            ? [new TargetShareMessage { Target = Candidates[0], Amount = Total }]
            : [],
    };
}

/// <summary>One chosen target's own share of a divided total (CR 601.2d).</summary>
[MessagePackObject]
public sealed class TargetShareMessage
{
    [Key(0)]
    public TargetMessage Target { get; set; } = new();

    [Key(1)]
    public int Amount { get; set; }
}

/// <summary>
/// This client decides whether to do something a spell or ability says it "may" do (CR 608.2d), such as Path to Exile's
/// "its controller may search their library". Answered with a <see cref="MayChosenAnswer"/>.
/// </summary>
[MessagePackObject]
public sealed class MayPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The spell or ability on the stack that asks, or for <c>PayLife</c> the land about to enter.</summary>
    [Key(1)]
    public ViewHandle Source { get; set; }

    /// <summary>
    /// What would be done, as the effect's verb: <c>SearchLibrary</c>; or <c>PayLife</c>, a shockland's "As this land
    /// enters, you may pay 2 life. If you don't, it enters tapped."
    /// </summary>
    [Key(2)]
    public string Verb { get; set; } = string.Empty;

    /// <summary>How much, for a verb with an amount: the life <c>PayLife</c> pays. Otherwise 0.</summary>
    [Key(3)]
    public int Amount { get; set; }

    /// <summary>Not doing it: "you may" never has to be taken up.</summary>
    public override AnswerPayload Default() => new MayChosenAnswer { Accept = false };
}

/// <summary>
/// This client chooses a player, without targeting them, as a spell or ability resolves (CR 608.2d): Baleful Mastery's
/// "an opponent draws a card". Answered with a <see cref="PlayerChosenAnswer"/> naming one of <see cref="Candidates"/>.
/// </summary>
[MessagePackObject]
public sealed class ChoosePlayerPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The spell or ability on the stack that asks.</summary>
    [Key(1)]
    public ViewHandle Source { get; set; }

    /// <summary>The seats that may be chosen, in turn order. Always two or more.</summary>
    [Key(2)]
    public List<int> Candidates { get; set; } = [];

    /// <summary>What the chosen player will do, as the effect's verb: <c>Draw</c>.</summary>
    [Key(3)]
    public string Verb { get; set; } = string.Empty;

    /// <summary>How much, for a verb with an amount: the cards <c>Draw</c> draws. Otherwise 0.</summary>
    [Key(4)]
    public int Amount { get; set; }

    /// <summary>The first candidate in turn order.</summary>
    public override AnswerPayload Default() => new PlayerChosenAnswer { Player = Candidates.Count > 0 ? Candidates[0] : 0 };
}

/// <summary>
/// This client, chosen at random, decides who takes the first turn (CR 103.1), before any hand is dealt: itself to play
/// first, or in a two-player game the other seat to draw first. Answered with a <see cref="PlayerChosenAnswer"/> naming
/// one of <see cref="Candidates"/>.
/// </summary>
[MessagePackObject]
public sealed class ChooseStartingPlayerPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>Every seat, in turn order.</summary>
    [Key(1)]
    public List<int> Candidates { get; set; } = [];

    /// <summary>Playing first.</summary>
    public override AnswerPayload Default() => new PlayerChosenAnswer { Player = Player };
}

/// <summary>
/// This client keeps its hand or takes a mulligan (CR 103.5). Answered with a <see cref="MayChosenAnswer"/>: accepting
/// takes the mulligan. Keeping with mulligans taken is followed by a <see cref="ChooseCardsPromptMessage"/> for
/// <see cref="CardChoicePurposeView.PutOnBottom"/>.
/// </summary>
[MessagePackObject]
public sealed class MulliganPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>Mulligans taken so far: zero for the hand first dealt.</summary>
    [Key(1)]
    public int Mulligans { get; set; }

    /// <summary>How many cards keeping this hand keeps, once the mulligans' cards are put on the bottom.</summary>
    [Key(2)]
    public int Keeps { get; set; }

    /// <summary>Keeping.</summary>
    public override AnswerPayload Default() => new MayChosenAnswer { Accept = false };
}

/// <summary>
/// This client chooses one or more colors as a spell or ability resolves, or as a permanent enters (CR 202.2b):
/// Coalition Relic's "add one mana of any color for each charge counter removed this way". Answered with a
/// <see cref="ColorsChosenAnswer"/> naming <see cref="Count"/> of <see cref="Choices"/>.
/// </summary>
[MessagePackObject]
public sealed class ChooseColorsPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The spell, ability or permanent that asks.</summary>
    [Key(1)]
    public ViewHandle Source { get; set; }

    /// <summary>The colors that may be chosen, in WUBRG order. Never empty.</summary>
    [Key(2)]
    public List<ColorView> Choices { get; set; } = [];

    /// <summary>How many colors to choose. One for "choose a color".</summary>
    [Key(3)]
    public int Count { get; set; } = 1;

    /// <summary>Whether the same color may be chosen more than once.</summary>
    [Key(4)]
    public bool MayRepeat { get; set; }

    /// <summary>What the colors are for, as the effect's verb: <c>AddMana</c>.</summary>
    [Key(5)]
    public string Verb { get; set; } = string.Empty;

    /// <summary>The first color on offer, as many times as are wanted.</summary>
    public override AnswerPayload Default() => new ColorsChosenAnswer
    {
        Colors = MayRepeat
            ? [.. Enumerable.Repeat(Choices.Count > 0 ? Choices[0] : ColorView.White, Count)]
            : [.. Choices.Take(Count)],
    };
}

/// <summary>
/// This client chooses a card name (CR 201.4) as a permanent enters (CR 614.12a), as Anointed Peacekeeper's "As this
/// creature enters, look at an opponent's hand, then choose any card name". Answered with a
/// <see cref="CardNamedAnswer"/> naming one of <see cref="Names"/>.
/// </summary>
[MessagePackObject]
public sealed class NameCardPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The card about to enter.</summary>
    [Key(1)]
    public ViewHandle Source { get; set; }

    /// <summary>Whose hand is being looked at, or zero for a name chosen without looking.</summary>
    [Key(2)]
    public int Owner { get; set; }

    /// <summary>That hand, in hand order, shown to this client alone.</summary>
    [Key(3)]
    public List<ViewHandle> Hand { get; set; } = [];

    /// <summary>Every name that may be chosen, sorted: each card the game knows.</summary>
    [Key(4)]
    public List<string> Names { get; set; } = [];

    /// <summary>A name worth choosing: the costliest nonland card in the hand looked at, or the first name there is.</summary>
    [Key(5)]
    public string Suggestion { get; set; } = string.Empty;

    /// <summary>The suggestion, which is always one of <see cref="Names"/>.</summary>
    public override AnswerPayload Default() => new CardNamedAnswer { Name = Suggestion };
}

/// <summary>
/// Two or more replacement or prevention effects are trying to modify the same event, and this client chooses which
/// applies first (CR 616.1).
/// <para>
/// Sources and hints run in step: the client shows each source card with what it would do, and answers with the index
/// of the one to apply first. Every one of them is a legal answer.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class ChooseReplacementPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The object the event is about, or no handle for an event about no one object.</summary>
    [Key(1)]
    public ViewHandle Subject { get; set; }

    /// <summary>What the event is, in words.</summary>
    [Key(2)]
    public string Event { get; set; } = string.Empty;

    /// <summary>The source of each applicable effect, in the order they may be chosen.</summary>
    [Key(3)]
    public List<ViewHandle> Sources { get; set; } = [];

    /// <summary>What each of those effects would do, in words. Runs in step with <see cref="Sources"/>.</summary>
    [Key(4)]
    public List<string> Hints { get; set; } = [];

    /// <summary>The first offered effect. Any choice is legal, so the first is as good an automatic answer as any.</summary>
    public override AnswerPayload Default() => new ReplacementChosenAnswer { Index = 0 };
}

/// <summary>
/// Several of this client's abilities triggered at once, and they choose the order the abilities go on the stack
/// (CR 603.3b). The last one named goes on last and therefore resolves first. Answered with a
/// <see cref="TriggersOrderedAnswer"/>.
/// </summary>
[MessagePackObject]
public sealed class OrderTriggersPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The source of each ability, in the order they triggered.</summary>
    [Key(1)]
    public List<ViewHandle> Sources { get; set; } = [];

    /// <summary>Each ability as it reads. Runs in step with <see cref="Sources"/>.</summary>
    [Key(2)]
    public List<string> Texts { get; set; } = [];

    /// <summary>The order they triggered in. Any order is legal, so this needs no thought to be correct.</summary>
    public override AnswerPayload Default()
    {
        var order = new TriggersOrderedAnswer();

        for (int i = 0; i < Sources.Count; i++)
        {
            order.Order.Add(i);
        }

        return order;
    }
}

/// <summary>
/// This client may pay a cost to stop something happening (CR 118.12), such as Mana Leak's {3}.
/// <para>
/// Mana abilities are activated from here exactly as they are while paying for a spell. Covering the cost does not pay
/// it: paying is its own option, offered once the pool covers the cost, and declining is always offered.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class UnlessPaysPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The spell asking, such as Mana Leak.</summary>
    [Key(1)]
    public ViewHandle Source { get; set; }

    /// <summary>What paying protects, such as the spell that would be countered.</summary>
    [Key(2)]
    public ViewHandle Subject { get; set; }

    /// <summary>The cost, in oracle notation.</summary>
    [Key(3)]
    public string Total { get; set; } = string.Empty;

    /// <summary>What the pool does not cover, in oracle notation. <c>{0}</c> when paying is possible now.</summary>
    [Key(4)]
    public string Remaining { get; set; } = string.Empty;

    [Key(5)]
    public int RemainingGeneric { get; set; }

    [Key(6)]
    public List<ColorView> RemainingColored { get; set; } = [];

    /// <summary>Mana abilities, undo, pay (when covered), and decline. Never empty.</summary>
    [Key(7)]
    public List<ActionOptionMessage> Options { get; set; } = [];

    /// <summary>Hybrid and Phyrexian symbols still owed (CR 107.4e-f), each payable another way besides its colors.</summary>
    [Key(8)]
    public List<HybridSymbolView> RemainingHybrid { get; set; } = [];

    /// <summary>
    /// What paying buys, in card vocabulary, when it buys something rather than stopping it: "CopySpell" for Chain
    /// Lightning's copy of <see cref="Subject"/>. Empty for "unless pays", where paying keeps the subject.
    /// </summary>
    [Key(9)]
    public string Verb { get; set; } = string.Empty;

    /// <summary>Declining, which is always among the options.</summary>
    public override AnswerPayload Default() => new TakeActionAnswer { Kind = ActionKindView.DeclineCost };
}

/// <summary>
/// This client has priority and may act or pass.
/// <para>
/// Options are prototypes the client fills in, never a flat enumeration of concrete actions: an X spell with
/// modes and divided damage would be millions of tuples. The client composes a candidate and the server
/// validates it, which is also why announcing a spell never makes the engine suspend mid-cast.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class PriorityPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    [Key(1)]
    public List<ActionOptionMessage> Options { get; set; } = [];

    /// <summary>Passing, which is always legal (CR 117.3d).</summary>
    public override AnswerPayload Default() => new PassAnswer();
}

[MessagePackObject]
public sealed class ActionOptionMessage
{
    [Key(0)]
    public ActionKindView Kind { get; set; }

    [Key(1)]
    public ViewHandle Source { get; set; }

    /// <summary>
    /// Which of the source's abilities, for an activation. Indexes the computed ability list. For a cast, which way to
    /// pay: 0 for the mana cost, n for the card's nth alternative cost (CR 118.9), described in <see cref="Description"/>.
    /// </summary>
    [Key(2)]
    public int AbilityIndex { get; set; }

    /// <summary>How many targets this action needs, zero for none.</summary>
    [Key(3)]
    public int TargetsRequired { get; set; }

    /// <summary>What the target could legally be. Advice the client picks from; the server re-checks the choice.</summary>
    [Key(4)]
    public List<TargetMessage> TargetCandidates { get; set; } = [];

    /// <summary>
    /// The mana an activation would add, for a mana ability. Advice: it lets a client paying a cost tell which land
    /// helps without knowing what a Forest is.
    /// </summary>
    [Key(5)]
    public List<ManaTypeView> Produces { get; set; } = [];

    /// <summary>
    /// For a cast: whether the pool plus the player's untapped mana sources could pay for it. Advice only — the spell
    /// may be announced regardless and taken back if it cannot be paid for — so a client that would rather not try,
    /// such as a bot, knows not to.
    /// </summary>
    [Key(6)]
    public bool Affordable { get; set; }

    /// <summary>
    /// For an activation: mana of a single color the ability adds on top of <see cref="Produces"/>. When more than zero,
    /// the answer has to name a color, as activating Black Lotus does.
    /// </summary>
    [Key(7)]
    public int ProducesOfAnyOneColor { get; set; }

    /// <summary>For a cast: whether the cost has {X}, so the answer carries a chosen X.</summary>
    [Key(8)]
    public bool HasX { get; set; }

    /// <summary>For a cast with {X}: the largest X that could be paid for, or -1 if none could. Advice only.</summary>
    [Key(9)]
    public int MaxX { get; set; }

    /// <summary>
    /// For an activation: how the ability reads, cost and all, such as "{T}, Sacrifice Wasteland: Destroy target nonbasic
    /// land." For showing on a menu, and no client reads a rule out of it. The tactical seat reads it as a player reads
    /// the menu, since it is all that says which ability this is; what it makes of it only chooses among what is offered.
    /// </summary>
    [Key(11)]
    public string Description { get; set; } = string.Empty;

    /// <summary>For an activation: life it costs.</summary>
    [Key(12)]
    public int LifeCost { get; set; }

    /// <summary>For a mana ability: damage it deals to its controller as it resolves.</summary>
    [Key(13)]
    public int DamageToController { get; set; }

    /// <summary>
    /// For an activation with <see cref="ProducesOfAnyOneColor"/>: the colors the answer may name, all five for Black
    /// Lotus and two for a Talisman. Naming any other is refused.
    /// </summary>
    [Key(14)]
    public List<ColorView> ColorChoices { get; set; } = [];

    /// <summary>
    /// For a mana ability: its cost as rules text, such as "{T}, Pay 1 life". Lets a client write one menu row per color
    /// the ability could add. For showing; never parsed.
    /// </summary>
    [Key(15)]
    public string CostText { get; set; } = string.Empty;

    /// <summary>
    /// "Up to one target" (CR 601.2c): the action may be taken aiming at nothing even while
    /// <see cref="TargetCandidates"/> offers something, so a client aiming it offers declining as well.
    /// </summary>
    [Key(16)]
    public bool TargetsOptional { get; set; }

    /// <summary>
    /// For an offer to put pool mana towards a spell: what that mana may be spent on. Its type is in <see cref="Produces"/>.
    /// For a mana ability: what the mana it adds may be spent on.
    /// </summary>
    [Key(17)]
    public ManaSpendFilterView Filter { get; set; }

    /// <summary>What spending that mana does to whatever it pays for, alongside <see cref="Filter"/>.</summary>
    [Key(18)]
    public ManaSpendEffectView Effect { get; set; }

    /// <summary>
    /// For a cast: the name of the spell this would cast, when that is not the source card's own name — an adventurer
    /// card cast as its Adventure (CR 715.3) casts "Battle Display", not Embereth Shieldbreaker. Empty when the spell
    /// is the card, which is the usual case. For showing on a menu; never parsed.
    /// </summary>
    [Key(19)]
    public string SpellName { get; set; } = string.Empty;

    /// <summary>
    /// For an activation: mana the ability adds whose colors are each chosen on their own (CR 106.1b), as Orcish
    /// Lumberjack's "three mana in any combination of {R} and/or {G}". When more than zero, the answer names that many
    /// colors out of <see cref="ColorChoices"/>, repeats and all — which is what one color for all of them cannot say.
    /// </summary>
    [Key(20)]
    public int ProducesOfAnyColorEach { get; set; }

    /// <summary>
    /// For an activation whose cost sacrifices a permanent its controller chooses (CR 118.3, 701.21a), as Orcish
    /// Lumberjack's "Sacrifice a Forest": the permanents that could pay it. When non-empty, the answer names one.
    /// </summary>
    [Key(21)]
    public List<ViewHandle> SacrificeCandidates { get; set; } = [];

    /// <summary>The controlled permanent this option taps to pay its mana ability cost.</summary>
    [Key(22)]
    public ViewHandle? Tap { get; set; }

    /// <summary>
    /// For a tap-to-pay or exile-to-pay option paying {1} of a cost's generic mana (waterbend, improvise, delve): how
    /// many more {1} its way of paying can still pay, however many candidates are offered for it. Zero for convoke.
    /// No mana ability is offered once one of these has paid (CR 601.2g-h), so a seat makes the rest in mana first.
    /// </summary>
    [Key(23)]
    public int PaysUpTo { get; set; }

    /// <summary>With <see cref="PaysUpTo"/>: which of the cost's ways of paying generic mana this candidate pays.</summary>
    [Key(24)]
    public int Alternative { get; set; }

    /// <summary>For a mana ability: life its controller gains as it resolves, as The Great Henge's.</summary>
    [Key(25)]
    public int LifeGained { get; set; }

    /// <summary>
    /// For a mana ability the player has of their own (CR 611.2a, 605.1a), which comes with no <see cref="Source"/>:
    /// the name of what gave it to them, "Channel", whatever has become of that since. Empty for anything else. For
    /// showing; never parsed.
    /// </summary>
    [Key(26)]
    public string GrantedBy { get; set; } = string.Empty;
}

/// <summary>
/// This client has announced a spell and still owes part of its cost (CR 601.2g-h).
/// <para>
/// The client tapping a land here sends an activation exactly as it would from a priority prompt; the server applies
/// the pool the moment it covers the cost in one way only, the spell goes onto the stack, and the prompt goes away.
/// When the pool could cover it in more than one way — a different mana would be left behind, counting a restriction
/// as a difference — the options offer pool mana to put towards the cost instead, one per kind that would narrow the
/// ways down, and the spell is cast once one way is left. Stopping is always offered, and returns the game to the
/// moment before the spell was announced (CR 733.1). Nobody else is told about any of this until the spell is paid
/// for.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class PayCostsPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>
    /// The spell being paid for, on the stack already (CR 601.2a); or, for an ability, its source, wherever that is - the
    /// ability itself is on the stack (CR 602.2a).
    /// </summary>
    [Key(1)]
    public ViewHandle Spell { get; set; }

    /// <summary>The whole cost, in oracle notation.</summary>
    [Key(2)]
    public string Total { get; set; } = string.Empty;

    /// <summary>What the pool does not cover, in oracle notation. For display.</summary>
    [Key(3)]
    public string Remaining { get; set; } = string.Empty;

    /// <summary>Generic mana still owed, so a client can tell whether any mana at all would help.</summary>
    [Key(4)]
    public int RemainingGeneric { get; set; }

    /// <summary>Coloured symbols still owed, one entry per symbol, so a client can tell which land would help.</summary>
    [Key(5)]
    public List<ColorView> RemainingColored { get; set; } = [];

    /// <summary>
    /// What the player may do: mana abilities to activate, the last one to take back, and stopping; pool mana to put
    /// towards the cost, when the pool could pay it in more than one way and which is the player's to choose; and for
    /// a spell with {X}, raising and lowering X, and casting. Never empty.
    /// </summary>
    [Key(6)]
    public List<ActionOptionMessage> Options { get; set; } = [];

    /// <summary>Whether the cost has {X}, so X is being chosen here rather than the cost simply being covered.</summary>
    [Key(7)]
    public bool HasX { get; set; }

    /// <summary>X so far. Zero for a spell without {X}.</summary>
    [Key(8)]
    public int X { get; set; }

    /// <summary>
    /// Mana in the pool already put towards the spell, by type and restriction. Still in the pool until the spell is
    /// cast, so a client showing the pool can say how much of it is spoken for.
    /// </summary>
    [Key(9)]
    public List<RestrictedManaView> Committed { get; set; } = [];

    /// <summary>
    /// Hybrid and Phyrexian symbols still owed (CR 107.4e-f), one entry per symbol, each payable another way too. Apart from
    /// <see cref="RemainingColored"/> because either colour will do, and a client that read only that list would think
    /// a cost of <c>{U/R}</c> owed nothing and give up paying for it.
    /// </summary>
    [Key(10)]
    public List<HybridSymbolView> RemainingHybrid { get; set; } = [];

    /// <summary>
    /// The spell's kicker cost in oracle notation (CR 702.33), or empty for none. A variable kicker uses
    /// <see cref="X"/> as its chosen value and <see cref="KickerPaid"/> to distinguish paid zero from declining it.
    /// </summary>
    [Key(11)]
    public string Kicker { get; set; } = string.Empty;

    /// <summary>Whether the kicker is multikicker (CR 702.33c), payable any number of times rather than once.</summary>
    [Key(12)]
    public bool Multikicker { get; set; }

    /// <summary>
    /// Whether that optional additional cost is buyback (CR 702.27) rather than kicker: the same payment, so
    /// <see cref="Kicker"/> still carries the cost and <see cref="X"/> whether it has been paid, but a client says
    /// "buy back" rather than "kick".
    /// </summary>
    [Key(13)]
    public bool Buyback { get; set; }

    /// <summary>
    /// Whether this is the announcement rather than the payment (CR 601.2b): <see cref="X"/> — the value of X, or with a
    /// <see cref="Kicker"/> how many times it is kicked — is being chosen before any target, and nothing is paid yet. Its
    /// only options are raising and lowering it, announcing it, and giving up; the payment is asked afterwards.
    /// </summary>
    [Key(14)]
    public bool AnnouncesXOrKicks { get; set; }

    /// <summary>
    /// For the announcement: the largest value the pool and the player's untapped mana sources could pay for, or -1
    /// when not even zero could be; for an X paid in life, the most the player could pay and still have life left.
    /// Advice only, like <see cref="ActionOptionMessage.MaxX"/>.
    /// </summary>
    [Key(15)]
    public int MostAffordable { get; set; }

    /// <summary>
    /// For the announcement: whether X is paid in life ("pay X life", CR 119.4) rather than mana, so a client says
    /// "life" and a bot spends it with the care life deserves.
    /// </summary>
    [Key(16)]
    public bool XPaidInLife { get; set; }

    /// <summary>
    /// Colorless mana symbols {C} still owed (CR 107.4c), so a client can tell that a colored or generic-only source
    /// would not help — parallel to <see cref="RemainingGeneric"/>, but for the count that must be paid with
    /// colorless mana specifically rather than with anything.
    /// </summary>
    [Key(17)]
    public int RemainingColorless { get; set; }

    /// <summary>Whether the optional {X} kicker was chosen, including at X=0.</summary>
    [Key(18)]
    public bool KickerPaid { get; set; }

    /// <summary>Whether the kicker's cost contains {X}, instead of X counting kicker payments.</summary>
    [Key(19)]
    public bool VariableKicker { get; set; }

    /// <summary>
    /// Whether no mana may be spent on this cost at all ("You can't spend mana to cast this spell", CR 601.2h, 106.4):
    /// only convoke's taps and delve's exiles pay it. Mana abilities are still offered (CR 601.2g), but their mana pays
    /// nothing here, so a client neither asks for lands or pool mana nor makes any.
    /// </summary>
    [Key(20)]
    public bool SpendsNoMana { get; set; }

    /// <summary>Giving up on the spell, which is always among the options (CR 733.1).</summary>
    public override AnswerPayload Default() => new TakeActionAnswer { Kind = ActionKindView.CancelCasting };
}

/// <summary>
/// One symbol with more than one printed way to pay it: a two-color hybrid symbol, payable with either colour
/// (CR 107.4e), or a Phyrexian symbol, payable with its one colour or with life (CR 107.4f).
/// </summary>
[MessagePackObject]
public sealed class HybridSymbolView
{
    /// <summary>The colours one mana of which could pay this symbol: both, for hybrid; the one, for Phyrexian.</summary>
    [Key(0)]
    public List<ColorView> Colors { get; set; } = [];

    /// <summary>The life that pays this symbol instead of mana (CR 107.4f), or zero for a plain hybrid symbol.</summary>
    [Key(1)]
    public int LifeCost { get; set; }

    public bool Accepts(ColorView color) => Colors.Contains(color);
}

/// <summary>Something a spell may target: a player, or an object named by handle.</summary>
[MessagePackObject]
public sealed class TargetMessage
{
    /// <summary>Set when the target is a player; zero otherwise.</summary>
    [Key(0)]
    public int Player { get; set; }

    /// <summary>Set when the target is an object; none otherwise.</summary>
    [Key(1)]
    public ViewHandle Object { get; set; }

    [IgnoreMember]
    public bool IsPlayer => Player != 0;
}

/// <summary>The active player must discard down to their maximum hand size (CR 514.1).</summary>
[MessagePackObject]
public sealed class DiscardPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    [Key(1)]
    public int Count { get; set; }

    [Key(2)]
    public int MaximumHandSize { get; set; }

    [Key(3)]
    public List<ViewHandle> Hand { get; set; } = [];

    /// <summary>The front of the hand, which is as good a choice as any when nobody is choosing.</summary>
    public override AnswerPayload Default() => new CardsChosenAnswer { Cards = [.. Hand.Take(Count)] };
}

/// <summary>The active player declares attackers (CR 508.1).</summary>
[MessagePackObject]
public sealed class DeclareAttackersPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    [Key(1)]
    public List<ViewHandle> Eligible { get; set; } = [];

    /// <summary>
    /// Seats that may be attacked. A list rather than a single defender, because each attacker picks its own
    /// target in a multiplayer game (CR 508.1).
    /// </summary>
    [Key(2)]
    public List<int> AttackablePlayers { get; set; } = [];

    /// <summary>Planeswalkers an opponent controls, each of which an attacker may be sent at instead (CR 508.1b).</summary>
    [Key(3)]
    public List<ViewHandle> AttackablePlaneswalkers { get; set; } = [];

    /// <summary>
    /// Battles a defending player protects, each of which an attacker may be sent at instead (CR 310.9b, 508.1b): a
    /// Siege the attacking player controls among them, since its protector is their opponent (CR 310.12a).
    /// </summary>
    [Key(5)]
    public List<ViewHandle> AttackableBattles { get; set; } = [];

    /// <summary>
    /// Creatures that must attack (CR 508.1d), as a goaded one must (CR 701.15b), each among <see cref="Eligible"/>. A
    /// declaration leaving one out, or sending it at a seat its requirement does not name, is refused.
    /// </summary>
    [Key(4)]
    public List<AttackRequirementMessage> Requirements { get; set; } = [];

    /// <summary>
    /// Pairs of an eligible creature and an offered seat, planeswalker or battle that may not be declared, because
    /// something says that one can't be attacked by that creature (CR 508.1c). A declaration naming one is refused. A
    /// seat, planeswalker or battle no eligible creature may attack is left out of the lists above instead.
    /// </summary>
    [Key(6)]
    public List<AttackMessage> Forbidden { get; set; } = [];

    /// <summary>Whether <paramref name="attack"/> is one of <see cref="Forbidden"/>'s pairs.</summary>
    public bool IsForbidden(AttackMessage attack) =>
        Forbidden.Any(refused => refused.Attacker == attack.Attacker
            && refused.TargetPermanent == attack.TargetPermanent
            && (!refused.TargetPermanent.IsNone || refused.TargetPlayer == attack.TargetPlayer));

    /// <summary>
    /// Every attack <paramref name="attacker"/> may be declared making, seats first: each offered seat, planeswalker
    /// and battle less the forbidden pairs.
    /// </summary>
    public List<AttackMessage> AttacksFor(ViewHandle attacker) =>
    [
        .. AttackablePlayers.Select(seat => new AttackMessage { Attacker = attacker, TargetPlayer = seat })
            .Concat(AttackablePlaneswalkers.Select(walker => new AttackMessage { Attacker = attacker, TargetPermanent = walker }))
            .Concat(AttackableBattles.Select(battle => new AttackMessage { Attacker = attacker, TargetPermanent = battle }))
            .Where(attack => !IsForbidden(attack)),
    ];

    /// <summary>Attacking with nobody but the creatures that must, each at the first thing it may.</summary>
    public override AnswerPayload Default() => new AttackersDeclaredAnswer
    {
        Attacks =
        [
            .. Requirements.Select(requirement => requirement.Players.Count > 0
                ? new AttackMessage { Attacker = requirement.Attacker, TargetPlayer = requirement.Players[0] }
                : AttacksFor(requirement.Attacker)[0]),
        ],
    };
}

/// <summary>
/// A creature that must attack, and where <see cref="Players"/> names any, must attack one of those seats: a goaded
/// creature attacks a player other than whoever goaded it (CR 701.15b). Empty means it may attack anything.
/// </summary>
[MessagePackObject]
public sealed class AttackRequirementMessage
{
    [Key(0)]
    public ViewHandle Attacker { get; set; }

    [Key(1)]
    public List<int> Players { get; set; } = [];

    /// <summary>Whether a declaration obeys this requirement: the same test the engine makes.</summary>
    public bool MetBy(IEnumerable<AttackMessage> attacks) =>
        attacks.Any(attack => attack.Attacker == Attacker
            && (Players.Count == 0 || (attack.TargetPermanent.IsNone && Players.Contains(attack.TargetPlayer))));
}

/// <summary>One defending player declares blockers (CR 509.1).</summary>
[MessagePackObject]
public sealed class DeclareBlockersPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    [Key(1)]
    public List<ViewHandle> Eligible { get; set; } = [];

    /// <summary>Attackers attacking this player: the only ones their creatures may block.</summary>
    [Key(2)]
    public List<ViewHandle> Attackers { get; set; } = [];

    /// <summary>
    /// Every blocker and attacker that may be paired. Narrower than <see cref="Eligible"/> against
    /// <see cref="Attackers"/> once a restriction such as flying (CR 702.9b) applies; a block outside this list is refused.
    /// </summary>
    [Key(3)]
    public List<BlockMessage> Allowed { get; set; } = [];

    /// <summary>
    /// The fewest creatures that may block each of <see cref="Attackers"/>, index for index: 2 for menace, 3 for "except
    /// by three or more", otherwise 1. A declaration giving an attacker some blockers but fewer is refused.
    /// </summary>
    [Key(4)]
    public List<int> MinimumBlockers { get; set; } = [];

    /// <summary>Blocking with nobody.</summary>
    public override AnswerPayload Default() => new BlockersDeclaredAnswer();
}

/// <summary>
/// This client divides the combat damage of each of their creatures that has more than one place to send it
/// (CR 510.1c-d, 702.19b): an attacker among its blockers and, with trample, the player it is attacking; a blocker
/// among the attackers it blocks. Answered with a <see cref="CombatDamageAssignedAnswer"/>.
/// </summary>
[MessagePackObject]
public sealed class AssignCombatDamagePromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>One per creature with a division to make, in combat order.</summary>
    [Key(1)]
    public List<DamageDivisionMessage> Divisions { get; set; } = [];

    /// <summary>The engine's own division of each creature's damage, which is always legal.</summary>
    public override AnswerPayload Default()
    {
        var answer = new CombatDamageAssignedAnswer();

        foreach (DamageDivisionMessage division in Divisions)
        {
            for (int i = 0; i < division.Creatures.Count; i++)
            {
                int amount = i < division.Suggested.Count ? division.Suggested[i] : 0;
                if (amount > 0)
                {
                    answer.Shares.Add(new DamageShareMessage { Source = division.Source, Creature = division.Creatures[i], Amount = amount });
                }
            }

            if (division.SuggestedToDefender > 0)
            {
                answer.Shares.Add(new DamageShareMessage { Source = division.Source, Creature = ViewHandle.None, Amount = division.SuggestedToDefender });
            }
        }

        return answer;
    }
}

/// <summary>One creature's combat damage and where it may go (CR 510.1).</summary>
[MessagePackObject]
public sealed class DamageDivisionMessage
{
    /// <summary>The attacking or blocking creature whose damage this is.</summary>
    [Key(0)]
    public ViewHandle Source { get; set; }

    /// <summary>How much it assigns: its power.</summary>
    [Key(1)]
    public int Damage { get; set; }

    /// <summary>What it may assign damage to: an attacker's blockers, or a blocker's attackers, in declaration order.</summary>
    [Key(2)]
    public List<ViewHandle> Creatures { get; set; } = [];

    /// <summary>
    /// Lethal damage for each creature, index for index: what must reach every one of them before any damage may go to
    /// the player (CR 702.19b). 1 from a source with deathtouch.
    /// </summary>
    [Key(3)]
    public List<int> Lethal { get; set; } = [];

    /// <summary>Trample: damage beyond lethal to every creature may go to the player being attacked instead.</summary>
    [Key(4)]
    public bool MayAssignToDefender { get; set; }

    /// <summary>The player the attacker is attacking, or 0 for a blocker or for an attacker attacking a planeswalker.</summary>
    [Key(5)]
    public int Defender { get; set; }

    /// <summary>The engine's own division, index for index with <see cref="Creatures"/>.</summary>
    [Key(6)]
    public List<int> Suggested { get; set; } = [];

    /// <summary>The rest of the engine's own division: what it would send to the player or planeswalker.</summary>
    [Key(7)]
    public int SuggestedToDefender { get; set; }

    /// <summary>The planeswalker the attacker is attacking, when the excess may go there instead (CR 702.19c).</summary>
    [Key(8)]
    public ViewHandle DefenderPermanent { get; set; }
}

/// <summary>Where one creature's combat damage goes, one share per recipient.</summary>
[MessagePackObject]
public sealed class DamageShareMessage
{
    [Key(0)]
    public ViewHandle Source { get; set; }

    /// <summary>The creature this share goes to, or no handle for the player the source is attacking.</summary>
    [Key(1)]
    public ViewHandle Creature { get; set; }

    [Key(2)]
    public int Amount { get; set; }
}

// --- Answers ---

[Union(0, typeof(PassAnswer))]
[Union(1, typeof(CardsChosenAnswer))]
[Union(2, typeof(TakeActionAnswer))]
[Union(3, typeof(AttackersDeclaredAnswer))]
[Union(4, typeof(BlockersDeclaredAnswer))]
[Union(5, typeof(CombatDamageAssignedAnswer))]
[Union(6, typeof(CardsDistributedAnswer))]
[Union(7, typeof(TargetsChosenAnswer))]
[Union(8, typeof(MayChosenAnswer))]
[Union(9, typeof(ModesChosenAnswer))]
[Union(10, typeof(ReplacementChosenAnswer))]
[Union(11, typeof(TriggersOrderedAnswer))]
[Union(12, typeof(CancelAnnouncementAnswer))]
[Union(13, typeof(CardNamedAnswer))]
[Union(14, typeof(PlayerChosenAnswer))]
[Union(15, typeof(ColorsChosenAnswer))]
[Union(16, typeof(TargetsDividedAnswer))]
public abstract class AnswerPayload;

/// <summary>Gives up the spell or ability being announced, in answer to any prompt that <see cref="PromptMessage.MayCancel"/>.</summary>
[MessagePackObject]
public sealed class CancelAnnouncementAnswer : AnswerPayload;

/// <summary>The modes chosen for a <see cref="ChooseModesPromptMessage"/>, as indexes into its modes.</summary>
[MessagePackObject]
public sealed class ModesChosenAnswer : AnswerPayload
{
    [Key(0)]
    public List<int> Modes { get; set; } = [];
}

/// <summary>Whether this client does what a <see cref="MayPromptMessage"/> offered.</summary>
[MessagePackObject]
public sealed class MayChosenAnswer : AnswerPayload
{
    [Key(0)]
    public bool Accept { get; set; }
}

/// <summary>The seat this client chooses for a <see cref="ChoosePlayerPromptMessage"/>: one of its candidates.</summary>
[MessagePackObject]
public sealed class PlayerChosenAnswer : AnswerPayload
{
    [Key(0)]
    public int Player { get; set; }
}

/// <summary>The colors this client chooses for a <see cref="ChooseColorsPromptMessage"/>: as many as it asked for.</summary>
[MessagePackObject]
public sealed class ColorsChosenAnswer : AnswerPayload
{
    [Key(0)]
    public List<ColorView> Colors { get; set; } = [];
}

/// <summary>The card name this client chooses for a <see cref="NameCardPromptMessage"/>: one of its names.</summary>
[MessagePackObject]
public sealed class CardNamedAnswer : AnswerPayload
{
    [Key(0)]
    public string Name { get; set; } = string.Empty;
}

/// <summary>
/// Which of the replacement effects offered by a <see cref="ChooseReplacementPromptMessage"/> applies first, as an
/// index into its sources.
/// </summary>
[MessagePackObject]
public sealed class ReplacementChosenAnswer : AnswerPayload
{
    [Key(0)]
    public int Index { get; set; }
}

/// <summary>
/// The order the triggered abilities of an <see cref="OrderTriggersPromptMessage"/> go on the stack, as indexes into
/// its sources: every index exactly once, the last one named resolving first.
/// </summary>
[MessagePackObject]
public sealed class TriggersOrderedAnswer : AnswerPayload
{
    [Key(0)]
    public List<int> Order { get; set; } = [];
}

/// <summary>
/// Where this client puts each of the cards it looked at: <see cref="Placements"/> runs index for index with the
/// prompt's destinations, and together the lists name every card looked at exactly once.
/// <para>
/// Within a list the cards are in the order they are put there, which for the top of a library means the first card
/// listed becomes the top card, and for the bottom means the last listed becomes the bottom card. With
/// <see cref="Shuffle"/> set the lists are ignored and the library is shuffled.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class CardsDistributedAnswer : AnswerPayload
{
    [Key(0)]
    public List<List<ViewHandle>> Placements { get; set; } = [];

    [Key(1)]
    public bool Shuffle { get; set; }
}

[MessagePackObject]
public sealed class AttackMessage
{
    [Key(0)]
    public ViewHandle Attacker { get; set; }

    /// <summary>The seat attacked, when <see cref="TargetPermanent"/> is none.</summary>
    [Key(1)]
    public int TargetPlayer { get; set; }

    /// <summary>The planeswalker attacked instead of a seat (CR 508.1b), or <see cref="ViewHandle.None"/>.</summary>
    [Key(2)]
    public ViewHandle TargetPermanent { get; set; }
}

[MessagePackObject]
public sealed class AttackersDeclaredAnswer : AnswerPayload
{
    [Key(0)]
    public List<AttackMessage> Attacks { get; set; } = [];
}

[MessagePackObject]
public sealed class BlockMessage
{
    [Key(0)]
    public ViewHandle Blocker { get; set; }

    [Key(1)]
    public ViewHandle Attacker { get; set; }
}

[MessagePackObject]
public sealed class BlockersDeclaredAnswer : AnswerPayload
{
    [Key(0)]
    public List<BlockMessage> Blocks { get; set; } = [];
}

/// <summary>
/// How each creature in an <see cref="AssignCombatDamagePromptMessage"/> divides its damage: one share per recipient,
/// adding up to the creature's damage. A share with no creature handle goes to the player the source is attacking.
/// </summary>
[MessagePackObject]
public sealed class CombatDamageAssignedAnswer : AnswerPayload
{
    [Key(0)]
    public List<DamageShareMessage> Shares { get; set; } = [];
}

[MessagePackObject]
public sealed class PassAnswer : AnswerPayload;

/// <summary>The target chosen for a triggered ability: one of the candidates, or none where that is allowed.</summary>
[MessagePackObject]
public sealed class TargetsChosenAnswer : AnswerPayload
{
    [Key(0)]
    public List<TargetMessage> Targets { get; set; } = [];
}

/// <summary>The targets and division chosen for a <see cref="ChooseDividedTargetsPromptMessage"/> (CR 601.2d).</summary>
[MessagePackObject]
public sealed class TargetsDividedAnswer : AnswerPayload
{
    [Key(0)]
    public List<TargetShareMessage> Shares { get; set; } = [];
}

[MessagePackObject]
public sealed class CardsChosenAnswer : AnswerPayload
{
    [Key(0)]
    public List<ViewHandle> Cards { get; set; } = [];
}

[MessagePackObject]
public sealed class TakeActionAnswer : AnswerPayload
{
    [Key(0)]
    public ActionKindView Kind { get; set; }

    [Key(1)]
    public ViewHandle Source { get; set; }

    [Key(2)]
    public int AbilityIndex { get; set; }

    /// <summary>Targets chosen as the spell is announced (CR 601.2c).</summary>
    [Key(3)]
    public List<TargetMessage> Targets { get; set; } = [];

    /// <summary>The color, for a mana ability that adds mana of any one color.</summary>
    [Key(5)]
    public ColorView? Color { get; set; }

    /// <summary>
    /// The colors, one per mana and repeats allowed, for a mana ability that adds mana "in any combination of" some
    /// colors (CR 106.1b). Empty for every other action.
    /// </summary>
    [Key(10)]
    public List<ColorView> Colors { get; set; } = [];

    /// <summary>
    /// The permanent sacrificed to pay a mana ability's cost (CR 118.3, 701.21a). Null for every other action, and for
    /// a cost that sacrifices its own source, which needs no choice.
    /// </summary>
    [Key(11)]
    public ViewHandle? Sacrifice { get; set; }

    /// <summary>The controlled permanent tapped for the mana ability's chosen tap cost.</summary>
    [Key(12)]
    public ViewHandle? Tap { get; set; }

    /// <summary>The pool mana to put towards a spell, when the action assigns some. Null otherwise.</summary>
    [Key(6)]
    public ManaTypeView? Mana { get; set; }

    /// <summary>What that mana may be spent on, when the action assigns some.</summary>
    [Key(8)]
    public ManaSpendFilterView Filter { get; set; }

    /// <summary>What spending that mana does to whatever it pays for, alongside <see cref="Filter"/>.</summary>
    [Key(9)]
    public ManaSpendEffectView Effect { get; set; }
}

/// <summary>
/// How a client talks to a game, whether that game is across a network or running in the same process.
/// <para>
/// Polling rather than callbacks or async: the web client drains this on a timer, so the client needs no
/// threading and no synchronisation context of its own. The WebSocket implementation keeps a reader filling a
/// queue; the loopback implementation hands messages straight over. Either way the client code is identical,
/// which is what makes "host and play" the same path as joining a server rather than a second one.
/// </para>
/// </summary>
public interface IGameTransport : IDisposable
{
    bool IsConnected { get; }

    void Send(ClientMessage message);

    /// <summary>Takes everything received since the last call. Empty when nothing has arrived.</summary>
    IReadOnlyList<ServerMessage> Receive();
}

/// <summary>
/// This client is searching its own library (CR 701.19).
/// <para>
/// The only message that carries a library's contents, and only ever to its owner while the search is outstanding.
/// Afterwards the server drops their handles, so the order after the shuffle cannot be matched against what was seen.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class SearchLibraryPromptMessage : PromptPayload
{
    [Key(0)]
    public int Player { get; set; }

    /// <summary>The required number of cards, or the most that may be taken when <see cref="Optional"/>.</summary>
    [Key(1)]
    public int Count { get; set; }

    /// <summary>Whether fewer may be taken, including none; true for a search with a stated quality (CR 701.23b).</summary>
    [Key(2)]
    public bool Optional { get; set; }

    [Key(3)]
    public List<ViewHandle> Library { get; set; } = [];

    /// <summary>The cards the search may take: all of <see cref="Library"/> for "a card", fewer for "a Plains or Island card".</summary>
    [Key(4)]
    public List<ViewHandle> Eligible { get; set; } = [];

    /// <summary>Where the chosen cards will go.</summary>
    [Key(5)]
    public SearchDestinationView Destination { get; set; }

    /// <summary>Whose library is searched: the searcher's own, or another player's (Deadly Cover-Up).</summary>
    [Key(6)]
    public int Owner { get; set; }

    /// <summary>Finding nothing, which a search always allows (CR 701.19d).</summary>
    public override AnswerPayload Default() => new CardsChosenAnswer();
}
