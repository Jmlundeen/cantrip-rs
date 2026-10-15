using Cantrip.Client;
using Cantrip.Client.Embedded;
using Cantrip.Core;
using Cantrip.Protocol;
using Cantrip.Server.Rules;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// What happens when a client sends something it should not.
/// <para>
/// The contract these pin down: a misbehaving or merely stale client gets an error back and the game is untouched.
/// Nothing a client can send makes the server disagree with itself, which is what "server authoritative" has to mean in
/// practice rather than as an intention. The rules are a scripted host's (<see cref="ScriptedHost"/>): a session's checks
/// of who owes which request are its own, and need no real engine to be exercised.
/// </para>
/// </summary>
public sealed class AuthorityTests
{
    private static readonly PlayerId One = new(1);
    private static readonly PlayerId Two = new(2);

    private static readonly RulesSeat[] Seats =
    [
        new("Alice", ["Island", "Island", "Lightning Bolt"]),
        new("Bob", ["Swamp"]),
    ];

    private static GameSession Session()
    {
        var session = new GameSession(NativeRulesHost.NewGame(new ScriptedHost(), Seats, 5, new RulesGameOptions()));
        session.Start();
        return session;
    }

    private static int OutstandingRequest(GameSession session, PlayerId seat)
    {
        foreach (ServerMessage message in session.Drain(seat))
        {
            if (message is PromptMessage prompt)
            {
                return prompt.RequestId;
            }
        }

        throw new InvalidOperationException($"Seat {seat.Value} has no outstanding prompt.");
    }

    private static ErrorMessage SoleError(GameSession session, PlayerId seat)
    {
        List<ErrorMessage> errors = [];
        foreach (ServerMessage message in session.Drain(seat))
        {
            if (message is ErrorMessage error)
            {
                errors.Add(error);
            }
        }

        return Assert.Single(errors);
    }

    [Fact]
    public void Answering_a_request_that_belongs_to_another_seat_is_refused()
    {
        using GameSession session = Session();
        int request = OutstandingRequest(session, One);

        session.Submit(Two, new AnswerMessage { RequestId = request, Answer = new PassAnswer() });

        Assert.Equal(ErrorCode.NotYourTurnToAnswer, SoleError(session, Two).Code);

        // And the game has not moved: the request is still outstanding for the seat that owes it.
        session.Submit(One, new AnswerMessage { RequestId = request, Answer = new PassAnswer() });
        Assert.DoesNotContain(session.Drain(One), message => message is ErrorMessage);
    }

    [Fact]
    public void Answering_the_same_request_twice_is_refused()
    {
        // Routine after a reconnect rather than hostile, which is why it is an error message and not a disconnect.
        using GameSession session = Session();
        int request = OutstandingRequest(session, One);

        session.Submit(One, new AnswerMessage { RequestId = request, Answer = new PassAnswer() });
        _ = session.Drain(One);

        session.Submit(One, new AnswerMessage { RequestId = request, Answer = new PassAnswer() });

        Assert.Equal(ErrorCode.StaleReply, SoleError(session, One).Code);
    }

    [Fact]
    public void Answering_an_unknown_request_is_refused()
    {
        using GameSession session = Session();

        session.Submit(One, new AnswerMessage { RequestId = 987654, Answer = new PassAnswer() });

        Assert.Equal(ErrorCode.StaleReply, SoleError(session, One).Code);
    }

    [Fact]
    public void An_answer_the_rules_refuse_is_reported_with_their_code_and_the_game_goes_on()
    {
        using GameSession session = Session();
        int request = OutstandingRequest(session, One);

        // A card choice to a priority prompt. The rules are the authority on legality, so their refusal is what is
        // reported rather than a second check out here that might disagree with it.
        session.Submit(One, new AnswerMessage { RequestId = request, Answer = new CardsChosenAnswer() });

        Assert.Equal(ErrorCode.IllegalAnswer, SoleError(session, One).Code);
        Assert.False(session.IsOver);
    }

    [Fact]
    public void A_protocol_mismatch_is_reported_at_the_handshake()
    {
        using GameSession session = Session();

        session.Submit(One, new JoinMessage { ProtocolVersion = 999, PlayerName = "stale build" });

        ErrorMessage error = SoleError(session, One);
        Assert.Equal(ErrorCode.ProtocolMismatch, error.Code);
        Assert.Contains("999", error.Detail, StringComparison.Ordinal);
    }

    [Fact]
    public void Answering_after_the_game_is_over_is_refused()
    {
        using GameSession session = Session();
        int request = OutstandingRequest(session, One);
        session.Concede(Two);
        _ = session.Drain(One);

        Assert.True(session.IsOver);
        session.Submit(One, new AnswerMessage { RequestId = request, Answer = new PassAnswer() });

        Assert.Equal(ErrorCode.StaleReply, SoleError(session, One).Code);
    }

    [Fact]
    public void A_client_cannot_answer_through_another_clients_transport()
    {
        // The transport binds a seat, so a client has no way to speak as someone else even if it tried.
        using var session = new GameSession(NativeRulesHost.NewGame(new ScriptedHost(), Seats, 5, new RulesGameOptions()));
        var second = new GameClient(new LoopbackTransport(session, Two));

        session.Start();
        second.Pump();

        // Seat two was not asked anything, so it has nothing to answer.
        Assert.False(second.IsWaitingOnMe);
        Assert.Throws<InvalidOperationException>(second.Pass);
    }
}
