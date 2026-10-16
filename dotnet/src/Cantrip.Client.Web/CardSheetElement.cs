using System.Runtime.InteropServices.JavaScript;
using Cantrip.Protocol;
using SkiaSharp;
using Uno.WinUI.Graphics2DSK;
using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// A sheet of sample card faces drawn by <see cref="CardFace"/> exactly as the table draws them, at a size big enough to
/// read, in place of the table (<c>?cardsheet</c>). It is for pictures of how the template lays a face out - where the
/// name sits against the art, how legible the type line is on each color - taken by the card render tests with
/// Playwright. It says <c>cantripCardSheet</c> is "ready" on the page once every face has been drawn.
/// </summary>
public sealed class CardSheetElement : SKCanvasElement
{
    public const float CardWidth = 420f;
    public const float CardHeight = 588f;
    public const float Gap = 16f;
    public const int Columns = 4;

    private static readonly ObjectView[] Cards =
    [
        Spell("Serra Pilgrim", "{1}{W}", [ColorView.White], [CardTypeView.Creature], ["Human", "Cleric"], 2, 2),
        Spell("Aether Scholar", "{2}{U}", [ColorView.Blue], [CardTypeView.Creature], ["Merfolk", "Wizard"], 2, 3),
        Spell("Graveyard Keeper", "{B}{B}", [ColorView.Black], [CardTypeView.Creature], ["Zombie"], 3, 1),
        Spell("Ember Charge", "{R}", [ColorView.Red], [CardTypeView.Instant], [], null, null),
        Spell("Thornback Wurm", "{3}{G}{G}", [ColorView.Green], [CardTypeView.Creature], ["Wurm"], 5, 5),
        Spell("Bant Charm of Many Names", "{G}{W}{U}", [ColorView.Green, ColorView.White, ColorView.Blue], [CardTypeView.Creature], ["Elf", "Knight"], 3, 3),
        Spell("Wide Open Plains", string.Empty, [], [CardTypeView.Land], [], null, null),
        Spell("The Ever-Replenishing Reliquary of Ages", "{4}{W}{W}{U}{U}{B}", [ColorView.White, ColorView.Blue, ColorView.Black], [CardTypeView.Sorcery], [], null, null),
        Spell("Tin Automaton", "{3}", [], [CardTypeView.Artifact, CardTypeView.Creature], ["Construct"], 2, 2),
        Spell("Glimmering Veil", "{1}{W}", [ColorView.White], [CardTypeView.Enchantment], [], null, null),
        Spell("Sorcerous Insight", "{U}{U}", [ColorView.Blue], [CardTypeView.Sorcery], [], null, null),
        Spell("Pyre Drake", "{2}{R}", [ColorView.Red], [CardTypeView.Creature], ["Drake"], 2, 2),
    ];

    public CardSheetElement()
    {
        int rows = (Cards.Length + Columns - 1) / Columns;
        Width = Gap + (Columns * (CardWidth + Gap));
        Height = Gap + (rows * (CardHeight + Gap));
    }

    protected override void RenderOverride(SKCanvas canvas, Size area)
    {
        canvas.Clear(new SKColor(34, 34, 38));

        for (int i = 0; i < Cards.Length; i++)
        {
            canvas.Save();
            canvas.Translate(Gap + ((i % Columns) * (CardWidth + Gap)), Gap + ((i / Columns) * (CardHeight + Gap)));
            CardFace.Draw(canvas, Cards[i], CardWidth, CardHeight, dimmed: false);
            canvas.Restore();
        }

        JSHost.GlobalThis.SetProperty("cantripCardSheet", "ready");
    }

    private static ObjectView Spell(string name, string cost, ColorView[] colors, CardTypeView[] types, string[] subtypes, int? power, int? toughness) => new()
    {
        Name = name,
        ManaCost = cost,
        Colors = [.. colors],
        CardTypes = [.. types],
        Subtypes = [.. subtypes],
        Power = power,
        Toughness = toughness,
        OracleText = "Sample rules text for judging the type line and name.",
        Zone = ZoneKindView.Hand,
        Kind = ObjectKindView.Card,
    };
}
