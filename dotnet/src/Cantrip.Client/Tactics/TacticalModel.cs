using System.Globalization;
using System.Reflection;
using Cantrip.Protocol;

namespace Cantrip.Client.Tactics;

/// <summary>
/// What a position is worth to the seat looking at it: a weighted sum over what it can see, in log-odds of winning.
/// <para>
/// The model is linear and symmetric. Each side's life, cards in hand, lands and permanents are turned into the same
/// features, a permanent's own being its size, its keywords and its name, and the position is worth this seat's sum
/// less the opponent's. That shape is what lets <see cref="TacticalPlay"/> use it without a rules engine: the worth
/// of a creature is a number it can subtract when a block would kill it, and the worth of a life total is a curve it
/// can read damage off, so "which block is best" is arithmetic over a board it never has to simulate past combat.
/// </para>
/// <para>
/// The weights are fitted, not chosen. <see cref="Prior"/> is a hand-written starting point good enough to play
/// from; the weights that ship (<c>tactical-model.txt</c>, an embedded resource) are a logistic regression of who
/// won onto <see cref="Features"/>, over games this seat played against itself, held towards the prior where the
/// games say little (<c>tools\bot-train.cmd</c>). A card the fit has not met has no weight of its own and is worth
/// what its size and keywords say, so the model plays any deck and plays the ones it trained on better.
/// </para>
/// </summary>
public sealed class TacticalModel
{
    /// <summary>What a card's own weight is filed under: its name, since that is what a card is to a player.</summary>
    public const string CardPrefix = "card:";

    private const string Resource = "tactical-model.txt";

    private static readonly Lazy<TacticalModel> Shipped = new(LoadShipped);

    private readonly Dictionary<string, double> _weights;

    private TacticalModel(Dictionary<string, double> weights) => _weights = weights;

    /// <summary>The fitted weights this build ships with, over the prior for anything they do not name.</summary>
    public static TacticalModel Default => Shipped.Value;

