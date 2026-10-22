using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// A declaration of blockers is settled with the host before it is sent (<see cref="GameClient.Answer"/>): checked
/// (<see cref="CheckBlocksMessage"/>), sent once the host calls it legal, and mended a block at a time from what the host says
/// may be added when it does not - so a client that declares without knowing a requirement (CR 509.1c: "blocks this turn if
/// able", Lure) or a restriction (menace) still declares what the rules take.
/// </summary>
public sealed class BlockSettlingTests
{
    private static ViewHandle H(int value) => new(value);

    private static BlockMessage B(int blocker, int attacker) => new() { Blocker = H(blocker), Attacker = H(attacker) };

    /// <summary>A transport whose host judges every check with <paramref name="rule"/>, as the native host does.</summary>
    private sealed class Judging(Func<IReadOnlyList<BlockMessage>, BlockVerdictView> rule) : IGameTransport
    {
        public Queue<ServerMessage> Inbound { get; } = [];

        public List<ClientMessage> Sent { get; } = [];

        public bool IsConnected => true;

        public void Send(ClientMessage message)
        {
            Sent.Add(message);
            if (message is CheckBlocksMessage check)
            {
                Inbound.Enqueue(new BlocksVerdictMessage { RequestId = check.RequestId, Blocks = check.Blocks, Verdict = rule(check.Blocks) });
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
    /// Attackers 1 and 2, blockers 10 and 11, every pair allowed; creature 10 blocks this turn if able (CR 509.1c), so a
    /// declaration is legal only where 10 blocks something.
    /// </summary>
    private static BlockVerdictView TenMustBlock(IReadOnlyList<BlockMessage> blocks)
    {
        List<BlockMessage> all = [B(10, 1), B(10, 2), B(11, 1), B(11, 2)];
        bool complete = blocks.Any(block => block.Blocker == H(10));
        return new BlockVerdictView
        {
            Complete = complete,
            Addable = [.. all.Where(pair => !blocks.Any(block => block.Blocker == pair.Blocker))],
            Issue = complete ? null : new BlockIssueView { Creature = H(10), Words = "must block", Rule = "509.1c" },
        };
    }

    private static (GameClient Client, Judging Transport) Asked(Func<IReadOnlyList<BlockMessage>, BlockVerdictView> rule)
    {
        var transport = new Judging(rule);
        var client = new GameClient(transport);
        transport.Inbound.Enqueue(new PromptMessage
        {
            RequestId = 9,
            Prompt = new DeclareBlockersPromptMessage
            {
                Player = 1,
                Eligible = [H(10), H(11)],
                Attackers = [H(1), H(2)],
                Allowed = [B(10, 1), B(10, 2), B(11, 1), B(11, 2)],
                MinimumBlockers = [1, 1],
                Requirements = [new BlockMessage { Blocker = H(10), Attacker = ViewHandle.None }],
                MostRequirements = 1,
                Suggested = [B(10, 1)],
                Verdict = rule([]),
            },
        });
        client.Pump();
        return (client, transport);
    }

    private static List<(int, int)>? Answered(Judging transport) =>
        transport.Sent.OfType<AnswerMessage>().LastOrDefault()?.Answer is BlockersDeclaredAnswer declared
            ? [.. declared.Blocks.Select(block => (block.Blocker.Value, block.Attacker.Value))]
            : null;

    [Fact]
    public void A_declaration_the_host_takes_is_sent_once_checked()
    {
        (GameClient client, Judging transport) = Asked(TenMustBlock);
        client.Answer(new BlockersDeclaredAnswer { Blocks = [B(10, 2), B(11, 2)] });

        Assert.True(client.IsSettling);
        Assert.Null(Answered(transport));
        Assert.Single(transport.Sent.OfType<CheckBlocksMessage>());

        client.Pump();
        Assert.False(client.IsSettling);
        Assert.Equal([(10, 2), (11, 2)], Answered(transport));
    }

    [Fact]
    public void No_blocks_where_the_prompt_says_none_is_legal_go_at_once()
    {
        (GameClient client, Judging transport) = Asked(_ => new BlockVerdictView { Complete = true });
        client.Answer(new BlockersDeclaredAnswer());

        Assert.False(client.IsSettling);
        Assert.Empty(transport.Sent.OfType<CheckBlocksMessage>());
        Assert.Equal([], Answered(transport));
    }

    /// <summary>
    /// A declaration that leaves the provoked creature out is refused: the blocks meant are kept and the required creature's
    /// block is added from what may be, the engine's own first.
    /// </summary>
    [Fact]
    public void A_declaration_the_host_refuses_is_mended_keeping_the_blocks_meant()
    {
        (GameClient client, Judging transport) = Asked(TenMustBlock);
        client.Answer(new BlockersDeclaredAnswer { Blocks = [B(11, 2)] });
        for (int i = 0; i < 10 && client.IsSettling; i++)
        {
            client.Pump();
        }

        Assert.False(client.IsSettling);
        Assert.Equal([(11, 2), (10, 1)], Answered(transport));
        Assert.NotEmpty(client.Mends);
    }

    /// <summary>Declaring nothing, where the prompt's own verdict says that is no declaration, is mended the same way.</summary>
    [Fact]
    public void Declaring_nothing_where_a_creature_must_block_is_mended()
    {
        (GameClient client, Judging transport) = Asked(TenMustBlock);
        client.Answer(new BlockersDeclaredAnswer());
        for (int i = 0; i < 10 && client.IsSettling; i++)
        {
            client.Pump();
        }

        Assert.Equal([(10, 1)], Answered(transport));
    }
}
