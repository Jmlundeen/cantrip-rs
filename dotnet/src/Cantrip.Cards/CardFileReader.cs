using System.Text.Json;
using Cantrip.Cards.Model;

namespace Cantrip.Cards;

/// <summary>A card file that could not be read, with every problem found in it.</summary>
public sealed class CardFileException : Exception
{
    public CardFileException(IReadOnlyList<string> problems)
        : base(string.Join(Environment.NewLine, problems))
    {
        Problems = problems;
    }

    public IReadOnlyList<string> Problems { get; }
}

/// <summary>
/// Reads a card file's printed characteristics: the name, cost, type line, power and toughness, loyalty and defense,
/// rules text, color indicator, and the other faces a card carries (back face, Adventure, Omen, split halves, prepare spell).
/// <para>
/// Only that. The rules of a card live in the native engine's card export, which reads the same files for itself, so
/// what is read here is what a drafter, a deck builder and a card list need: a card as it is printed. Every other key
/// of a card file is left alone, and none of it is checked.
/// </para>
/// </summary>
public static class CardFileReader
{
    private static readonly JsonDocumentOptions DocumentOptions = new()
    {
        CommentHandling = JsonCommentHandling.Skip,
        AllowTrailingCommas = true,
    };

    /// <summary>Reads the one card a file holds.</summary>
    /// <exception cref="CardFileException">The file is not valid JSON or a characteristic does not parse.</exception>
    public static CardDefinition LoadFile(string path)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(path);
        return Load(File.ReadAllText(path));
    }

    /// <summary>Reads one card object.</summary>
    /// <exception cref="CardFileException">The text is not a card object or a characteristic does not parse.</exception>
    public static CardDefinition Load(string json)
    {
        ArgumentNullException.ThrowIfNull(json);

        JsonDocument document;
        try
        {
            document = JsonDocument.Parse(json, DocumentOptions);
        }
        catch (JsonException error)
        {
            throw new CardFileException([$"not valid JSON: {error.Message}"]);
        }

        using (document)
        {
            if (document.RootElement.ValueKind != JsonValueKind.Object)
            {
                throw new CardFileException(["a card file holds exactly one card, as a JSON object."]);
            }

            List<string> problems = [];
            CardDefinition? card = Build(document.RootElement, parentId: null, problems);

            if (card is null || problems.Count > 0)
            {
                throw new CardFileException(problems);
            }

            return card;
        }
    }

    private static CardDefinition? Build(JsonElement entry, CardId? parentId, List<string> problems)
    {
        string name = Text(entry, "name")?.Trim() ?? string.Empty;
        CardId id = parentId ?? new CardId(name);

        if (id.IsNone)
        {
            problems.Add("name: a name is required: a card's name is its id.");
            return null;
        }

        List<CardDefinition> halves = [];
        if (entry.TryGetProperty("halves", out JsonElement halvesElement) && halvesElement.ValueKind == JsonValueKind.Array)
        {
            foreach (JsonElement half in halvesElement.EnumerateArray())
            {
                if (Build(half, id, problems) is { } built)
                {
                    halves.Add(built);
                }
            }
        }

        bool split = halves.Count > 0;

        ManaCost cost = ManaCost.None();
        TypeLine typeLine = new();

        if (split)
        {
            // CR 709.4: a split card's cost and type line are its halves' combined.
            cost = halves.Skip(1).Aggregate(halves[0].ManaCost, (total, half) => total.Plus(half.ManaCost));
            typeLine = Combined(halves);
        }
        else
        {
            try
            {
                cost = ManaCost.Parse(Text(entry, "manaCost"));
            }
            catch (ManaCostFormatException error)
            {
                problems.Add($"{id}: manaCost: {error.Message}");
            }

            try
            {
                typeLine = TypeLineParser.Parse(Text(entry, "type"), Strings(entry, "subtypes"));
            }
            catch (TypeLineFormatException error)
            {
                problems.Add($"{id}: type: {error.Message}");
            }
        }

        List<ManaColor> colorIndicator = [];
        foreach (string color in Strings(entry, "colorIndicator") ?? [])
        {
            if (Enum.TryParse(color, ignoreCase: true, out ManaColor parsed) && Enum.IsDefined(parsed))
            {
                colorIndicator.Add(parsed);
            }
            else
            {
                problems.Add($"{id}: colorIndicator: '{color}' is not a color.");
            }
        }

        List<DraftAbility> draftAbilities = [];
        foreach (string ability in Strings(entry, "draftAbilities") ?? [])
        {
            if (Enum.TryParse(ability, ignoreCase: true, out DraftAbility parsed) && Enum.IsDefined(parsed) && !int.TryParse(ability, out _))
            {
                draftAbilities.Add(parsed);
            }
            else
            {
                problems.Add($"{id}: draftAbilities: '{ability}' is not a draft ability.");
            }
        }

        return new CardDefinition
        {
            Id = id,
            Name = name,
            ManaCost = cost,
            TypeLine = typeLine,
            Power = Int(entry, "power"),
            Toughness = Int(entry, "toughness"),
            Loyalty = Int(entry, "loyalty"),
            Defense = Int(entry, "defense"),
            StationPt = Text(entry, "stationPt") ?? string.Empty,
            ColorIndicator = colorIndicator,
            OracleText = Text(entry, "oracleText") ?? string.Empty,
            BackFace = Face(entry, "backFace", id, problems),
            BackFacePlayableAsLand = entry.TryGetProperty("backFacePlayableAsLand", out JsonElement land) && land.ValueKind == JsonValueKind.True,
            Adventure = Face(entry, "adventure", id, problems),
            Omen = Face(entry, "omen", id, problems),
            Prepare = Face(entry, "prepare", id, problems),
            Halves = halves,
            DraftAbilities = draftAbilities,
        };
    }

    private static CardDefinition? Face(JsonElement entry, string key, CardId id, List<string> problems) =>
        Property(entry, key) is { ValueKind: JsonValueKind.Object } face ? Build(face, id, problems) : null;

    private static JsonElement? Property(JsonElement entry, string key)
    {
        foreach (JsonProperty property in entry.EnumerateObject())
        {
            if (string.Equals(property.Name, key, StringComparison.OrdinalIgnoreCase))
            {
                return property.Value;
            }
        }

        return null;
    }

    private static string? Text(JsonElement entry, string key) =>
        Property(entry, key) is { ValueKind: JsonValueKind.String } value ? value.GetString() : null;

    private static int? Int(JsonElement entry, string key) =>
        Property(entry, key) is { ValueKind: JsonValueKind.Number } value && value.TryGetInt32(out int number) ? number : null;

    private static List<string>? Strings(JsonElement entry, string key) =>
        Property(entry, key) is { ValueKind: JsonValueKind.Array } value
            ? [.. value.EnumerateArray().Where(item => item.ValueKind == JsonValueKind.String).Select(item => item.GetString()!)]
            : null;

    /// <summary>
    /// The type line a split card has off the stack (CR 709.4): every supertype, card type and subtype either half
    /// prints, each once and in printed order.
    /// </summary>
    private static TypeLine Combined(IReadOnlyList<CardDefinition> halves)
    {
        var combined = new TypeLine();

        foreach (CardDefinition half in halves)
        {
            foreach (Supertype supertype in half.TypeLine.Supertypes.Where(supertype => !combined.Has(supertype)))
            {
                combined.Supertypes.Add(supertype);
            }

            foreach (CardType type in half.TypeLine.CardTypes.Where(type => !combined.Is(type)))
            {
                combined.CardTypes.Add(type);
            }

            foreach (string subtype in half.TypeLine.Subtypes.Where(subtype => !combined.HasSubtype(subtype)))
            {
                combined.Subtypes.Add(subtype);
            }
        }

        return combined;
    }
}
