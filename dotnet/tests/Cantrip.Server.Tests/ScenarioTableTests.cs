using Cantrip.Client;
using Cantrip.Protocol;
using Cantrip.Server.Host;
using Cantrip.Server.Rules;
using Cantrip.TestKit;
using Xunit;
using Xunit.Abstractions;

namespace Cantrip.Server.Tests;

/// <summary>
/// Scenarios played at a real table (docs/scenarios-in-the-browser.md): a real server dealing through the real native
/// host, the script answering Bot2, and Bot1 a protocol client over a real socket sending exactly what each step says the
/// script means. What a browser seat does with clicks, done here with the answers themselves: it proves the table's half,
/// so a browser run that fails is the page's. On the wall clock because it is a socket and a process; it skips, saying
/// so, when the host binary has not been built.
/// </summary>
[Trait("Category", "WallClock")]
public sealed class ScenarioTableTests(ITestOutputHelper output)
{
    private static readonly TimeSpan Patience = TimeSpan.FromSeconds(30);

    [Theory]
    [InlineData("duress-takes-the-named-card")]
    [InlineData("black-lotus-sacrificed-for-mana-is-taken-back-and-can-be-used-again")]
    [InlineData("lotus-petal-and-a-forest-are-taken-back-latest-first")]
    [InlineData("a-land-tapped-while-paying-is-taken-back-and-the-cast-once-paid-is-not")]
    [InlineData("pentad-prism-spending-a-charge-counter-is-taken-back")]
    [InlineData("a-treasure-sacrificed-for-mana-is-taken-back")]
    [InlineData("lions-eye-diamond-and-chromatic-star-are-not-taken-back")]
    public async Task A_scenario_passes_with_bot1_a_client_at_a_real_table(string name)
    {
        if (!File.Exists(NativeHostProcess.Locate()))
        {
            output.WriteLine($"Skipped: no cantrip-native-host at {NativeHostProcess.Locate()}.");
            return;
        }

        ScenarioVerdict verdict = await PlayAsync(name, (client, expected) => client.Answer(expected.Answer!));
        Assert.True(verdict.Passed, verdict.Report);
    }

    /// <summary>The table's half of strictness: a seat answering other than the script means fails the scenario at the line.</summary>
    [Fact]
    public async Task A_client_answering_otherwise_fails_the_scenario_at_the_line()
    {
        if (!File.Exists(NativeHostProcess.Locate()))
        {
            output.WriteLine($"Skipped: no cantrip-native-host at {NativeHostProcess.Locate()}.");
            return;
        }

        ScenarioVerdict verdict = await PlayAsync(
            "duress-takes-the-named-card",
            (client, expected) => client.Answer(expected.Line is null ? expected.Answer! : new PassAnswer()));
        Assert.False(verdict.Passed);
        Assert.Contains("Bot1 answered", verdict.Report, StringComparison.Ordinal);
    }

    private static async Task<ScenarioVerdict> PlayAsync(string name, Action<GameClient, ScenarioExpected> answer)
    {
        ScenarioSpec scenario = ScenarioSpec.Load(Path.Combine(RepoRoot.Path, "scenarios", name + ".cantrip"), [1]);
        await using ScenarioTable table = await ScenarioTable.StartAsync(scenario, RepoRoot.ContentRoot);
        var client = new GameClient(await WebSocketTransport.ConnectAsync(table.Address));
        client.Join("You", 1);
        try
        {
            while (true)
            {
                ScenarioProgress progress = await table.NextAsync(Patience);
                if (progress.Verdict is { } verdict)
                {
                    return verdict;
                }

                if (progress.Expected is not { Seat: 1 } expected)
                {
                    continue;
                }

                await PromptedAsync(client, expected.RequestId);
                if (expected.Undo)
                {
                    client.Answer(TakeUndo(client.Prompt!.Prompt));
                }
                else
                {
                    answer(client, expected);
                }
            }
        }
        finally
        {
            client.Disconnect();
        }
    }

    /// <summary>Pumps the client until it holds the question the table is asking.</summary>
    private static async Task PromptedAsync(GameClient client, int requestId)
    {
        DateTime deadline = DateTime.UtcNow + Patience;
        while (client.Prompt?.RequestId != requestId)
        {
            if (DateTime.UtcNow > deadline)
            {
                throw new TimeoutException($"The client never got request {requestId}; it holds {client.Prompt?.Describe() ?? "nothing"}.");
            }

            if (client.Pump() == 0)
            {
                await Task.Delay(5);
            }
        }
    }

    private static TakeActionAnswer TakeUndo(PromptPayload prompt)
    {
        List<ActionOptionMessage> options = prompt switch
        {
            PriorityPromptMessage priority => priority.Options,
            PayCostsPromptMessage pay => pay.Options,
            _ => [],
        };
        ActionOptionMessage undo = options.Single(option => option.Kind == ActionKindView.UndoManaAbility);
        return new TakeActionAnswer { Kind = undo.Kind, Source = undo.Source };
    }
}
