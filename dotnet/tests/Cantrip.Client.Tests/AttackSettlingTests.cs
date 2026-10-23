using Cantrip.Client.Presentation;
using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// A declaration of attackers is settled with the host before it is sent (<see cref="GameClient.Answer"/>), as blocks are:
/// checked (<see cref="CheckAttacksMessage"/>), sent once the host calls it legal, and mended from what the host says may be
/// added - so a client that declares without counting CR 508.1d's requirements still declares what the rules take; and a
/// question whose check the host refused is answered with the engine's own declaration rather than the same proposal again.
/// </summary>
public sealed class AttackSettlingTests
{
    private static ViewHandle H(int value) => new(value);

    private static AttackMessage At(int attacker, int seat) => new() { Attacker = H(attacker), TargetPlayer = seat };

    /// <summary>A transport whose host judges every check as the gallery's stand-in does, or refuses them all.</summary>
    private sealed class Judging(DeclareAttackersPromptMessage prompt, ErrorCode? refuse = null) : IGameTransport
    {
        public Queue<ServerMessage> Inbound { get; } = [];

        public List<ClientMessage> Sent { get; } = [];

        public bool IsConnected => true;

        public void Send(ClientMessage message)
        {
            Sent.Add(message);
            if (message is CheckAttacksMessage check)
            {
                Inbound.Enqueue(refuse is { } code
                    ? new ErrorMessage { Code = code, Detail = "refused", RequestId = check.RequestId }
                    : new AttacksVerdictMessage { RequestId = check.RequestId, Attacks = check.Attacks, Verdict = FixtureTransport.StandInAttackVerdict(prompt, check.Attacks) });
            }
        }

        public IReadOnlyList<ServerMessage> Receive()
        {
            List<ServerMessage> messages = [.. Inbound];
            Inbound.Clear();
            return messages;
        }

        public void Dispose()
        {
        }
    }

    /// <summary>
    /// Creature 1 goaded by seat 2 and told to attack seat 2 this combat, in a game of three (seats 2 and 3 to attack): attacking
    /// either seat obeys two of its three requirements; not attacking obeys none.
    /// </summary>
    private static DeclareAttackersPromptMessage Goaded() => FixtureTransport.JudgeAttackers(new DeclareAttackersPromptMessage
    {
        Player = 1,
        Eligible = [H(1), H(4)],
        AttackablePlayers = [2, 3],
        Requirements =
        [
            new AttackRequirementMessage { Attacker = H(1) },
            new AttackRequirementMessage { Attacker = H(1), OtherThan = 2 },
            new AttackRequirementMessage { Attacker = H(1), TargetPlayer = 2 },
        ],
    });

    private static (GameClient Client, Judging Transport) Asked(DeclareAttackersPromptMessage prompt, ErrorCode? refuse = null)
    {
        var transport = new Judging(prompt, refuse);
        var client = new GameClient(transport);
        transport.Inbound.Enqueue(new PromptMessage { RequestId = 5, Prompt = prompt });
        client.Pump();
        return (client, transport);
    }

    private static List<(int, int)>? Answered(Judging transport) =>
        transport.Sent.OfType<AnswerMessage>().LastOrDefault()?.Answer is AttackersDeclaredAnswer declared
            ? [.. declared.Attacks.Select(attack => (attack.Attacker.Value, attack.TargetPlayer))]
            : null;

    [Fact]
    public void The_stand_in_counts_the_requirements_as_the_rules_do()
    {
        DeclareAttackersPromptMessage prompt = Goaded();
        Assert.Equal(2, prompt.MostRequirements);
        Assert.False(prompt.Verdict.Complete);
        Assert.True(FixtureTransport.StandInAttackVerdict(prompt, [At(1, 2)]).Complete);
        Assert.True(FixtureTransport.StandInAttackVerdict(prompt, [At(1, 3)]).Complete);
    }

    [Fact]
    public void Either_seat_is_sent_once_checked()
    {
        (GameClient client, Judging transport) = Asked(Goaded());
        client.Answer(new AttackersDeclaredAnswer { Attacks = [At(1, 3)] });
        Assert.True(client.IsSettling);
        client.Pump();
        Assert.Equal([(1, 3)], Answered(transport));
    }

    [Fact]
    public void Leaving_the_goaded_creature_home_is_mended_keeping_the_attacks_meant()
    {
        (GameClient client, Judging transport) = Asked(Goaded());
        client.Answer(new AttackersDeclaredAnswer { Attacks = [At(4, 3)] });
        for (int i = 0; i < 10 && client.IsSettling; i++)
        {
            client.Pump();
        }

        Assert.False(client.IsSettling);
        List<(int, int)> sent = Answered(transport)!;
        Assert.Contains((4, 3), sent);
        Assert.Contains(sent, attack => attack.Item1 == 1);
        Assert.NotEmpty(client.Mends);
    }

    /// <summary>
    /// A seat whose check is refused is asked again, and if it proposes the same again it is not checked a second time: the
    /// engine's own declaration goes, so an automatic seat cannot go round forever.
    /// </summary>
    [Fact]
    public void A_question_whose_check_was_refused_is_answered_with_the_engines_declaration_the_second_time()
    {
        (GameClient client, Judging transport) = Asked(Goaded(), ErrorCode.UnknownHandle);
        client.Answer(new AttackersDeclaredAnswer { Attacks = [At(4, 3)] });
        client.Pump();
        Assert.Null(Answered(transport));
        Assert.Equal(5, client.Prompt?.RequestId);

        client.Answer(new AttackersDeclaredAnswer { Attacks = [At(4, 3)] });
        Assert.Equal([(1, 2)], Answered(transport));
        Assert.Single(transport.Sent.OfType<CheckAttacksMessage>());
    }
}
