namespace Cantrip.Core;

/// <summary>
/// Identity of a game object (card, token, spell copy, ability on the stack, emblem...).
/// Numbered by the rules engine; never a GUID, never a hash.
/// </summary>
public readonly record struct ObjectId(int Value) : IComparable<ObjectId>
{
    public static readonly ObjectId None = default;
    public bool IsNone => Value == 0;
    public int CompareTo(ObjectId other) => Value.CompareTo(other.Value);
    public override string ToString() => Value == 0 ? "obj:none" : $"obj:{Value}";
}

public readonly record struct PlayerId(int Value) : IComparable<PlayerId>
{
    public static readonly PlayerId None = default;
    public bool IsNone => Value == 0;
    public int CompareTo(PlayerId other) => Value.CompareTo(other.Value);
    public override string ToString() => Value == 0 ? "player:none" : $"player:{Value}";
}

public readonly record struct TeamId(int Value) : IComparable<TeamId>
{
    public int CompareTo(TeamId other) => Value.CompareTo(other.Value);
    public override string ToString() => $"team:{Value}";
}

public readonly record struct EffectInstanceId(int Value) : IComparable<EffectInstanceId>
{
    public static readonly EffectInstanceId None = default;
    public int CompareTo(EffectInstanceId other) => Value.CompareTo(other.Value);
    public override string ToString() => $"effect:{Value}";
}

/// <summary>
/// CR 613.7 timestamp. Monotonic, and <em>reassigned</em> on the events listed in 613.7
/// (entering the battlefield, an Aura or Equipment becoming attached or reattached, a
/// face-down permanent being turned face up, ...), which is why this is a value allocated
/// from state rather than a creation-order index.
/// </summary>
public readonly record struct Timestamp(long Value) : IComparable<Timestamp>
{
    public static readonly Timestamp None = default;
    public int CompareTo(Timestamp other) => Value.CompareTo(other.Value);
    public override string ToString() => $"ts:{Value}";
}

/// <summary>
/// Bumped every time an object changes zones or ceases to exist. Pairs with
/// <see cref="ObjectId"/> in an <see cref="ObjectRef"/> so that a stale reference reads
/// last known information (CR 608.2h) instead of whatever now occupies that id, and so that
/// CR 400.7 ("a reference to a card that changed zones is a reference to a different object")
/// falls out of the model rather than being special-cased.
/// </summary>
public readonly record struct ZoneChangeEpoch(int Value) : IComparable<ZoneChangeEpoch>
{
    public static readonly ZoneChangeEpoch None = default;
    public int CompareTo(ZoneChangeEpoch other) => Value.CompareTo(other.Value);
    public override string ToString() => $"epoch:{Value}";
}

/// <summary>
/// The only way to refer to a game object. Resolve through the engine's object reader, which
/// returns live derived characteristics when <see cref="Epoch"/> still matches and the LKI
/// snapshot when it does not.
/// </summary>
public readonly record struct ObjectRef(ObjectId Id, ZoneChangeEpoch Epoch) : IComparable<ObjectRef>
{
    public static readonly ObjectRef None = default;
    public bool IsNone => Id.IsNone;

    public int CompareTo(ObjectRef other)
    {
        int c = Id.CompareTo(other.Id);
        return c != 0 ? c : Epoch.CompareTo(other.Epoch);
    }

    public override string ToString() => $"{Id}@{Epoch.Value}";
}

/// <summary>Identifies one outstanding request for player input.</summary>
public readonly record struct RequestId(int Value) : IComparable<RequestId>
{
    public static readonly RequestId None = default;
    public int CompareTo(RequestId other) => Value.CompareTo(other.Value);
    public override string ToString() => $"req:{Value}";
}

/// <summary>
/// Position in the decision log. Monotonic across the whole game. A reply carrying a stale
/// sequence (a reconnecting client resending) is rejected rather than appended.
/// </summary>
public readonly record struct DecisionSeq(int Value) : IComparable<DecisionSeq>
{
    public static readonly DecisionSeq None = default;
    public DecisionSeq Next() => new(Value + 1);
    public int CompareTo(DecisionSeq other) => Value.CompareTo(other.Value);
    public override string ToString() => $"seq:{Value}";
}
