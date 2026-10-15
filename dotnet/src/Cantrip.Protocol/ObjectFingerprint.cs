using MessagePack;

namespace Cantrip.Protocol;

/// <summary>
/// The digest a server stamps on each object it sends (<see cref="ObjectView.Fingerprint"/>): the object's own encoding
/// hashed, so every field counts, a field added later included, without a list of them to keep in step.
/// <para>
/// Worked out on the server, where encoding is cheap, so the client does not have to: a browser client redrawing a board
/// of two hundred permanents on every view spent much of each redraw working out afresh what every one of them looked
/// like, when nearly all of them were exactly what they had been.
/// </para>
/// </summary>
public static class ObjectFingerprint
{
    /// <summary>Stamps every object in a view with its fingerprint.</summary>
    public static void Stamp(PlayerView view)
    {
        ArgumentNullException.ThrowIfNull(view);
        foreach (ObjectView item in view.Objects)
        {
            item.Fingerprint = Of(item);
        }
    }

    /// <summary>An object's fingerprint: never zero, which says none was worked out.</summary>
    public static long Of(ObjectView item)
    {
        ArgumentNullException.ThrowIfNull(item);

        long stamped = item.Fingerprint;
        item.Fingerprint = 0;
        byte[] encoded = MessagePackSerializer.Serialize(item);
        item.Fingerprint = stamped;

        // FNV-1a, 64 bits: not for security, only to tell an unchanged object from a changed one.
        ulong hash = 14695981039346656037UL;
        foreach (byte b in encoded)
        {
            hash = (hash ^ b) * 1099511628211UL;
        }

        return (long)(hash | 1UL);
    }
}
