using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>When a counterspell is spent: <c>policy.counterbar</c>.</summary>
public sealed class TacticalCounterTests
{
    private static TacticalPlay With(double bar) =>
        new(TacticalModel.Prior().With([KeyValuePair.Create("policy.counterbar", bar)]), seed: 1);

    private static ObjectView OnStack(int handle, string name, int manaValue, int power, int toughness, string text = "")
    {
        ObjectView spell = InHand(handle, name, CardTypeView.Creature, manaValue, text, power, toughness);
        spell.Zone = ZoneKindView.Stack;
        spell.ZoneOwner = 0;
        spell.Owner = Them;
        spell.Controller = Them;
        return spell;
    }

    private static PriorityPromptMessage Counter(int spell) => new()
    {
        Player = Me,
        Options =
        [
            new()
            {
                Kind = ActionKindView.CastSpell, Source = Handle(10), Affordable = true,
                TargetsRequired = 1, TargetCandidates = [At(spell)],
            },
        ],
    };

    private static GameClient Board(ObjectView spell, int theirHand)
    {
        GameClient client = Seated([Land(1), Land(2), spell,
            InHand(10, "Counterspell", CardTypeView.Instant, 2, "Counter target spell.", color: ColorView.Blue)], active: Them);
        client.View!.Players.Find(player => player.Seat == Them)!.HandCount = theirHand;
        return client;
    }

    [Fact]
    public void Keeps_the_counterspell_for_something_bigger_while_the_opponent_holds_cards()
    {
        ObjectView bear = OnStack(20, "Bear", 2, 2, 2);

        Assert.IsType<TakeActionAnswer>(With(0).Decide(Board(bear, 4), Counter(20)));
        Assert.IsType<PassAnswer>(With(2).Decide(Board(bear, 4), Counter(20)));

        // Their last card: whatever it is, it is the one to counter.
        Assert.IsType<TakeActionAnswer>(With(2).Decide(Board(bear, 0), Counter(20)));

        ObjectView dragon = OnStack(21, "Dragon", 6, 6, 6, "Flying");
        Assert.IsType<TakeActionAnswer>(With(2).Decide(Board(dragon, 4), Counter(21)));
    }

    [Fact]
    public void Draw_go_keeps_an_instant_for_the_end_of_their_turn_while_holding_a_counterspell()
    {
        GameClient client = Seated([Land(1), Land(2), Land(3), Land(4),
            InHand(10, "Counterspell", CardTypeView.Instant, 2, "Counter target spell.", color: ColorView.Blue),
            InHand(11, "Deep Thought", CardTypeView.Instant, 2, "Draw two cards.", color: ColorView.Blue)]);
        var prompt = new PriorityPromptMessage { Player = Me, Options = [new() { Kind = ActionKindView.CastSpell, Source = Handle(11), Affordable = true }] };

        Assert.Equal(Handle(11), Assert.IsType<TakeActionAnswer>(new TacticalPlay(TacticalModel.Prior(), seed: 1).Decide(client, prompt)).Source);
        Assert.IsType<PassAnswer>(new TacticalPlay(TacticalModel.Prior().With([KeyValuePair.Create("policy.drawgo", 1.0)]), seed: 1).Decide(client, prompt));

        // At the end of their turn it is cast.
        GameClient theirs = Seated([Land(1), Land(2), Land(3), Land(4),
            InHand(10, "Counterspell", CardTypeView.Instant, 2, "Counter target spell.", color: ColorView.Blue),
            InHand(11, "Deep Thought", CardTypeView.Instant, 2, "Draw two cards.", color: ColorView.Blue)], active: Them, step: TurnStepView.End);
        Assert.Equal(Handle(11), Assert.IsType<TakeActionAnswer>(new TacticalPlay(TacticalModel.Prior().With([KeyValuePair.Create("policy.drawgo", 1.0)]), seed: 1).Decide(theirs, prompt)).Source);
    }

    [Fact]
    public void Tells_why_a_counterspell_was_kept_and_why_one_was_cast()
    {
        ObjectView bear = OnStack(20, "Bear", 2, 2, 2);

        var kept = new StringWriter();
        TacticalPlay keeping = With(2);
        keeping.Narrator = kept;
        Assert.IsType<PassAnswer>(keeping.Decide(Board(bear, 4), Counter(20)));
        Assert.Contains("Counterspell kept: Bear is worth", kept.ToString(), StringComparison.Ordinal);
        Assert.Contains("more for the 4 cards they hold", kept.ToString(), StringComparison.Ordinal);

        var cast = new StringWriter();
        TacticalPlay casting = With(0);
        casting.Narrator = cast;
        Assert.IsType<TakeActionAnswer>(casting.Decide(Board(bear, 4), Counter(20)));
        Assert.Contains("Counterspell on Bear: worth", cast.ToString(), StringComparison.Ordinal);

        // Without the mana for it, it says so rather than nothing.
        PriorityPromptMessage broke = Counter(20);
        broke.Options[0].Affordable = false;
        var poor = new StringWriter();
        TacticalPlay waiting = With(0);
        waiting.Narrator = poor;
        Assert.IsType<PassAnswer>(waiting.Decide(Board(bear, 4), broke));
        Assert.Contains("Counterspell in hand against Bear: not enough mana", poor.ToString(), StringComparison.Ordinal);
    }

    [Fact]
    public void Phases_its_board_out_with_Teferis_Protection_against_a_sweeper_and_for_nothing_else()
    {
        const string protection = "Until your next turn, your life total can't change and you gain protection from everything. "
            + "All permanents you control phase out.\nExile Teferi's Protection.";
        ObjectView wrath = OnStack(20, "Wrath of God", 4, 0, 0, "Destroy all creatures. They can't be regenerated.");
        wrath.CardTypes = [CardTypeView.Sorcery];
        wrath.Power = null;
        wrath.Toughness = null;
        var cast = new PriorityPromptMessage { Player = Me, Options = [new() { Kind = ActionKindView.CastSpell, Source = Handle(10), Affordable = true }] };
        ObjectView Protection() => InHand(10, "Teferi's Protection", CardTypeView.Instant, 3, protection, color: ColorView.White);

        GameClient swept = Seated([Land(1), Land(2), Land(3), Creature(4, "Mine", 3, 3), wrath, Protection()], active: Them);
        Assert.Equal(Handle(10), Assert.IsType<TakeActionAnswer>(With(0).Decide(swept, cast)).Source);

        // Against a Wrath that would take more of theirs, and with nothing on the stack, it is kept.
        GameClient theirs = Seated([Land(1), Land(2), Land(3), Creature(4, "Theirs", 3, 3, Them), wrath, Protection()], active: Them);
        Assert.IsType<PassAnswer>(With(0).Decide(theirs, cast));
        GameClient idle = Seated([Land(1), Land(2), Land(3), Creature(4, "Mine", 3, 3), Protection()]);
        Assert.IsType<PassAnswer>(With(0).Decide(idle, cast));
    }
}
