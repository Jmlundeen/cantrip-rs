namespace Cantrip.Server.Rules;

/// <summary>One seat of a game to be dealt: who sits there and the cards they bring, by name.</summary>
/// <param name="Deck">The deck, one entry per copy, top first when dealt unshuffled.</param>
/// <param name="Sideboard">
/// The cards the seat owns outside the game (CR 100.4, 400.11a): what "a card you own from outside the game" chooses among
/// (North Wind Avatar). The host refuses a card it does not know here as in the deck.
/// </param>
public sealed record RulesSeat(string Name, IReadOnlyList<string> Deck, IReadOnlyList<string> Sideboard)
{
    public RulesSeat(string name, IReadOnlyList<string> deck) : this(name, deck, [])
    {
    }
}

/// <summary>Starting conditions beyond the seed (CR 103).</summary>
public sealed class RulesGameOptions
{
    /// <summary>
    /// The seat to take the first turn, from one; null plays out CR 103.1, where a seat chosen at random decides.
    /// </summary>
    public int? FirstPlayer { get; init; }

    /// <summary>The seat that chooses who goes first instead of one picked at random (CR 103.1); null for random.</summary>
    public int? Chooser { get; init; }

    /// <summary>Whether the decks are shuffled. False deals them in order, which is what a scenario wants.</summary>
    public bool Shuffle { get; init; } = true;

    /// <summary>
    /// In a Limited game, the basic lands the event supplies, of which every sideboard holds any number (Magic Tournament
    /// Rules; North Wind Avatar's ruling): one brought into the game from outside it leaves another there. Empty otherwise.
    /// </summary>
    public IReadOnlyList<string> BasicLandsOutside { get; init; } = [];

    /// <summary>A <c>.cantrip</c> scenario to play instead of dealing the seats' decks; null for a dealt game.</summary>
    public ScenarioSpec? Scenario { get; init; }
}

/// <summary>Deals a game on a rules engine, from a description of its seats.</summary>
public interface IRulesHostFactory
{
    /// <summary>
    /// Deals a new game: the seats in turn order, dealt from <paramref name="seed"/>, which with the decks is what the
    /// game is resumed from.
    /// </summary>
    /// <exception cref="InvalidOperationException">A deck names a card the rules do not know.</exception>
    IRulesHost Create(IReadOnlyList<RulesSeat> seats, ulong seed, RulesGameOptions options);
}

/// <summary>
/// The rules engine's factory: the native engine's sidecar, <see cref="NativeHostProcess.Shared"/>, or a transport given
/// for a test.
/// </summary>
public static class RulesHostFactory
{
    /// <summary>The factory every table deals its games through unless it is given another.</summary>
    public static IRulesHostFactory Native { get; } = new NativeRulesHostFactory();
}

/// <summary>
/// Deals on the native engine's sidecar: <see cref="NativeHostProcess.Shared"/>, or a transport given for a test. The
/// cards are the engine's own, named as the decks name them; a card it has not ported is the host's refusal to deal
/// (<see cref="RulesRejectedException"/>).
/// </summary>
public sealed class NativeRulesHostFactory : IRulesHostFactory
{
    private readonly IHostTransport? _transport;

    public NativeRulesHostFactory(IHostTransport? transport = null)
    {
        _transport = transport;
    }

    public IRulesHost Create(IReadOnlyList<RulesSeat> seats, ulong seed, RulesGameOptions options) =>
        NativeRulesHost.NewGame(_transport ?? NativeHostProcess.Shared, seats, seed, options);
}
