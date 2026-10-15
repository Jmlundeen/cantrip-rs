using System.Text.Json;
using Cantrip.Client.Tactics;

namespace Cantrip.BotLab;

/// <summary>
/// Says what the tactical seat makes of every card in a dataset: which spells it reads an effect off, and which it
/// does not.
/// <para>
/// The seat plays a card it cannot read badly or not at all, and which cards those are is not something a win rate
/// says. This lists them, so that teaching <see cref="CardReading"/> a new shape starts from the shapes there are most
/// of rather than from whichever card someone happened to notice.
/// </para>
/// </summary>
public static class ReadLab
{
    public static int Run(string root, string[] args)
    {
        ArgumentNullException.ThrowIfNull(args);

        string dataset = Array.IndexOf(args, "--dataset") is >= 0 and int at && at + 1 < args.Length ? args[at + 1] : "mtg";
        bool all = args.Contains("--all");
        var options = new JsonDocumentOptions { CommentHandling = JsonCommentHandling.Skip, AllowTrailingCommas = true };

        List<(string Name, string Type, string Text, string Read)> spells = [];

        foreach (string path in Directory.GetFiles(Path.Combine(root, "content", dataset, "cards"), "*.json").Order(StringComparer.Ordinal))
        {
            using JsonDocument card = JsonDocument.Parse(File.ReadAllText(path), options);
            JsonElement at0 = card.RootElement;
            string name = at0.TryGetProperty("name", out JsonElement n) ? n.GetString() ?? string.Empty : string.Empty;
            string type = at0.TryGetProperty("type", out JsonElement t) ? t.GetString() ?? string.Empty : string.Empty;
            string text = at0.TryGetProperty("oracleText", out JsonElement o) ? o.GetString() ?? string.Empty : string.Empty;

            if (!type.Contains("Instant", StringComparison.Ordinal) && !type.Contains("Sorcery", StringComparison.Ordinal))
            {
                continue;
            }

            spells.Add((name, type, text, Read(text)));
        }

        foreach (IGrouping<string, (string Name, string Type, string Text, string Read)> group in spells.GroupBy(spell => spell.Read).OrderByDescending(group => group.Count()))
        {
            Console.WriteLine($"{group.Count(),4}  {group.Key}");
        }

        Console.WriteLine();
        foreach ((string name, string _, string text, string read) in spells.Where(spell => all || spell.Read.StartsWith("UNREAD", StringComparison.Ordinal)))
        {
            Console.WriteLine($"{read,-18} {name}: {text.ReplaceLineEndings(" / ")}");
        }

        return 0;
    }

    /// <summary>How the seat reads one spell: what its text is taken to do, in a word.</summary>
    private static string Read(string text)
    {
        IReadOnlyList<string> modes = CardReading.Modes(text);
        if (modes.Count > 0)
        {
            return modes.Any(mode => Read(mode) is var read && !read.StartsWith("UNREAD", StringComparison.Ordinal)) ? "modal" : "UNREAD modal";
        }

        EffectRead effect = CardReading.Effect(text);
        bool targets = CardReading.Plain(text).Contains("target", StringComparison.Ordinal);

        if (effect.Kind != EffectKind.Unknown)
        {
            return effect.Kind.ToString();
        }

        if (CardReading.Draws(text) > 0)
        {
            return "draw";
        }

        if (CardReading.Tokens(text).Count > 0)
        {
            return "tokens";
        }

        if (CardReading.Selects(text))
        {
            return "select";
        }

        return targets ? "UNREAD targeted" : "UNREAD";
    }
}
