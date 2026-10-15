using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// Scenarios played in the browser (docs/scenarios-in-the-browser.md): every <c>.cantrip</c> file marked <c># Browser</c>,
/// Bot1 a page clicking its way through the script (<see cref="ScenarioPlayer"/>). <c>CANTRIP_SCENARIO=name</c> plays that
/// one scenario instead, marked or not - with <c>CANTRIP_WEB_HEADED=1</c>, in a browser to watch
/// (<c>tools\scenario-in-browser.cmd</c>).
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class ScenarioInBrowserTests(WebClient web)
{
    /// <summary>The marker that puts a scenario in the browser suite.</summary>
    public const string Marker = "# Browser";

    public static TheoryData<string> Marked()
    {
        var data = new TheoryData<string>();
        string? one = Environment.GetEnvironmentVariable("CANTRIP_SCENARIO");
        if (!string.IsNullOrWhiteSpace(one))
        {
            data.Add(one.Trim().Replace(".cantrip", string.Empty, StringComparison.Ordinal));
            return data;
        }

        foreach (string path in Directory.EnumerateFiles(Path.Combine(WebClient.Root, "scenarios"), "*.cantrip").Order(StringComparer.Ordinal))
        {
            if (File.ReadLines(path).Any(line => line.TrimEnd() == Marker))
            {
                data.Add(Path.GetFileNameWithoutExtension(path));
            }
        }

        return data;
    }

    [Theory]
    [MemberData(nameof(Marked))]
    public async Task A_scenario_plays_through_the_browser(string name)
    {
        string? failure = await ScenarioPlayer.PlayAsync(web, Path.Combine(WebClient.Root, "scenarios", name + ".cantrip"));
        Assert.True(failure is null, $"{name}:\n{failure}");
    }
}
