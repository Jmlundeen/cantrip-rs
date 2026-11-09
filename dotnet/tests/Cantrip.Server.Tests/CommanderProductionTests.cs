using Cantrip.Client.Embedded;
using Cantrip.Decks;
using Xunit;

namespace Cantrip.Server.Tests;

public sealed class CommanderProductionTests
{
    [Fact]
    public void The_real_embedded_deal_preserves_each_seats_selected_commanders()
    {
        var factory = new CapturingRulesFactory();
        DeckList white = DeckList.Parse("white", "Commander\nKytheon, Hero of Akros\nDeck\n98 Plains\nArcane Signet", "test");
        DeckList green = DeckList.Parse("green", "Commander\nOmnath, Locus of Mana\nDeck\n98 Forest\nArcane Signet", "test");
        using HostedGame hosted = HostAndPlay.StartTable(factory, [white, green], DeckFormat.Find("commander")!, 1, ["A", "B", "C"]);
        Assert.Equal(["Kytheon, Hero of Akros"], factory.Seats[0].Commanders);
        Assert.Equal(["Omnath, Locus of Mana"], factory.Seats[1].Commanders);
        Assert.Equal(["Kytheon, Hero of Akros"], factory.Seats[2].Commanders);
        Assert.All(factory.Seats, seat => Assert.Equal(99, seat.Deck.Count));
        Assert.DoesNotContain("Kytheon, Hero of Akros", factory.Seats[0].Deck);
        Assert.DoesNotContain("Omnath, Locus of Mana", factory.Seats[1].Deck);
    }
}
