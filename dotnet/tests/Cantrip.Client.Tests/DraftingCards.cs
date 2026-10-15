using Cantrip.Client.Drafting;
using Cantrip.Protocol;

namespace Cantrip.Client.Tests;

/// <summary>
/// Cards for the drafting tests, as a pack shows them: made up, and named for what they are, because the drafter knows
/// no card by name and a test that leaned on a real card's reputation would be testing nothing.
/// </summary>
internal static class DraftingCards
{
    private static int _next = 1;

    public static ObjectView Spell(string name, string cost, CardTypeView type = CardTypeView.Sorcery, string text = "") => new()
    {
        Handle = new ViewHandle(_next++),
        CardId = name,
        Name = name,
        ManaCost = cost,
        ManaValue = ManaValue(cost),
        CardTypes = [type],
        Colors = [.. "WUBRG".Select((letter, at) => (letter, at)).Where(pair => cost.Contains(pair.letter, StringComparison.Ordinal)).Select(pair => (ColorView)pair.at)],
        OracleText = text,
    };

    public static ObjectView Creature(string name, string cost, int power, int toughness, string text = "")
    {
        ObjectView card = Spell(name, cost, CardTypeView.Creature, text);
        card.Power = power;
        card.Toughness = toughness;
        return card;
    }

    public static ObjectView Land(string name, string text, params string[] subtypes) => new()
    {
        Handle = new ViewHandle(_next++),
        CardId = name,
        Name = name,
        CardTypes = [CardTypeView.Land],
        Subtypes = [.. subtypes],
        OracleText = text,
        ProducesMana = text.Contains("Add", StringComparison.Ordinal) || subtypes.Length > 0,
    };

    /// <summary>A card read, and worth what it is said to be.</summary>
    public static DraftCard Worth(this ObjectView view, double worth) => DraftCard.Read(view).WithWorth(worth);

    /// <summary>A plain creature of a colour at a cost, for filling a pool: "R3" is a red three-drop.</summary>
    public static ObjectView Filler(char color, int manaValue, int number) =>
        Creature($"{color}{manaValue} #{number}", (manaValue > 1 ? $"{{{manaValue - 1}}}" : string.Empty) + $"{{{color}}}", manaValue, manaValue);

    private static int ManaValue(string cost)
    {
        int value = 0;
        foreach (string symbol in cost.Split('{', '}').Where(part => part.Length > 0 && !part.Contains(' ', StringComparison.Ordinal)))
        {
            value += int.TryParse(symbol, out int generic) ? generic : symbol == "X" ? 0 : 1;
        }

        return value;
    }
}
