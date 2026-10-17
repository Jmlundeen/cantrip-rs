using System.Text.RegularExpressions;
using Cantrip.Protocol;

namespace Cantrip.Client.Tactics;

/// <summary>
/// The words of a text-changing spell (CR 612: Mind Bend, Sleight of Mind, Magical Hack, Artificial Evolution, Trait
/// Doctoring), asked as it resolves: the word to replace is one its target's text has, or the change does nothing; the word
/// replacing it helps this seat when the target is its own and hurts the target's controller otherwise. And Old Man of the
/// Sea's "you may choose not to untap this creature" (CR 502.3).
/// </summary>
public sealed partial class TacticalPlay
{
    private static readonly string[] LandTypes = ["Plains", "Island", "Swamp", "Mountain", "Forest"];

    private static readonly Dictionary<string, ColorView> LandColor = new(StringComparer.Ordinal)
    {
        ["Plains"] = ColorView.White,
        ["Island"] = ColorView.Blue,
        ["Swamp"] = ColorView.Black,
        ["Mountain"] = ColorView.Red,
        ["Forest"] = ColorView.Green,
    };

    /// <summary>The permanent or spell a resolving text-changing spell targets, if it can be seen.</summary>
    private static ObjectView? TextChangeTarget(GameClient client, ViewHandle spell) =>
        client.Object(spell)?.Targets.Select(target => client.Object(target.Object)).OfType<ObjectView>().FirstOrDefault();

    /// <summary>
    /// Whether `word` (a color word, a land type or a creature type) is a word of the target's text or type line now:
    /// alone, plural, after "non" or before "walk" (CR 612.2).
    /// </summary>
    private static bool HasWord(ObjectView target, string word)
    {
        if (target.Subtypes.Exists(subtype => string.Equals(subtype, word, StringComparison.OrdinalIgnoreCase)))
        {
            return true;
        }

        string pattern = $@"\b(non)?{Regex.Escape(word)}(s|es|walk)?\b";
        return Regex.IsMatch(Regex.Replace(target.OracleText, @"\([^)]*\)", string.Empty), pattern, RegexOptions.IgnoreCase);
    }

    /// <summary>The colors among the permanents and spells a side has out, this seat's hand added for its own side.</summary>
    private static List<ColorView> SideColors(GameClient client, PlayerView view, bool mine)
    {
        List<ColorView> colors = [.. view.Objects
            .Where(card => card.Zone is ZoneKindView.Battlefield or ZoneKindView.Stack && Mine(client, card) == mine && !IsLand(card))
            .SelectMany(card => card.Colors)];
        if (mine)
        {
            colors.AddRange(client.MyHand().Where(card => !IsLand(card)).SelectMany(card => card.Colors));
        }

        return colors;
    }

    /// <summary>
    /// The new word, ranked: for this seat's own target, the one its side has most of (its creatures' type, the color its
    /// cards are or its lands should make); for an opponent's, the one that side has least of, so the change takes away
    /// what the text did for them.
    /// </summary>
    private static int RankedNewWord(IReadOnlyList<int> eligible, Func<int, int> count, bool mine) =>
        mine ? eligible.OrderByDescending(count).First() : eligible.OrderBy(count).First();

    /// <summary>"Replace a color word" or "Replace a basic land type" (Mind Bend, Trait Doctoring): the kind its target has.</summary>
    private static int? TextChangeKind(GameClient client, ChooseModesPromptMessage prompt)
    {
        int color = prompt.Modes.FindIndex(mode => mode == "Replace a color word");
        int land = prompt.Modes.FindIndex(mode => mode == "Replace a basic land type");
        if (color < 0 || land < 0 || TextChangeTarget(client, prompt.Source) is not { } target)
        {
            return null;
        }

        bool colors = Enum.GetValues<ColorView>().Any(c => HasWord(target, c.ToString()));
        bool lands = LandTypes.Any(type => HasWord(target, type));
        return lands && !colors ? land : color;
    }

    /// <summary>A creature type or basic land type of a text change (CR 612.1): the one to replace, or the one replacing it.</summary>
    private static int? TextChangeSubtype(GameClient client, PlayerView view, ChooseModesPromptMessage prompt)
    {
        bool replaced = prompt.Heading.Contains(": choose the ", StringComparison.Ordinal) && prompt.Heading.EndsWith(" to replace", StringComparison.Ordinal);
        bool replacing = prompt.Heading.Contains(": choose the ", StringComparison.Ordinal) && prompt.Heading.EndsWith(" that replaces it", StringComparison.Ordinal);
        if (!(replaced || replacing) || prompt.Eligible.Count == 0)
        {
            return null;
        }

        ObjectView? target = TextChangeTarget(client, prompt.Source);
        if (replaced)
        {
            return target is null ? prompt.Eligible[0] : prompt.Eligible.FirstOrDefault(mode => HasWord(target, prompt.Modes[mode]), prompt.Eligible[0]);
        }

        bool mine = target is null || Mine(client, target);
        bool lands = prompt.Modes.TrueForAll(mode => LandColor.ContainsKey(mode));
        if (lands)
        {
            List<ColorView> colors = SideColors(client, view, mine);
            return RankedNewWord(prompt.Eligible, mode => colors.Count(c => c == LandColor[prompt.Modes[mode]]), mine);
        }

        List<ObjectView> side = [.. view.Objects.Where(card => card.Zone == ZoneKindView.Battlefield && Mine(client, card) == mine && IsCreature(card))];
        if (mine)
        {
            side.AddRange(client.MyHand().Where(IsCreature));
        }

        return RankedNewWord(prompt.Eligible, mode => side.Count(card => card.Subtypes.Contains(prompt.Modes[mode])), mine);
    }

    /// <summary>A color word of a text change (CR 612.1), or null for a choice of colors of another kind.</summary>
    private static ColorView? TextChangeColor(GameClient client, PlayerView view, ChooseColorsPromptMessage prompt)
    {
        if (prompt.Verb is not ("ReplaceWord" or "ReplacingWord"))
        {
            return null;
        }

        ObjectView? target = TextChangeTarget(client, prompt.Source);
        if (prompt.Verb == "ReplaceWord")
        {
            return target is null ? prompt.Choices[0] : prompt.Choices.FirstOrDefault(c => HasWord(target, c.ToString()), prompt.Choices[0]);
        }

        bool mine = target is null || Mine(client, target);
        List<ColorView> colors = SideColors(client, view, mine);
        List<int> indices = [.. Enumerable.Range(0, prompt.Choices.Count)];
        return prompt.Choices[RankedNewWord(indices, i => colors.Count(c => c == prompt.Choices[i]), mine)];
    }

    /// <summary>
    /// "You may choose not to untap this creature during your untap step" (CR 502.3): left tapped while it holds something
    /// it took "for as long as it remains tapped" (Old Man of the Sea), which the held permanent says; untapped otherwise.
    /// </summary>
    private static bool UntapsInUntapStep(GameClient client, MayPromptMessage prompt)
    {
        string? name = client.Object(prompt.Source)?.Name;
        return name is null || !client.BattlefieldOf(client.Seat).Any(permanent =>
            permanent.GrantedAbilities.Exists(line => line.Text.Contains($"for as long as {name} remains tapped", StringComparison.Ordinal)));
    }
}
