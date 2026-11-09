using Cantrip.Client.Embedded;
using Cantrip.Client;
using Cantrip.Decks;
using Cantrip.Protocol;
using Cantrip.Server.Rules;
using Cantrip.Server.Host;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>Explicit ordered decks through the actual embedded deal and native sidecar; no random seat decisions.</summary>
[Trait("Category", "WallClock")]
public sealed class CommanderNativeSetupTests
{
    private sealed class OrderedNative(IHostTransport process) : IRulesHostFactory
    {
        public IRulesHost Game { get; private set; } = null!;
        public TaskCompletionSource<IReadOnlyList<RulesSeat>> Dealt { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
        public IRulesHost Create(IReadOnlyList<RulesSeat> seats, ulong seed, RulesGameOptions options)
        {
            Game = NativeRulesHost.NewGame(process, seats, seed, new RulesGameOptions
            {
                MinimumDeckSize = options.MinimumDeckSize,
                Commander = options.Commander,
                FirstPlayer = 1,
                Shuffle = false,
            });
            Dealt.TrySetResult(seats);
            return Game;
        }
    }

    [Fact]
    public async Task The_actual_lobby_validates_and_deals_the_explicit_commander_sections()
    {
        string binary = NativeHostProcess.Locate();
        Assert.True(File.Exists(binary), $"Build the current private native host first: {binary}");
        using NativeHostProcess process = NativeHostProcess.Start(binary);
        var factory = new OrderedNative(process);
        // Use the current catalog's complete legality entries here; the embedded test separately proves Signet mana.
        DeckList deck = DeckList.Parse("white", "Commander\nKytheon, Hero of Akros\nDeck\n99 Plains", "test");
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Port = 0, Players = 2, AutoStart = true, ContentRoot = RepoRoot.ContentRoot,
            Decks = [deck], Rules = factory, Format = DeckFormat.Find("commander")!,
        });
        using WebSocketTransport firstTransport = await WebSocketTransport.ConnectAsync(new Uri($"ws://127.0.0.1:{server.Port}/game"));
        using WebSocketTransport secondTransport = await WebSocketTransport.ConnectAsync(new Uri($"ws://127.0.0.1:{server.Port}/game"));
        var first = new GameClient(firstTransport);
        var second = new GameClient(secondTransport);
        first.Join("A", 1);
        second.Join("B", 2);
        IReadOnlyList<RulesSeat> seats = await factory.Dealt.Task.WaitAsync(TimeSpan.FromSeconds(10));
        Assert.Equal(2, seats.Count);
        Assert.All(seats, seat =>
        {
            Assert.Equal(["Kytheon, Hero of Akros"], seat.Commanders);
            Assert.Equal(99, seat.Deck.Count);
            Assert.DoesNotContain("Kytheon, Hero of Akros", seat.Deck);
        });
    }

    [Fact]
    public void Production_commander_deal_sets_forty_life_command_zone_and_real_signet_mana()
    {
        string binary = NativeHostProcess.Locate();
        Assert.True(File.Exists(binary), $"Build the current private native host first: {binary}");
        using NativeHostProcess process = NativeHostProcess.Start(binary);
        var factory = new OrderedNative(process);
        DeckList white = DeckList.Parse("white", "Commander\nKytheon, Hero of Akros\nDeck\n2 Plains\nArcane Signet\n96 Plains", "test");
        DeckList other = DeckList.Parse("other", "Commander\nIsamaru, Hound of Konda\nDeck\n99 Plains", "test");
        using HostedGame hosted = HostAndPlay.StartTable(factory, [white, other], DeckFormat.Find("commander")!, 1, ["A", "B"]);
        IRulesHost game = factory.Game;
        RulesStep step = game.Start();
        Assert.All(step.Views[1].Players, player => Assert.Equal(40, player.Life));
        Assert.Equal(["Kytheon, Hero of Akros", "Isamaru, Hound of Konda"],
            step.Views[1].Objects.Where(card => card.Zone == ZoneKindView.Command).OrderBy(card => card.Owner).Select(card => card.Name));
        int lands = 0;
        bool announced = false;
        for (int count = 0; count < 250; count++)
        {
            Assert.False(step.Finished);
            RulesRequest request = Assert.Single(step.Requests);
            AnswerPayload answer = request.Prompt.Default();
            if (request.Seat == 1 && request.Prompt is PriorityPromptMessage priority)
            {
                PlayerView view = step.Views[1];
                string Name(ActionOptionMessage option) => view.Objects.Single(card => card.Handle == option.Source).Name;
                if (priority.Options.FirstOrDefault(option => option.Kind == ActionKindView.ActivateAbility && Name(option) == "Arcane Signet") is { } signet)
                {
                    Assert.Equal([ColorView.White], signet.ColorChoices);
                    TakeActionAnswer activation = TakeActionAnswer.Take(signet);
                    activation.Color = ColorView.White;
                    step = game.Answer(request.RequestId, 1, activation);
                    Assert.Equal(1, step.Views[1].Players.Single(player => player.Seat == 1).Mana[(int)ManaTypeView.White]);
                    Assert.Equal(1, step.Views[1].Players.Single(player => player.Seat == 1).Mana.Sum());
                    Assert.True(step.Views[1].Objects.Single(card => card.Name == "Arcane Signet" && card.Zone == ZoneKindView.Battlefield).Tapped);
                    return;
                }
                if (lands < 2 && priority.Options.FirstOrDefault(option => option.Kind == ActionKindView.PlayLand) is { } land)
                {
                    lands++;
                    answer = TakeActionAnswer.Take(land);
                }
                else if (lands == 2 && !announced && priority.Options.FirstOrDefault(option => option.Kind == ActionKindView.CastSpell && Name(option) == "Arcane Signet") is { } cast)
                {
                    announced = true;
                    answer = TakeActionAnswer.Take(cast);
                }
            }
            else if (request.Seat == 1 && request.Prompt is PayCostsPromptMessage costs)
            {
                ActionOptionMessage option = costs.Options.FirstOrDefault(option => option.Kind == ActionKindView.ActivateAbility)
                    ?? costs.Options.FirstOrDefault(option => option.Kind == ActionKindView.AssignMana)
                    ?? costs.Options.First(option => option.Kind == ActionKindView.ConfirmCasting);
                TakeActionAnswer payment = TakeActionAnswer.Take(option);
                if (option.Kind == ActionKindView.AssignMana) payment.Mana = option.Produces[0];
                answer = payment;
            }
            step = game.Answer(request.RequestId, request.Seat, answer);
        }
        Assert.Fail("The forced production deal never reached the Signet activation.");
    }
}
