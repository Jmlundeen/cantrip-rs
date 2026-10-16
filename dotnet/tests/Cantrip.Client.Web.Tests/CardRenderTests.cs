using Microsoft.Playwright;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// Pictures of card faces, drawn by the real client in a real browser: the page opened with <c>?cardsheet</c> draws a
/// sheet of sample faces with <c>CardFace</c>, and this saves it as TestResults\web\CardRenderTests\card-sheet.png. It
/// asserts nothing about the pixels; it is run (tools\card-render.cmd) to look at a change to the template.
/// </summary>
[Collection(WebClientCollection.Name)]
[Trait("Category", "WallClock")]
public sealed class CardRenderTests(WebClient web)
{
    [Fact]
    public async Task The_card_sheet_is_drawn()
    {
        IBrowserContext context = await web.Browser.NewContextAsync(new BrowserNewContextOptions
        {
            ViewportSize = new ViewportSize { Width = 1760, Height = 1950 },
            DeviceScaleFactor = 2,
        });
        IPage page = await context.NewPageAsync();
        var errors = new List<string>();
        page.PageError += (_, error) => errors.Add(error);

        await page.GotoAsync($"{web.Site}?cardsheet");
        await page.WaitForFunctionAsync("() => globalThis.cantripCardSheet === 'ready'", null, new PageWaitForFunctionOptions { Timeout = 90_000 });
        // Art and fonts arrive after the first draw; the sheet is drawn again as they do.
        await Task.Delay(1500);
        await page.EvaluateAsync("() => document.querySelector('.uno-loader')?.remove()");
        await Task.Delay(500);

        string path = Path.Combine(WebClient.Pictures(nameof(CardRenderTests)), "card-sheet.png");
        await page.ScreenshotAsync(new PageScreenshotOptions { Path = path });
        await context.CloseAsync();

        Assert.Empty(errors);
        Assert.True(File.Exists(path));
    }
}
