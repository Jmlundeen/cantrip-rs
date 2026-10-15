using System.Text.Json;
using System.Text.Json.Nodes;
using Cantrip.Cards.Model;

namespace Cantrip.Datasets;

/// <summary>
/// A set, in the product sense: a named list of cards that already exist, each as this set prints it.
/// <para>
/// A set never defines a card. Every name it lists has to be defined in some dataset's <c>cards/</c> folder, and a
/// set that lists a name nobody defines fails the load. What a set may do is specialise a card it lists: a sparse
/// override of the card's fields, merged over the one definition (RFC 7386 JSON merge patch), for when this printing
/// differs — and eventually for what every printing differs in, like its image. The card keeps its name, which is its
/// identity, so a game built with a set's overrides still has exactly one card under each name.
/// </para>
/// <para>
/// So a cube is a set: a list of cards that exist, drawn from whichever datasets define them, with no data duplicated.
/// </para>
/// </summary>
public sealed class SetList
{
    internal SetList(
        string code,
        string name,
        string datasetId,
        string path,
        IReadOnlyList<CardId> cards,
        IReadOnlyDictionary<CardId, CardDefinition> overrides)
    {
        Code = code;
        Name = name;
        DatasetId = datasetId;
        Path = path;
        Cards = cards;
        Overrides = overrides;
    }

    /// <summary>The set's code, unique across every loaded dataset (compared ignoring case).</summary>
    public string Code { get; }

    public string Name { get; }

    /// <summary>The dataset whose <c>sets/</c> folder holds the list. Diagnostics only: a set is found by its code.</summary>
    public string DatasetId { get; }

    /// <summary>The file the list was read from.</summary>
    public string Path { get; }

    /// <summary>The cards the set lists, in the order it lists them.</summary>
    public IReadOnlyList<CardId> Cards { get; }

    /// <summary>
    /// The cards this set specialises, as definitions with the set's override already merged in and linted. A listed
    /// card absent from here is printed exactly as it is defined.
    /// </summary>
    public IReadOnlyDictionary<CardId, CardDefinition> Overrides { get; }

    public override string ToString() => $"{Code} ({Cards.Count} cards)";
}

/// <summary>RFC 7386 JSON merge patch, which is what a set's override is.</summary>
internal static class MergePatch
{
    public static readonly JsonDocumentOptions DocumentOptions = new()
    {
        CommentHandling = JsonCommentHandling.Skip,
        AllowTrailingCommas = true,
    };

    /// <summary>
    /// <paramref name="patch"/> applied to <paramref name="target"/>: an object merges key by key, a null removes the
    /// key, and anything else — an array included — replaces what was there. Neither input is modified.
    /// </summary>
    public static JsonNode? Apply(JsonNode? target, JsonNode? patch)
    {
        if (patch is not JsonObject patchObject)
        {
            return patch?.DeepClone();
        }

        var result = target is JsonObject targetObject ? (JsonObject)targetObject.DeepClone() : new JsonObject();

        foreach ((string key, JsonNode? value) in patchObject)
        {
            if (value is null)
            {
                result.Remove(key);
                continue;
            }

            JsonNode? merged = Apply(result[key], value);
            result.Remove(key);
            result[key] = merged;
        }

        return result;
    }
}
