using MessagePack;

namespace Cantrip.Protocol;

/// <summary>
/// How one viewer refers to one object. Allocated per viewer, and never the engine's internal object id.
/// <para>
/// The separation is not hygiene, it closes a leak. Internal ids are stable for an object's whole life
/// and are allocated in deck order at setup, so putting them on the wire would let a client correlate a
/// card it saw once with a card it sees later — including across a shuffle, which CR 701.22 says must
/// randomise the library beyond anyone's knowledge.
/// </para>
/// <para>
/// A handle is minted the first time a viewer can observe an object and <em>discarded the moment they
/// cannot</em>. A card that goes to a library and comes back gets a fresh handle, so there is no
/// correlation to be made even in principle, rather than one the projection is careful not to reveal.
/// </para>
/// </summary>
[MessagePackObject]
public readonly record struct ViewHandle([property: Key(0)] int Value) : IComparable<ViewHandle>
{
    public static readonly ViewHandle None = default;

    [IgnoreMember]
    public bool IsNone => Value == 0;

    public int CompareTo(ViewHandle other) => Value.CompareTo(other.Value);

    public override string ToString() => Value == 0 ? "h:none" : $"h:{Value}";
}