    /// <summary>
    /// The hand-written weights, before any game was played: a creature is worth its size, evasion is worth more on
    /// a bigger body, and a point of life is worth more the fewer there are.
    /// </summary>
    public static TacticalModel Prior() => new(new Dictionary<string, double>(StringComparer.Ordinal)
    {
        // Life, per point, in bands: the last five are worth far more than the fifth five.
        ["life.0-5"] = 0.25,
        ["life.5-10"] = 0.12,
        ["life.10-20"] = 0.05,
        ["life.20+"] = 0.02,

        ["hand"] = 0.25,
        ["land.1-4"] = 0.22,
        ["land.5+"] = 0.06,
        ["active"] = 0.10,

        ["creature"] = 0.10,
        ["creature.power"] = 0.15,
        ["creature.toughness"] = 0.08,
        ["creature.flying"] = 0.06,
        ["creature.firststrike"] = 0.04,
        ["creature.doublestrike"] = 0.12,
        ["creature.deathtouch"] = 0.20,
        ["creature.lifelink"] = 0.05,
        ["creature.trample"] = 0.02,
        ["creature.vigilance"] = 0.08,
        ["creature.menace"] = 0.03,
        ["creature.reach"] = 0.03,
        ["creature.hexproof"] = 0.10,
        ["creature.indestructible"] = 0.30,
        ["creature.defender"] = -0.15,
        ["creature.prowess"] = 0.05,
        ["creature.token"] = -0.03,
        ["creature.text"] = 0.08,

        ["permanent.artifact"] = 0.15,
        ["permanent.enchantment"] = 0.15,
        ["permanent.planeswalker"] = 0.50,
        ["permanent.loyalty"] = 0.08,
        ["permanent.manavalue"] = 0.08,

        // How the seat plays rather than what it sees: these are not fitted to results, only set.
        // The least a play has to gain to be worth making.
        ["policy.margin"] = 0.03,
        // What a mana spent is worth in itself, so a turn's mana is used when two plans gain the same.
        ["policy.tempo"] = 0.02,
        // What casting a spell this seat cannot read is taken to gain, over the card it costs.
        ["policy.unread"] = 0.05,
        // Choosing what comes next off the top of a library: a scry, a surveil, a look.
        ["policy.select"] = 0.06,
        // An ability that makes its own creature bigger for good.
        ["policy.grow"] = 0.15,
        // A pump, a counter or an attachment on its own creature.
        ["policy.help"] = 0.20,
        // Hurting a permanent some way that does not remove it, as a share of what it is worth.
        ["policy.harm"] = 0.30,
        // Returning a permanent to its owner's hand, as a share of what it is worth.
        ["policy.bounce"] = 0.40,
        // How much of the damage an attack leaves it open to on the way back is counted against attacking.
        ["policy.crackback"] = 0.50,
        // What it costs to end its turn dead to an attack it could not block.
        ["policy.exposed"] = 3.00,
        // "Whenever this attacks" and "whenever you attack", each time one triggers.
        ["policy.attacktrigger"] = 0.25,
        // "Whenever this deals combat damage to a player", each time one does.
        ["policy.connect"] = 0.20,
        // A whole turn more: a card, a land, an attack.
        ["policy.extraturn"] = 0.90,
        // Whether mana is kept back on its own turn for a counterspell in hand: 1 for yes, 0 for never.
        ["policy.holdcounter"] = 1.00,
        // What a turn's plays have to gain before that mana is spent on them after all.
        ["policy.tapout"] = 0.50,
        // An artifact that makes mana is cast for what a land would be worth, this much of it per mana; 0 for never.
        ["policy.rocks"] = 1.00,
        // A card a permanent can be cashed in for (Mind Stone, a Bauble) is counted when it is cast, this much of it.
        ["policy.stored"] = 0.75,
        // What a point of loyalty a loyalty ability adds or spends is counted as, in loyalty's own weight; 0 for the
        // old way, where only an ability that adds loyalty or aims at something was ever used.
        ["policy.loyalty"] = 1.00,
        // The emblem a planeswalker's last ability leaves behind.
        ["policy.emblem"] = 1.50,
        // Whether attackers are sent at an opponent's planeswalker where that is better than at the player: 1, or 0.
        ["policy.walkers"] = 1.00,
        // Whether a Vehicle is crewed to attack with: 1, or 0 for never.
        ["policy.crew"] = 1.00,
        // The life this seat must have left after paying two for a Phyrexian mana symbol it has no mana for; 0 for
        // never paying life for one.
        ["policy.phyrexian"] = 8.00,
        // Whether a search that puts its card onto the battlefield takes the one worth most there, whatever it costs.
        ["policy.fetch"] = 1.00,
        // What a card an opponent is made to discard is worth, in cards; 0 for the old way, where making a player
        // discard was worth the same whatever they held.
        ["policy.discard"] = 1.00,
        // The turns of its upkeep cost a seat must have the life for before it puts another counter on a permanent
        // that costs it life for each (The One Ring); 0 for never asking.
        ["policy.burden"] = 3.00,
        // Whether a seat refuses to pay the last of its life for mana - giving the spell up, lowering its X, declining
        // an unless-cost - rather than paying it: 1, or 0 for paying it as it always did.
        ["policy.lastlife"] = 1.00,
        // What a creature cheated onto the battlefield (Sneak Attack, Through the Breach, Exhume, Animate Dead) is
        // counted as, of its worth there; 0 for reading those cards as nothing, as the seat did.
        ["policy.cheat"] = 1.00,
        // How many cards more than a card a spell must be worth before a counterspell is spent on it, while the
        // opponent holds three or more cards (scaled down with fewer); 0 for countering anything worth a card.
        ["policy.counterbar"] = 0.00,
        // Whether a Mox paid for with a card (Chrome Mox, Mox Diamond) is cast while the seat is building its mana and
        // has the card to spare: 1, or 0 for never, as it was.
        ["policy.imprint"] = 1.00,
        // What a planeswalker in hand is counted for beyond its type: the loyalty it enters with and the best of its
        // untargeted abilities used at once, this much of it; 0 for its type alone, as before.
        ["policy.walkercast"] = 1.00,
        // Whether a graveyard-hate permanent (Soul-Guide Lantern, Ghost Vacuum) is used as one: in answer to a spell
        // that brings a creature back, kept rather than cashed in while an opponent's graveyard holds a creature worth
        // bringing back, and aimed at their creature cards first: 1, or 0 for a cantrip, as it was.
        ["policy.graveyard"] = 1.00,
        // Draw-go: with mana kept up for a counterspell on its own turn, an instant or a flash card waits for the end
        // of the opponent's turn, when the counterspell has not been needed: 1, or 0 to cast it now, as before.
        ["policy.drawgo"] = 0.00,
    });

    /// <summary>Reads weights as <c>name = value</c> lines over the prior; <c>#</c> starts a comment.</summary>
    public static TacticalModel Parse(string text)
    {
        ArgumentNullException.ThrowIfNull(text);

        TacticalModel model = Prior();
        foreach (string raw in text.Split('\n'))
        {
            string line = raw.Trim();
            int equals = line.LastIndexOf('=');
            if (line.Length == 0 || line[0] == '#' || equals <= 0)
            {
                continue;
            }

            if (double.TryParse(line[(equals + 1)..].Trim(), NumberStyles.Float, CultureInfo.InvariantCulture, out double value))
            {
                model._weights[line[..equals].Trim()] = value;
            }
        }

        return model;
    }

