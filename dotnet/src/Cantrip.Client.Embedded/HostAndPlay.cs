using Cantrip.Core;
using Cantrip.Protocol;
using Cantrip.Server;
using Cantrip.Server.Rules;

namespace Cantrip.Client.Embedded;

/// <summary>
/// Starts a game inside this process and hands back a transport per seat.
/// <para>
/// The only thing that distinguishes this from joining a remote server is which <see cref="IGameTransport"/> the
/// client ends up holding. Everything above that interface — the view, the prompts, the answers, the rendering — is
/// identical, so single-player is not a second implementation of the game to keep in step with the networked one.
/// </para>
/// </summary>
public static class HostAndPlay
{
    /// <summary>
    /// Starts a game from decks named one per seat on the rules engine <paramref name="rules"/> deals: the decks' card
    /// names as written, dealt round again where there are more seats than decks, as a server's table deals them
    /// (<c>GameServer.BuildTable</c>).
    /// <para>
    /// For playing many games in one process: every seat's deck is stated, so a matchup is the matchup asked for, and
    /// the loopback wiring is the same one a single-player client gets — the seats answer real protocol prompts from
    /// the real session, which is what makes a bulk run a test of the server and not of a shortcut around it.
    /// </para>
    /// <para>
    /// The cards are the native engine's own: nothing under <c>content/</c> is loaded, and a card the engine has not
    /// ported is the host's refusal to deal (<see cref="RulesRejectedException"/>).
    /// </para>
    /// </summary>
    public static HostedGame StartTable(
        IRulesHostFactory rules,
        IReadOnlyList<DeckList> decks,
        ulong seed,
        IReadOnlyList<string> playerNames)
    {
        ArgumentNullException.ThrowIfNull(rules);
        ArgumentNullException.ThrowIfNull(decks);
        ArgumentNullException.ThrowIfNull(playerNames);

        if (decks.Count == 0)
        {
            throw new ArgumentException("A table needs at least one deck.", nameof(decks));
        }

        static List<string> Named(IReadOnlyList<DeckListEntry> entries) =>
            [.. entries.SelectMany(entry => Enumerable.Repeat(entry.Card, entry.Copies))];

        List<RulesSeat> seats = [.. playerNames.Select((name, index) =>
            new RulesSeat(name, Named(decks[index % decks.Count].Entries), Named(decks[index % decks.Count].Sideboard)))];

        IRulesHost game = rules.Create(seats, seed, new RulesGameOptions());

        return Host(new GameSession(game), Listed(decks));
    }

    /// <summary>
    /// Each seat's decklist as names and counts, seat one first.
    /// <para>
    /// Kept because a seat answered by something that reasons about the game wants it: what is left in the library
    /// is most of what a decision turns on, and it is the one thing about a game that no projection of the position
    /// can say. Nothing else reads it, and a seat that ignores it is none the worse.
    /// </para>
    /// </summary>
    private static Dictionary<int, string> Listed(IReadOnlyList<DeckList> decks)
    {
        Dictionary<int, string> listed = [];

        for (int seat = 1; seat <= decks.Count; seat++)
        {
            listed[seat] = string.Join(
                '\n',
                decks[seat - 1].Entries.Select(entry => $"{entry.Copies} {entry.Card}"));
        }

        return listed;
    }

    private static HostedGame Host(GameSession session, Dictionary<int, string> decks)
    {
        Dictionary<int, IGameTransport> transports = [];
        foreach (PlayerId seat in session.Seats)
        {
            transports[seat.Value] = new LoopbackTransport(session, seat);
        }

        return new HostedGame(session, transports, decks);
    }
}

/// <summary>A game running in this process, with a transport per seat.</summary>
public sealed class HostedGame : IDisposable
{
    private readonly Dictionary<int, IGameTransport> _transports;

    internal HostedGame(
        GameSession session,
        Dictionary<int, IGameTransport> transports,
        IReadOnlyDictionary<int, string> decks)
    {
        _session = session;
        _transports = transports;
        Decks = decks;
    }

    /// <summary>What each seat was dealt, as a decklist of names and counts.</summary>
    public IReadOnlyDictionary<int, string> Decks { get; }

    private readonly GameSession _session;

    /// <summary>Start the hosted authority without exposing its implementation to presentation.</summary>
    public void Start() => _session.Start();

    public IGameTransport TransportFor(int seat) => _transports[seat];

    public IReadOnlyCollection<int> Seats => _transports.Keys;

    public void Dispose()
    {
        foreach (IGameTransport transport in _transports.Values)
        {
            transport.Dispose();
        }

        _session.Dispose();
    }
}
