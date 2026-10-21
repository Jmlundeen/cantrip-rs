using Cantrip.Protocol;

namespace Cantrip.Server.Rules;

/// <summary>
/// The whole of what a <see cref="GameSession"/> asks of a rules engine, in the wire's own vocabulary and nothing
/// engine-shaped.
/// <para>
/// A session's job is clients: mailboxes, who may answer what, asking again after a refusal, bringing a seat back up to
/// date. Everything about the rules — what is legal, what each seat may see, what happened — comes through here as
/// <c>Cantrip.Protocol</c> types, already projected for each seat and already in each seat's own handle space. That is
/// what lets the same session serve the native engine's sidecar (<see cref="NativeRulesHost"/>) and a test's double with
/// no second code path, and what makes this the shape of the sidecar's protocol: one advance in, one
/// <see cref="RulesStep"/> out.
/// </para>
/// </summary>
public interface IRulesHost : IDisposable
{
    /// <summary>The seats, numbered from one, in turn order.</summary>
    IReadOnlyList<int> Seats { get; }

    /// <summary>Each seat's name, index for index with <see cref="Seats"/>.</summary>
    IReadOnlyList<string> SeatNames { get; }

    /// <summary>The team a seat plays for (CR 810), for ending a game from outside its rules.</summary>
    int Team(int seat);

    /// <summary>
    /// A seat's deck as the game began, each card as printed, for its welcome: by name, with how many, in name order.
    /// Nothing in it names an object in the game.
    /// </summary>
    IReadOnlyList<DeckCardView> Decklist(int seat);

    /// <summary>Starts the game, replaying any history it was given, and says what the opening produced.</summary>
    RulesStep Start();

    /// <summary>
    /// Answers an outstanding request for a seat.
    /// </summary>
    /// <exception cref="RulesRejectedException">
    /// The answer was refused and the game is as it was: the request is still outstanding, except for a
    /// <see cref="ErrorCode.StaleReply"/>, which names one that is not.
    /// </exception>
    RulesStep Answer(int requestId, int seat, AnswerPayload answer);

    /// <summary>Whether a seat may concede now: only when that ends the game (CR 104.3a, 800.4a).</summary>
    bool CanConcede(int seat);

    /// <summary>The seat concedes; the game ends with everyone else winning.</summary>
    RulesStep Concede(int seat);

    /// <summary>
    /// The seat loses and leaves the game from outside it - its match clock has run out - as a concession does (CR 104.3a),
    /// whether or not that ends the game: in a game of more than two, the others play on without it (CR 800.4a).
    /// </summary>
    RulesStep Leave(int seat);

    /// <summary>A seat's current view, for a client being brought back up to date. Nothing advances.</summary>
    PlayerView View(int seat);

    /// <summary>A card as printed, by the id a view names it by, or null for a card the rules do not know.</summary>
    ObjectView? Oracle(string cardId);

    bool IsOver { get; }

    GameOutcomeView Outcome { get; }

    /// <summary>The team that won, or zero while the game goes on or ended in a draw.</summary>
    int WinningTeam { get; }

    /// <summary>
    /// Every answer given through <see cref="Answer"/> since the game started, in order: what a game is resumed from,
    /// by starting again from the same deal and giving them again.
    /// </summary>
    IReadOnlyList<RulesAnswer> History { get; }
}

/// <summary>What one advance of the game produced, projected for every seat.</summary>
/// <param name="Requests">
/// Every question outstanding, not only the new ones: when several are outstanding at once — each defending player
/// declaring blocks — every answer reports the ones still waiting too, and the session sends only what it has not sent.
/// </param>
/// <param name="Views">Each seat's view of the position now, by seat.</param>
/// <param name="Events">
/// What happened since the last step as each seat may know it, by seat, each event with the sequence number of the
/// entry it came from, so a later retraction can name it and the same event reads as one log line for every seat.
/// </param>
/// <param name="Sequence">How far through the log every seat has now been brought.</param>
/// <param name="Finished">Whether the game is over.</param>
public sealed record RulesStep(
    IReadOnlyList<RulesRequest> Requests,
    IReadOnlyDictionary<int, PlayerView> Views,
    IReadOnlyDictionary<int, IReadOnlyList<(long Sequence, GameEventView View)>> Events,
    long Sequence,
    bool Finished,
    ScenarioProgress? Scenario = null);

/// <summary>One question outstanding, in the handle space of the seat it is for.</summary>
/// <param name="RequestId">The host's id for it, echoed back with the answer. Never reused within a game.</param>
/// <param name="MayCancel">Asked while the seat is still announcing something it may give up (<see cref="PromptMessage.MayCancel"/>).</param>
/// <param name="Presentation">Wording facts for the client, or null where there are none (<see cref="PromptMessage.Presentation"/>).</param>
public sealed record RulesRequest(
    int RequestId,
    int Seat,
    PromptPayload Prompt,
    bool MayCancel,
    PromptPresentationContext? Presentation);

/// <summary>An answer a seat gave, as it gave it.</summary>
public sealed record RulesAnswer(int Seat, AnswerPayload Answer);

/// <summary>The rules refused an answer, with the code the client is to be told and why.</summary>
public sealed class RulesRejectedException : Exception
{
    public RulesRejectedException(ErrorCode code, string detail) : base(detail)
    {
        Code = code;
        Detail = detail;
    }

    public ErrorCode Code { get; }

    public string Detail { get; }
}
