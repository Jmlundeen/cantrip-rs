using System.Text.RegularExpressions;
using Cantrip.Protocol;

namespace Cantrip.Client.Tactics;

/// <summary>
/// Which mana source pays: the one that leaves open the mana for the plays this seat means to make next - the next
/// spell of the turn's plan, and the instant it means to hold up through the opponent's turn - and, of those, the one
/// that hurts least, wastes least, and is a plain land rather than a land or creature that does something else.
/// </summary>
public sealed partial class TacticalPlay
{
    /// <summary>One mana cost as the payment reads it: its coloured symbols, each with the colours that pay it, and its generic.</summary>
    /// <param name="Pips">Each symbol that wants a colour: the colours that pay it; empty for {C}, which only colourless pays.</param>
    internal sealed record ManaNeed(List<ColorView[]> Pips, int Generic);

    /// <summary>A source of mana the payment may tap, as its offered mana abilities say.</summary>
    private sealed record ManaSource(ViewHandle Handle, HashSet<ColorView> Colors, bool Colorless, int Amount, int Rank);

    /// <summary>
    /// The mana costs of the plays this seat means to make after the one being paid for: the rest of the turn's plan and
    /// the instant it holds up for the opponent's turn, in that order. Set when the plan is made, at priority.
    /// </summary>
    private List<ManaNeed> _keepOpen = [];

    private static readonly Regex Symbol = new(@"\{([^}]+)\}", RegexOptions.Compiled);

    private static ColorView? Letter(char letter) => letter switch
    {
        'W' => ColorView.White,
        'U' => ColorView.Blue,
        'B' => ColorView.Black,
        'R' => ColorView.Red,
        'G' => ColorView.Green,
        _ => null,
    };

    /// <summary>A printed mana cost read for paying: <c>{2}{U}{U}</c> is two blue symbols and two generic.</summary>
    internal static ManaNeed CostNeed(string cost)
    {
        List<ColorView[]> pips = [];
        int generic = 0;
        foreach (Match match in Symbol.Matches(cost ?? string.Empty))
        {
            string inside = match.Groups[1].Value.ToUpperInvariant();
            if (int.TryParse(inside, System.Globalization.NumberStyles.None, System.Globalization.CultureInfo.InvariantCulture, out int number))
            {
                generic += number;
            }
            else if (inside == "C")
            {
                pips.Add([]);
            }
            else if (inside.Length == 1 && Letter(inside[0]) is { } color)
            {
                pips.Add([color]);
            }
            else if (inside.Contains('/', StringComparison.Ordinal))
            {
                // Hybrid and Phyrexian symbols: any of their colours pays (life is not planned for).
                ColorView[] colors = [.. inside.Split('/').Where(part => part.Length == 1).Select(part => Letter(part[0])).OfType<ColorView>()];
                if (colors.Length > 0)
                {
                    pips.Add(colors);
                }
            }
        }

        return new ManaNeed(pips, generic);
    }

    /// <summary>
    /// Whether these sources can pay all of these costs together: every coloured symbol a source of its colour, one source's
    /// mana to a symbol each, and enough left over for the generic.
    /// </summary>
    private static bool Payable(IReadOnlyList<ManaSource> sources, IEnumerable<ManaNeed> needs)
    {
        List<ColorView[]> pips = [];
        int generic = 0;
        foreach (ManaNeed need in needs)
        {
            pips.AddRange(need.Pips);
            generic += need.Generic;
        }

        bool Pays(ManaSource source, ColorView[] pip) => pip.Length == 0 ? source.Colorless : pip.Any(source.Colors.Contains);

        int total = sources.Sum(source => source.Amount);
        if (total < pips.Count + generic)
        {
            return false;
        }

        // The symbols fewest sources can pay go first.
        List<ColorView[]> ordered = [.. pips.OrderBy(pip => sources.Count(source => Pays(source, pip)))];
        int[] left = [.. sources.Select(source => source.Amount)];
        int tries = 0;

        bool Assign(int at)
        {
            if (at == ordered.Count)
            {
                return true;
            }

            if (++tries > 20_000)
            {
                return false;
            }

            for (int s = 0; s < sources.Count; s++)
            {
                if (left[s] > 0 && Pays(sources[s], ordered[at]))
                {
                    left[s]--;
                    if (Assign(at + 1))
                    {
                        return true;
                    }

                    left[s]++;
                }
            }

            return false;
        }

        return Assign(0);
    }

