using Cantrip.Cards.Model;
using Cantrip.Protocol;

namespace Cantrip.Projection;

/// <summary>
/// Translates a card's printed vocabulary into the wire's copy of it.
/// <para>
/// The duplication exists so Protocol need not reference the card model. Every mapping throws on an unrecognised value
/// rather than defaulting, so adding a card type to the model fails loudly instead of quietly projecting as something else.
/// </para>
/// </summary>
internal static class Vocabulary
{
    public static CardTypeView CardType(CardType type) => type switch
    {
        Cards.Model.CardType.Artifact => CardTypeView.Artifact,
        Cards.Model.CardType.Battle => CardTypeView.Battle,
        Cards.Model.CardType.Creature => CardTypeView.Creature,
        Cards.Model.CardType.Enchantment => CardTypeView.Enchantment,
        Cards.Model.CardType.Instant => CardTypeView.Instant,
        Cards.Model.CardType.Kindred => CardTypeView.Kindred,
        Cards.Model.CardType.Land => CardTypeView.Land,
        Cards.Model.CardType.Planeswalker => CardTypeView.Planeswalker,
        Cards.Model.CardType.Sorcery => CardTypeView.Sorcery,
        _ => throw Unknown(type),
    };

    public static SupertypeView Supertype(Supertype supertype) => supertype switch
    {
        Cards.Model.Supertype.Basic => SupertypeView.Basic,
        Cards.Model.Supertype.Legendary => SupertypeView.Legendary,
        Cards.Model.Supertype.Ongoing => SupertypeView.Ongoing,
        Cards.Model.Supertype.Snow => SupertypeView.Snow,
        Cards.Model.Supertype.World => SupertypeView.World,
        _ => throw Unknown(supertype),
    };

    public static ColorView Color(ManaColor color) => color switch
    {
        ManaColor.White => ColorView.White,
        ManaColor.Blue => ColorView.Blue,
        ManaColor.Black => ColorView.Black,
        ManaColor.Red => ColorView.Red,
        ManaColor.Green => ColorView.Green,
        _ => throw Unknown(color),
    };

    private static ArgumentOutOfRangeException Unknown<T>(T value)
        where T : struct, Enum =>
        new(nameof(value), value, $"{typeof(T).Name}.{value} has no wire equivalent. Add one to Cantrip.Protocol.");
}