    /// <summary>The weights as <see cref="Parse"/> reads them back, general ones first and then the cards by name.</summary>
    public string Write(string heading = "")
    {
        var text = new System.Text.StringBuilder();
        foreach (string line in heading.Split('\n', StringSplitOptions.RemoveEmptyEntries))
        {
            text.Append("# ").Append(line.TrimEnd()).Append('\n');
        }

        foreach ((string name, double value) in _weights
            .OrderBy(pair => pair.Key.StartsWith(CardPrefix, StringComparison.Ordinal))
            .ThenBy(pair => pair.Key, StringComparer.Ordinal))
        {
            text.Append(name).Append(" = ").Append(value.ToString("0.####", CultureInfo.InvariantCulture)).Append('\n');
        }

        return text.ToString();
    }

    /// <summary>Every weight by name, for the fit to start from and to write back.</summary>
    public IReadOnlyDictionary<string, double> Weights => _weights;

    /// <summary>A model with these weights in place of this one's, name for name.</summary>
    public TacticalModel With(IEnumerable<KeyValuePair<string, double>> weights)
    {
        ArgumentNullException.ThrowIfNull(weights);

        var model = new TacticalModel(new Dictionary<string, double>(_weights, StringComparer.Ordinal));
        foreach ((string name, double value) in weights)
        {
            model._weights[name] = value;
        }

        return model;
    }

    public double Weight(string name) => _weights.GetValueOrDefault(name);

    // --- What things are worth ---------------------------------------------------------------------------------------

    /// <summary>What a life total is worth to whoever has it. Concave: zero at zero, steep to five, shallow past twenty.</summary>
    public double Life(int life)
    {
        double worth = 0;
        Bands(life, (name, amount) => worth += Weight(name) * amount);
        return worth;
    }

    /// <summary>What a card in hand is worth to whoever holds it, sight unseen.</summary>
    public double Card => Weight("hand");

    /// <summary>
    /// What a permanent on the battlefield is worth to whoever controls it. A land is worth nothing here: lands are
    /// counted, not weighed.
    /// </summary>
    public double Worth(ObjectView permanent) => Worth(permanent, CardReading.KeywordsOf(permanent));

    /// <summary>As <see cref="Worth(ObjectView)"/>, for a caller that has already read the keywords.</summary>
    public double Worth(ObjectView permanent, Keywords keywords)
    {
        ArgumentNullException.ThrowIfNull(permanent);

        double worth = 0;
        Permanent(permanent, keywords, (name, amount) => worth += Weight(name) * amount);
        return worth;
    }

    /// <summary>
    /// What a card would be worth on the battlefield, for one that is not there yet: a creature in hand, at its
    /// printed size.
    /// </summary>
    public double WorthOnceCast(ObjectView card) => Worth(card);

    /// <summary>What the position is worth to the seat whose view it is: the sum of <see cref="Features"/>, weighted.</summary>
    public double Evaluate(PlayerView view)
    {
        double worth = 0;
        foreach ((string name, double amount) in Features(view))
        {
            worth += Weight(name) * amount;
        }

        return worth;
    }

    // --- Features: the one place that says what the model sees -------------------------------------------------------

    /// <summary>
    /// The position as the model sees it, from the viewer's side: every feature that is not zero, this seat's count
    /// of it less its opponents'. What <see cref="Evaluate"/> weighs and what the fit regresses the result onto, so
    /// the two cannot disagree about what a feature means.
    /// </summary>
    public static Dictionary<string, double> Features(PlayerView view)
    {
        ArgumentNullException.ThrowIfNull(view);

        Dictionary<string, double> features = new(StringComparer.Ordinal);
        void Add(string name, double amount)
        {
            if (amount != 0)
            {
                features[name] = features.GetValueOrDefault(name) + amount;
            }
        }

        foreach (PlayerSummaryView player in view.Players)
        {
            if (player.Status != PlayerStatusView.Active)
            {
                continue;
            }

            double side = player.Seat == view.Viewer ? 1 : -1;
            Bands(player.Life, (name, amount) => Add(name, side * amount));
            Add("hand", side * player.HandCount);
        }

        Add("active", view.ActivePlayer == view.Viewer ? 1 : -1);

        Dictionary<int, int> lands = [];
        foreach (ObjectView permanent in view.Objects)
        {
            if (permanent.Zone != ZoneKindView.Battlefield)
            {
                continue;
            }

            if (permanent.CardTypes.Contains(CardTypeView.Land) && !permanent.CardTypes.Contains(CardTypeView.Creature))
            {
                lands[permanent.Controller] = lands.GetValueOrDefault(permanent.Controller) + 1;
                continue;
            }

            double side = permanent.Controller == view.Viewer ? 1 : -1;
            Permanent(permanent, CardReading.KeywordsOf(permanent), (name, amount) => Add(name, side * amount));
        }

        foreach ((int seat, int count) in lands)
        {
            double side = seat == view.Viewer ? 1 : -1;
            Add("land.1-4", side * Math.Min(count, 4));
            Add("land.5+", side * Math.Max(count - 4, 0));
        }

        return features;
    }