    /// <summary>
    /// The mana ability to use next in paying this cost so that what is left can still pay the plays kept open, or null to
    /// leave the payment to the ordinary way: a cost with X or a choice still to announce, mana already in the pool, a cost
    /// paid by tapping or exiling things, or one this reading cannot account for.
    /// </summary>
    private TakeActionAnswer? Planned(GameClient client, PayCostsPromptMessage prompt)
    {
        if (Policy("payplan") <= 0 || prompt.HasX || prompt.AnnouncesXOrKicks
            || prompt.Options.Exists(option => option.Kind is ActionKindView.AssignMana or ActionKindView.TapToPay or ActionKindView.ExileToPay))
        {
            return null;
        }

        List<ActionOptionMessage> abilities = [.. prompt.Options.Where(option => option.Kind == ActionKindView.ActivateAbility
            && !option.PaysNothing && option.ProducesOfAnyColorEach == 0 && option.SacrificeCandidates.Count == 0)];
        if (abilities.Count == 0 || abilities.Count != prompt.Options.Count(option => option.Kind == ActionKindView.ActivateAbility))
        {
            return null;
        }

        List<ManaSource> sources = [.. abilities.GroupBy(option => option.Source).Select(group =>
        {
            HashSet<ColorView> colors = [.. group.SelectMany(option => option.Produces.Where(type => type != ManaTypeView.Colorless).Select(type => (ColorView)(int)type)
                .Concat(option.ProducesOfAnyOneColor > 0 ? option.ColorChoices : []))];
            bool colorless = group.Any(option => option.Produces.Contains(ManaTypeView.Colorless) || (option.ProducesOfAnyOneColor > 0 && option.ColorChoices.Count == 0));
            int amount = group.Max(option => option.Produces.Count + option.ProducesOfAnyOneColor);
            return new ManaSource(group.Key, colors, colorless, Math.Max(amount, 1), Rank(client.Object(group.Key)));
        })];

        ManaNeed owed = new([.. prompt.RemainingColored.Select(color => new[] { color }),
            .. prompt.RemainingHybrid.Select(symbol => symbol.Colors.ToArray()),
            .. Enumerable.Repeat(Array.Empty<ColorView>(), prompt.RemainingColorless)], prompt.RemainingGeneric);
        int owedTotal = owed.Pips.Count + owed.Generic;
        if (owedTotal == 0 || !Payable(sources, [owed]))
        {
            return null;
        }

        // What one use of this ability pays of what is owed, and what is then still owed.
        ManaNeed? After(ActionOptionMessage option, ColorView? named, out int paid)
        {
            List<ManaTypeView> made = [.. option.Produces];
            if (option.ProducesOfAnyOneColor > 0)
            {
                made.AddRange(Enumerable.Repeat(named is { } color ? (ManaTypeView)(int)color : ManaTypeView.Colorless, option.ProducesOfAnyOneColor));
            }

            List<ColorView[]> pips = [.. owed.Pips];
            int generic = owed.Generic;
            paid = 0;
            foreach (ManaTypeView mana in made)
            {
                int at = pips.FindIndex(pip => mana == ManaTypeView.Colorless ? pip.Length == 0 : pip.Contains((ColorView)(int)mana));
                if (at >= 0)
                {
                    pips.RemoveAt(at);
                    paid++;
                }
                else if (generic > 0)
                {
                    generic--;
                    paid++;
                }
            }

            return paid == 0 ? null : new ManaNeed(pips, generic);
        }

        // Every plan kept open if it can be; failing that, the held instant is let go, then each later play of the turn.
        for (int keep = _keepOpen.Count; keep >= 0; keep--)
        {
            List<ManaNeed> kept = [.. _keepOpen.Take(keep)];
            (TakeActionAnswer Answer, (int, int, int, int) Key)? best = null;

            foreach (ActionOptionMessage option in abilities)
            {
                ManaSource source = sources.First(candidate => candidate.Handle == option.Source);
                IEnumerable<ColorView?> names = option.ProducesOfAnyOneColor > 0 && option.ColorChoices.Count > 0
                    ? option.ColorChoices.Select(color => (ColorView?)color)
                    : [null];

                foreach (ColorView? named in names)
                {
                    if (After(option, named, out int paid) is not { } rest)
                    {
                        continue;
                    }

                    List<ManaSource> others = [.. sources.Where(other => other.Handle != option.Source)];
                    if (!Payable(others, [rest, .. kept]))
                    {
                        continue;
                    }

                    int made = option.Produces.Count + option.ProducesOfAnyOneColor;
                    (int, int, int, int) key = (option.LifeCost + option.DamageToController, Math.Max(made - paid, 0), source.Rank,
                        source.Colors.Count + (source.Colorless ? 1 : 0));
                    if (best is null || key.CompareTo(best.Value.Key) < 0)
                    {
                        best = (AutoPlay.Answer(option, named), key);
                    }
                }
            }

            if (best is { } chosen)
            {
                return chosen.Answer;
            }
        }

        return null;
    }

