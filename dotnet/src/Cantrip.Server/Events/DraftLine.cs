using Cantrip.Protocol;

namespace Cantrip.Server.Events;

/// <summary>
/// A line of a draft's record of picks, as a seat reads it: the words, each card in them by name in square brackets
/// the way the game log writes them (<see cref="DraftMessage.Recent"/>), and those cards, so the client can show the
/// card a name stands for.
/// </summary>
internal sealed class DraftLine
{
    public string Text { get; private set; } = string.Empty;

    public List<LogReferenceView> Cards { get; } = [];

    /// <summary>Writes a line, naming cards in it through the function it is handed.</summary>
    public static DraftLine Write(Func<Func<ObjectView, string>, string> words)
    {
        var line = new DraftLine();
        line.Text = words(line.Name);
        return line;
    }

    private string Name(ObjectView card)
    {
        Cards.Add(new LogReferenceView { Name = card.Name, Card = card });
        return $"[{card.Name}]";
    }
}
