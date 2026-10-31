using Microsoft.Playwright;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// A link opened again in the tab it was opened in. A page takes its credential - a seat's ticket, a tablet's device
/// token - out of its address as it loads, so the link differs from the address left only after its hash, and a browser
/// given it only moves to the fragment. The page loads again instead and takes the credential as a first load does: the
/// way back for a tablet or a seat whose page has stuck, with the link it was given.
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class LinkAgainTests(WebClient web)
{
    private static readonly float Patience = 90_000;

    [Fact]
    public async Task A_tablets_link_opened_again_in_its_tab_loads_the_page_again_with_its_token()
    {
        await using TabletDriver tablet = await TabletDriver.OpenAsync(web, "#device=first", WebClient.Pictures(nameof(LinkAgainTests)), "tablet");
        Assert.Equal("first", await tablet.Page.EvaluateAsync<string>("() => globalThis.cantripDevice"));

        await OpenAgainAsync(tablet.Page, "device=second");

        await tablet.Page.WaitForFunctionAsync("() => globalThis.cantripDevice === 'second'", null, new PageWaitForFunctionOptions { Timeout = Patience });
        Assert.DoesNotContain("second", await tablet.Page.EvaluateAsync<string>("() => location.href"), StringComparison.Ordinal);
    }

    [Fact]
    public async Task A_seats_link_opened_again_in_its_tab_loads_the_page_again_with_its_ticket()
    {
        await using IBrowserContext context = await web.Browser.NewContextAsync();
        IPage page = await context.NewPageAsync();
        await page.GotoAsync($"{web.Site}?server={Uri.EscapeDataString("ws://localhost:1/game")}#ticket=first");
        await page.WaitForFunctionAsync("() => globalThis.cantripTicket === 'first'", null, new PageWaitForFunctionOptions { Timeout = Patience });

        await OpenAgainAsync(page, "ticket=second");

        await page.WaitForFunctionAsync("() => globalThis.cantripTicket === 'second'", null, new PageWaitForFunctionOptions { Timeout = Patience });
        Assert.DoesNotContain("second", await page.EvaluateAsync<string>("() => location.href"), StringComparison.Ordinal);
    }

    /// <summary>
    /// Gives the tab its link again - its address with this after the hash, as typing or tapping the link does - and
    /// waits for the page to load anew: a marker set on the old page is gone from the new one.
    /// </summary>
    private static async Task OpenAgainAsync(IPage page, string fragment)
    {
        await page.EvaluateAsync("() => { globalThis.cantripOldPage = true; }");
        string address = await page.EvaluateAsync<string>("() => location.href");
        var loaded = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        void Loaded(object? sender, IPage _) => loaded.TrySetResult();
        page.Load += Loaded;
        await page.EvaluateAsync("a => { location.href = a; }", $"{address.Split('#')[0]}#{fragment}");
        await loaded.Task.WaitAsync(TimeSpan.FromMilliseconds(Patience));
        page.Load -= Loaded;
        Assert.False(await page.EvaluateAsync<bool>("() => globalThis.cantripOldPage === true"));
    }
}