    /// <summary>
    /// The order sources are spent in, all else equal: a plain land, then another permanent that only makes mana, then a land
    /// that does something else as well (one that becomes a creature, a utility land), then a creature.
    /// </summary>
    private static int Rank(ObjectView? source)
    {
        if (source is null)
        {
            return 1;
        }

        if (IsCreature(source))
        {
            return 3;
        }

        if (IsLand(source))
        {
            // A land with an activated ability that is not a mana ability: one that becomes a creature, or a utility land.
            // A fetch land's search is how it makes its mana, so it is not one.
            bool more = CardReading.Plain(CardReading.Text(source)).Split('\n').Any(line =>
                line.IndexOf(':', StringComparison.Ordinal) is > 0 and int colon
                && !line[colon..].StartsWith(": add", StringComparison.Ordinal)
                && !line.Contains("search your library", StringComparison.Ordinal));
            return more ? 2 : 0;
        }

        return 1;
    }

    /// <summary>
    /// What to keep open while paying for the first play of a plan: the rest of the plan, then the instant held up through
    /// the opponent's turn - the counterspell the mana is kept back for, or else the best instant-speed play left out of
    /// the plan that the mana after it still covers.
    /// </summary>
    private void KeepOpen(GameClient client, PlayerView view, List<Candidate> plan, Candidate first, List<Candidate> candidates, int budget, int reserve)
    {
        _keepOpen = [];
        foreach (Candidate next in plan.Where(candidate => candidate != first).OrderByDescending(candidate => candidate.Gain))
        {
            _keepOpen.Add(NeedOf(client, next.Option));
        }

        if (view.ActivePlayer != client.Seat)
        {
            return;
        }

        int left = budget - plan.Sum(candidate => candidate.Cost);
        if (reserve > 0)
        {
            ObjectView? counter = client.MyHand()
                .Where(card => card.CardTypes.Contains(CardTypeView.Instant) && CardReading.Effect(CardReading.Text(card)).Kind == EffectKind.Counter)
                .OrderBy(card => card.ManaValue)
                .FirstOrDefault();
            if (counter is not null)
            {
                _keepOpen.Add(CostNeed(counter.ManaCost));
                return;
            }
        }

        Candidate? held = candidates
            .Where(candidate => !plan.Contains(candidate) && candidate.Option.Kind == ActionKindView.CastSpell && candidate.Cost <= left
                && Instantly(client, candidate.Option) && candidate.Option.Source != first.Option.Source)
            .OrderByDescending(candidate => candidate.Gain)
            .FirstOrDefault();
        if (held is not null)
        {
            _keepOpen.Add(NeedOf(client, held.Option));
        }
    }

    private static ManaNeed NeedOf(GameClient client, ActionOptionMessage option) =>
        option.Kind == ActionKindView.CastSpell && option.AbilityIndex == 0 && !option.WithoutPayingManaCost && client.Object(option.Source) is { } card
            ? CostNeed(card.ManaCost)
            : CostNeed(option.CostText);
}
