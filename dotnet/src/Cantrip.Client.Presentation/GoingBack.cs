using Cantrip.Protocol;

namespace Cantrip.Client.Presentation;

/// <summary>
/// The way back to the page a player came to a table from, once its game is over (<see cref="TableMessage.Return"/>):
/// the site's draft queues after a draft's game, its constructed play after a constructed one. The result is shown first;
/// the page goes back by itself after <see cref="Wait"/>, or at once from its button, unless the player has put the
/// result away to look at the board, when it waits for them to go. A table that names no page - one of the player's own,
/// or a booster pod's game, whose players the table sends on itself - has no way back, and nothing here does anything.
/// </summary>
public sealed class GoingBack(TimeSpan wait)
{
    /// <summary>How long the result is shown before the page goes back by itself.</summary>
    public static readonly TimeSpan DefaultWait = TimeSpan.FromSeconds(30);

    private DateTimeOffset? _since;
    private bool _staying;
    private bool _gone;

    public GoingBack()
        : this(DefaultWait)
    {
    }

    public TimeSpan Wait { get; } = wait;

    /// <summary>The page to go back to, once the game is over and the table names one; null otherwise.</summary>
    public Uri? Address { get; private set; }

    /// <summary>What the page is called ("Draft Queues").</summary>
    public string Name { get; private set; } = string.Empty;

    /// <summary>What the button that goes back says.</summary>
    public string Button => Name.Length > 0 ? $"Back to {Name}" : "Back";

    /// <summary>
    /// The page the table names, as a page may be gone to: an absolute http(s) address, never a script or anything else
    /// a table could name.
    /// </summary>
    public static Uri? PageOf(TableMessage? table) =>
        table is { Return.Length: > 0 } && Uri.TryCreate(table.Return, UriKind.Absolute, out Uri? address) && address.Scheme is "http" or "https"
            ? address
            : null;

    /// <summary>
    /// Follows the table, every frame: whether its game is over and where it goes back to. Returns the page to go to once
    /// it is time - the wait over and the player not staying - and only once.
    /// </summary>
    public Uri? Follow(TableMessage? table, bool over, DateTimeOffset now)
    {
        Address = over ? PageOf(table) : null;
        if (Address is null)
        {
            Name = string.Empty;
            _since = null;
            return null;
        }

        Name = table!.ReturnName;
        _since ??= now;
        return !_staying && now - _since >= Wait ? Go() : null;
    }

    /// <summary>The player put the result away to look at the board: the page waits for them to go back themselves.</summary>
    public void Stay() => _staying = true;

    /// <summary>Whether the page is counting down to going back by itself.</summary>
    public bool Counting => Address is not null && !_staying && !_gone;

    /// <summary>The page to go to now, from the button; null with none, or once gone.</summary>
    public Uri? Go()
    {
        if (Address is null || _gone)
        {
            return null;
        }

        _gone = true;
        return Address;
    }

    /// <summary>What the result says about going back while it counts down ("Back to Draft Queues in 30 s"); empty otherwise.</summary>
    public string Countdown(DateTimeOffset now)
    {
        if (!Counting || _since is not { } since)
        {
            return string.Empty;
        }

        int seconds = (int)Math.Ceiling(Math.Max(0, (Wait - (now - since)).TotalSeconds));
        return $"{Button} in {seconds} s";
    }
}
