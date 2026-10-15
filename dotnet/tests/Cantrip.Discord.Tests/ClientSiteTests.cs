using System.Net;
using System.Net.Sockets;
using Cantrip.Art;
using Xunit;

namespace Cantrip.Discord.Tests;

/// <summary>The web client every ticket reply opens, as the bot serves it, and the link that carries a seat to it.</summary>
public sealed class ClientSiteTests
{
    [Fact]
    public void The_link_names_the_table_and_carries_the_ticket_after_the_hash_so_no_request_ever_holds_it()
    {
        string ticket = new('C', 64);
        Uri link = ClientSite.Link(
            new Uri("http://example.net:7779/"), new Uri("ws://example.net:7782/game"), ticket, new Uri("http://example.net:7779/"));

        Assert.Equal("http://example.net:7779/", link.GetLeftPart(UriPartial.Path));
        Assert.DoesNotContain(ticket, link.Query, StringComparison.Ordinal);

        // Read back the way the client reads it: its query for the table, and the fragment for the ticket (Ticket.js).
        Dictionary<string, string> query = Pairs(link.Query.TrimStart('?'));
        Assert.Equal("ws://example.net:7782/game", query["server"]);
        Assert.Equal("http://example.net:7779/", query["art"]);
        Assert.Equal(ticket, Pairs(link.Fragment.TrimStart('#'))["ticket"]);
    }

    [Fact]
    public async Task The_bot_serves_the_client_and_its_art_from_one_port()
    {
        string root = Directory.CreateTempSubdirectory("cantrip-client-site").FullName;
        try
        {
            await File.WriteAllTextAsync(Path.Combine(root, "index.html"), "<!doctype html><title>Cantrip</title>");
            var probe = new TcpListener(IPAddress.Loopback, 0);
            probe.Start();
            int port = ((IPEndPoint)probe.LocalEndpoint).Port;
            probe.Stop();

            var art = new ArtStore(new ArtStoreOptions { ContentRoot = root, CacheDirectory = Path.Combine(root, "art") });
            await using var site = await ClientSite.StartAsync("127.0.0.1", port, root, art);
            using var http = new HttpClient();

            HttpResponseMessage page = await http.GetAsync(new Uri($"http://127.0.0.1:{port}/"));
            Assert.Equal(HttpStatusCode.OK, page.StatusCode);
            Assert.Contains("Cantrip", await page.Content.ReadAsStringAsync(), StringComparison.Ordinal);

            // Art for a card there is none of is a 404 from the art route, not the site's missing-file answer.
            HttpResponseMessage missing = await http.GetAsync(new Uri($"http://127.0.0.1:{port}/art/256/no-such-card.webp"));
            Assert.Equal(HttpStatusCode.NotFound, missing.StatusCode);
            Assert.Equal("*", missing.Headers.GetValues("Access-Control-Allow-Origin").Single());
        }
        finally
        {
            Directory.Delete(root, recursive: true);
        }
    }

    private static Dictionary<string, string> Pairs(string text) => text.Split('&')
        .Select(pair => pair.Split('=', 2))
        .ToDictionary(pair => pair[0], pair => Uri.UnescapeDataString(pair[1]));
}
