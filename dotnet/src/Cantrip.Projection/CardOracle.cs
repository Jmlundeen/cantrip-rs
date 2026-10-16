using System.Text.RegularExpressions;
using Cantrip.Cards.Model;
using Cantrip.Protocol;

namespace Cantrip.Projection;

/// <summary>
/// A card as it is printed, for a client to read: what a draft shows a drafter and a deck builder shows a player, for
/// every card of the content - not only those the rules engine has ported.
/// <para>
/// Characteristics only: name, cost, type line, text, power and toughness, loyalty or defense, and colors from the cost
/// and the color indicator (CR 202.2). The other face of a double-faced card, the halves of a split card and a prepare
/// spell go with it, as a game's own oracle sends them. Nothing the rules would add (an ability granted, a mana ability a
/// basic land has by its subtype) is here, because nothing here runs the rules.
/// </para>
/// </summary>
public sealed partial class CardOracle
{
    private readonly CardCatalog _cards;
    private readonly Dictionary<CardId, ObjectView?> _oracles = [];

    public CardOracle(CardCatalog cards)
    {
        ArgumentNullException.ThrowIfNull(cards);
        _cards = cards;
    }

    /// <summary>The card by its id, or null for a card the content does not have. Worked out once for each card.</summary>
    public ObjectView? Oracle(CardId id)
    {
        lock (_oracles)
        {
            if (_oracles.TryGetValue(id, out ObjectView? known))
            {
                return known;
            }

            if (!_cards.TryGet(id, out CardDefinition? card))
            {
                return _oracles[id] = null;
            }

            ObjectView view = Printed(card!);
            view.OtherFace = card!.BackFace is { } back ? Printed(back) : null;
            view.Halves.AddRange(card.Halves.Select(Printed));
            view.PrepareSpell = card.Prepare is { } prepare ? Printed(prepare) : null;
            view.InsetSpell = (card.Adventure ?? card.Omen) is { } inset ? Printed(inset) : null;
            return _oracles[id] = view;
        }
    }

    /// <summary>
    /// One printed face as a card to be read: its name, cost, type line, text and printed power and toughness, and its
    /// colors from its cost and its color indicator. Nothing else, because a face has no state.
    /// </summary>
    public static ObjectView Printed(CardDefinition face)
    {
        ArgumentNullException.ThrowIfNull(face);

        var view = new ObjectView
        {
            Handle = ViewHandle.None,
            Known = true,
            CardId = face.Id.Value,
            Name = face.Name,
            ManaCost = face.ManaCost.ToString(),
            ManaValue = face.ManaCost.ManaValue,
            OracleText = face.OracleText,
            Power = face.Power,
            Toughness = face.Toughness,
            StartingCounter = face.Loyalty ?? face.Defense,
        };

        foreach (Supertype supertype in face.TypeLine.Supertypes)
        {
            view.Supertypes.Add(Vocabulary.Supertype(supertype));
        }

        foreach (CardType type in face.TypeLine.CardTypes)
        {
            view.CardTypes.Add(Vocabulary.CardType(type));
        }

        view.Subtypes.AddRange(face.TypeLine.Subtypes);

        // A basic land makes mana by its subtype (CR 305.6) and prints nothing; any other card says so in its text.
        view.ProducesMana = face.IsBasicLand || (face.IsLand && AddsMana().IsMatch(face.OracleText));

        // CR 202.2: colors come from the mana cost, and from the color indicator a back face carries instead of one.
        foreach (ManaColor color in face.ManaCost.Colors.Concat(face.ColorIndicator).Distinct())
        {
            view.Colors.Add(Vocabulary.Color(color));
        }

        return view;
    }

    [GeneratedRegex(@"\bAdd \{", RegexOptions.CultureInvariant)]
    private static partial Regex AddsMana();
}
