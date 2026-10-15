using SkiaSharp;

namespace Cantrip.Client.Web;

/// <summary>
/// Card faces laid out once and shared by every place that shows the same face at the same size: the 209 Gleemox of a
/// library search are one recording, and a card moving from the hand to the battlefield finds its face already made. Keyed
/// by everything drawn on the face (<see cref="Faces.LookOf"/>) and the size it is laid out at, so two cards share a
/// recording exactly when they would look the same.
/// <para>
/// A durable face - the card as it is, nothing on it (<see cref="Faces.IsDurable"/>) - is kept for the session and never
/// let go to make room. The rest, faces of the moment, are kept most recently used first up to a memory budget. A face drawn with less art than it wanted
/// is let go when more of the card's art arrives, so it is drawn again with it; one drawn with all it wanted is kept
/// however much sharper the art that arrives later, which is for drawing the card larger.
/// </para>
/// </summary>
internal static class FaceCache
{
    /// <summary>
    /// How much memory the faces of the moment may take, as their recordings count it; durable faces are not counted,
    /// since they are never let go. The art and frames a recording draws are shared, and not counted in it.
    /// </summary>
    private const long Budget = 32L * 1024 * 1024;

    /// <summary>
    /// How much of the budget drawing ahead and warming may fill. They never make room by letting a face go, so the rest
    /// stays for the faces in view.
    /// </summary>
    private const long AheadBudget = Budget * 85 / 100;

    private static long _used;
    private static long _durableUsed;

    /// <summary>How much the durable faces hold, and the faces of the moment, in bytes.</summary>
    public static long DurableBytes => _durableUsed;

    public static long MomentBytes => _used;

    private static readonly Dictionary<string, (SKPicture Face, string Slug, string Look, bool WantsArt, bool Stale, bool Durable, LinkedListNode<string> Node)> Kept = new(StringComparer.Ordinal);
    private static readonly LinkedList<string> Recent = new();

    /// <summary>The key of the face last drawn for each look at whatever size, for standing in while the size changes.</summary>
    private static readonly Dictionary<string, string> Latest = new(StringComparer.Ordinal);

    static FaceCache()
    {
        CardArt.Changed += Forget;
        PlayerSettings.ArtQualityChanged += Clear;
        CardArt.Relocated += Clear;
    }

    /// <summary>The face drawn under this key, if it is still kept and still current; counts as a use.</summary>
    public static SKPicture? Get(string key)
    {
        if (!Kept.TryGetValue(key, out var held) || held.Stale)
        {
            return null;
        }

        Recent.Remove(held.Node);
        Recent.AddLast(held.Node);
        return held.Face;
    }

    /// <summary>The face drawn under this key, if it is still kept and still current, without counting as a use.</summary>
    public static SKPicture? Peek(string? key) => key is not null && Kept.TryGetValue(key, out var held) && !held.Stale ? held.Face : null;

    /// <summary>
    /// The face drawn under this key to show until it is drawn again, current or not: a face let go for sharper art is
    /// still the card, and is shown rather than a blank until its replacement is made.
    /// </summary>
    public static SKPicture? StandIn(string? key) => key is not null && Kept.TryGetValue(key, out var held) ? held.Face : null;

    /// <summary>
    /// The face most recently drawn for a look at any size, if one is still kept: drawn scaled while the face at the size
    /// wanted is yet to be made, so a card resized keeps its face rather than blanking. Not counted as a use.
    /// </summary>
    public static SKPicture? AnySize(string look) => Latest.TryGetValue(look, out string? key) ? StandIn(key) : null;

    /// <summary>Keeps a face drawn under a key; <paramref name="look"/> is the key less the size it was drawn at.</summary>
    /// <param name="wantsArt">Whether it was drawn with less art than it wanted, and should be let go when more comes.</param>
    /// <param name="durable">Whether it is kept for good (<see cref="Faces.IsDurable"/>) rather than while there is room.</param>
    public static void Put(string key, string look, string slug, SKPicture face, bool wantsArt, bool durable)
    {
        if (Kept.TryGetValue(key, out var old))
        {
            Drop(key, old.Look, old.Node, old.Face, old.Durable);
        }

        Kept[key] = (face, slug, look, wantsArt, false, durable, Recent.AddLast(key));
        Latest[look] = key;
        if (!durable)
        {
            _used += Bytes(face);
        }
        else
        {
            _durableUsed += Bytes(face);
        }

        // The least recently used face of the moment goes first; a durable one is passed over, and so is the one just kept.
        LinkedListNode<string>? next = Recent.First;
        while (_used > Budget && next is not null)
        {
            LinkedListNode<string> at = next;
            next = at.Next;
            var held = Kept[at.Value];
            if (!held.Durable && at.Value != key)
            {
                Drop(at.Value, held.Look, at, held.Face, durable: false);
            }
        }
    }

    /// <summary>
    /// Whether another face would still leave the faces in view their share of the budget. A durable face is never let
    /// go, and takes nothing from it.
    /// </summary>
    public static bool HasRoomFor(bool durable) => durable || _used <= AheadBudget;

    private static long Bytes(SKPicture picture) => picture.ApproximateBytesUsed;

    private static void Drop(string key, string look, LinkedListNode<string> node, SKPicture face, bool durable)
    {
        if (!durable)
        {
            _used -= Bytes(face);
        }
        else
        {
            _durableUsed -= Bytes(face);
        }

        Kept.Remove(key);
        Recent.Remove(node);
        if (Latest.TryGetValue(look, out string? latest) && latest == key)
        {
            Latest.Remove(look);
        }

        face.Dispose();
    }

    /// <summary>
    /// Marks a card's faces that wanted more art than they had as out of date, for more of it having arrived: each is
    /// drawn again the next time it is wanted, and shown meanwhile as a stand-in.
    /// </summary>
    private static void Forget(string slug)
    {
        foreach (string key in Kept.Where(pair => CardArt.Covers(pair.Value.Slug, slug) && pair.Value.WantsArt).Select(pair => pair.Key).ToList())
        {
            Kept[key] = Kept[key] with { Stale = true };
        }
    }

    /// <summary>
    /// Lets every face go, for the art quality having changed - each was drawn with art of another size - or where art is
    /// having changed, when a face drawn as having none is not marked as wanting it.
    /// </summary>
    private static void Clear()
    {
        foreach ((string key, var held) in Kept.ToList())
        {
            Drop(key, held.Look, held.Node, held.Face, held.Durable);
        }
    }
}
