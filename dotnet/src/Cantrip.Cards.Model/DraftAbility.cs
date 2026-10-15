namespace Cantrip.Cards.Model;

/// <summary>
/// A draft ability (CR 905.2): what a card does while a booster pack is being drafted, carried out by the draft. Each is
/// one thing a draft knows how to do, as a keyword is one thing a game does; a card names the ones it prints.
/// </summary>
public enum DraftAbility
{
    /// <summary>
    /// "As you draft a card, you may draft an additional card from that booster pack. If you do, put this card into that
    /// booster pack." Cogwork Librarian. Usable from its drafter's pool on any later pick, once, as it leaves the pool.
    /// </summary>
    AdditionalPickPutThisIntoPack,
}
