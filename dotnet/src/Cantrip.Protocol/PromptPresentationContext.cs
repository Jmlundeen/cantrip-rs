using MessagePack;

namespace Cantrip.Protocol;

/// <summary>Semantic action identifiers for localization, distinct from where a card happens to be displayed.</summary>
public enum PromptActionView
{
    Choose, Discard, PutOnLibrary, Keep, Sacrifice, Exile, ReturnToHand, Untap, PutOntoBattlefield,
    PutIntoHand, Cast, Tap, Distribute, Pay, OrderReplacement, Destroy, Vote, PutIntoGraveyard, Play, Splice,
    PutOnLibraryBottom, Attach, Encode, Behold, Blight, TapToReduce, PutCounters, Craft, Reveal, Plot,
}

public enum PromptCauseView { Effect, Cost, Rule }

[MessagePackObject]
public sealed class PromptOriginView
{
    [Key(0)] public ZoneKindView Zone { get; set; }
    [Key(1)] public int Owner { get; set; }
}

/// <summary>Facts the presenter cannot safely infer from purpose or zone. Counts remain server-authoritative.</summary>
[MessagePackObject]
public sealed class PromptPresentationContext
{
    [Key(0)] public PromptActionView Action { get; set; }
    [Key(1)] public PromptCauseView Cause { get; set; }
    [Key(2)] public int Chooser { get; set; }
    [Key(3)] public int Owner { get; set; }
    [Key(4)] public ViewHandle Source { get; set; }
    [Key(5)] public List<PromptOriginView> Origins { get; set; } = [];
    [Key(6)] public int Minimum { get; set; }
    [Key(7)] public int Maximum { get; set; }
    [Key(8)] public bool Ordered { get; set; }
}
