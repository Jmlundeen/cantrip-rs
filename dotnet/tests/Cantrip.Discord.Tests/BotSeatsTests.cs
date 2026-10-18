using Xunit;

namespace Cantrip.Discord.Tests;

public sealed class BotSeatsTests
{
    [Theory]
    [InlineData("wss://cantrip.example.com/table/7781/game", "127.0.0.1", "ws://127.0.0.1:7781/game")]
    [InlineData("wss://cantrip.example.com/base/table/7782/game", "0.0.0.0", "ws://127.0.0.1:7782/game")]
    [InlineData("wss://cantrip.example.com/table/7783/game", "10.0.0.5", "ws://10.0.0.5:7783/game")]
    public void A_bot_reaches_a_table_behind_the_proxy_on_its_own_port(string given, string listen, string expected) =>
        Assert.Equal(new Uri(expected), BotSeats.Direct(new Uri(given), listen));

    [Fact]
    public void An_address_that_is_not_behind_the_proxy_is_left_as_it_is()
    {
        var address = new Uri("ws://play.example.com:7781/game");
        Assert.Equal(address, BotSeats.Direct(address, "127.0.0.1"));
    }
}