    private static void Bands(int life, Action<string, double> add)
    {
        int left = Math.Max(life, 0);
        add("life.0-5", Math.Min(left, 5));
        add("life.5-10", Math.Clamp(left - 5, 0, 5));
        add("life.10-20", Math.Clamp(left - 10, 0, 10));
        add("life.20+", Math.Clamp(left - 20, 0, 20));
    }

    private static void Permanent(ObjectView permanent, Keywords keywords, Action<string, double> add)
    {
        if (!permanent.Known || permanent.FaceDown)
        {
            // A face-down permanent is a nameless 2/2 to everyone, its controller's own estimate included.
            add("creature", 1);
            add("creature.power", 2);
            add("creature.toughness", 2);
            return;
        }

        if (permanent.CardTypes.Contains(CardTypeView.Creature))
        {
            double power = Math.Clamp(permanent.Power ?? 0, 0, 12);
            double toughness = Math.Clamp(permanent.Toughness ?? 0, 0, 12);

            add("creature", 1);
            add("creature.power", power);
            add("creature.toughness", toughness);

            // Evasion and its like are worth what they carry: flying on a 1/1 is a point of damage a turn, on a 4/4 four.
            Scaled(keywords, Keywords.Flying, "creature.flying", power, add);
            Scaled(keywords, Keywords.FirstStrike, "creature.firststrike", power, add);
            Scaled(keywords, Keywords.DoubleStrike, "creature.doublestrike", power, add);
            Scaled(keywords, Keywords.Lifelink, "creature.lifelink", power, add);
            Scaled(keywords, Keywords.Trample, "creature.trample", power, add);
            Scaled(keywords, Keywords.Menace, "creature.menace", power, add);
            Flat(keywords, Keywords.Deathtouch, "creature.deathtouch", add);
            Flat(keywords, Keywords.Vigilance, "creature.vigilance", add);
            Flat(keywords, Keywords.Reach, "creature.reach", add);
            Flat(keywords, Keywords.Hexproof, "creature.hexproof", add);
            Flat(keywords, Keywords.Indestructible, "creature.indestructible", add);
            Flat(keywords, Keywords.Defender, "creature.defender", add);
            Flat(keywords, Keywords.Prowess, "creature.prowess", add);

            if (permanent.Kind == ObjectKindView.Token)
            {
                add("creature.token", 1);
            }
            else if (permanent.OracleText.Length > 0)
            {
                // A creature card that says anything at all usually says something good.
                add("creature.text", 1);
            }
        }
        else if (permanent.CardTypes.Contains(CardTypeView.Planeswalker))
        {
            add("permanent.planeswalker", 1);
            add("permanent.loyalty", permanent.Counters.Find(counter => counter.Name == "loyalty")?.Count ?? 0);
        }
        else if (permanent.Kind != ObjectKindView.Token)
        {
            add(permanent.CardTypes.Contains(CardTypeView.Enchantment) ? "permanent.enchantment" : "permanent.artifact", 1);
            add("permanent.manavalue", Math.Clamp(permanent.ManaValue, 0, 8));
        }

        // The card's own weight: whatever its size and keywords do not say about it. A token's is filed under the
        // token's name, which is all a token has.
        if (permanent.Name.Length > 0)
        {
            add(CardPrefix + permanent.Name, 1);
        }
    }

    private static void Scaled(Keywords has, Keywords keyword, string name, double power, Action<string, double> add)
    {
        if ((has & keyword) != 0)
        {
            add(name, power);
        }
    }

    private static void Flat(Keywords has, Keywords keyword, string name, Action<string, double> add)
    {
        if ((has & keyword) != 0)
        {
            add(name, 1);
        }
    }

    private static TacticalModel LoadShipped()
    {
        Assembly assembly = typeof(TacticalModel).Assembly;
        string? name = Array.Find(assembly.GetManifestResourceNames(), resource => resource.EndsWith(Resource, StringComparison.Ordinal));
        if (name is null || assembly.GetManifestResourceStream(name) is not { } stream)
        {
            return Prior();
        }

        using var reader = new StreamReader(stream);
        return Parse(reader.ReadToEnd());
    }
}
