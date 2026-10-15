namespace Cantrip.Core;

/// <summary>
/// The keys of the choices kept on an object (CR 614.12a): a quality chosen about a permanent — as it enters, or
/// later — that the permanent's own abilities read back as "the chosen name", "the chosen color", "the chosen
/// quality".
/// <para>
/// A key is card vocabulary rather than a rule, so a card invents its own and nothing central has to learn it. Only
/// <see cref="Name"/> is shared, because the engine's own card-name question, the shared naming recipe and the
/// client's label for it all have to agree on one spelling.
/// </para>
/// </summary>
public static class ObjectChoice
{
    /// <summary>A card name chosen about the object (CR 201.4), as Anointed Peacekeeper's.</summary>
    public const string Name = "name";

    /// <summary>
    /// What a client prints in front of a choice's value. "Chosen name" for <see cref="Name"/>, and "Chosen
    /// &lt;key&gt;" for anything else, so a card that invents a key gets a readable label without editing this.
    /// </summary>
    public static string Label(string key) => key == Name ? "Chosen name" : $"Chosen {key}";
}
