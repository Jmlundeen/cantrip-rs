using Cantrip.Client.Tactics;

namespace Cantrip.BotLab;

/// <summary>How a fit came out, measured on positions it was not shown.</summary>
/// <param name="Before">Log loss of the model the fit started from, on the held-out positions.</param>
/// <param name="After">Log loss of the fitted model, on the same positions.</param>
/// <param name="Accuracy">How often the fitted model's favourite went on to win.</param>
public sealed record FitReport(int Samples, int Features, double Before, double After, double Accuracy);

/// <summary>
/// Fits the model's weights to who won: a logistic regression of the result onto <see cref="TacticalModel.Features"/>,
/// over positions from games already played.
/// <para>
/// Held towards the hand-written prior rather than towards nothing. A weight the games say a great deal about goes
/// where they say; one they say little about — a keyword the deck barely has, a card seen a dozen times — stays
/// near what a player would have guessed, which for a card's own weight is zero: worth what its size and keywords
/// say and no more. That is the difference between a model that has learned what Mother of Runes is worth and one
/// that has learned which card happened to be on the table in the forty games it saw.
/// </para>
/// <para>
/// The policy weights are not fitted. They are not features of a position, so the result of a game says nothing
/// about them through this regression, and they are carried across unchanged.
/// </para>
/// </summary>
public static class Fit
{
    /// <summary>A card is given a weight of its own only once it has been seen in this many positions.</summary>
    private const int Seen = 300;

    /// <param name="general">
    /// How firmly a general weight is held to the prior: the penalty on its squared distance from the prior, divided
    /// by the number of positions, so that more games loosen the hold and a larger number tightens it.
    /// </param>
    /// <param name="card">The same for a card's own weight, held to zero.</param>
    public static (TacticalModel Model, FitReport Report) Run(
        IReadOnlyList<Sample> samples,
        TacticalModel from,
        double general = 200,
        double card = 400,
        int epochs = 600)
    {
        ArgumentNullException.ThrowIfNull(samples);
        ArgumentNullException.ThrowIfNull(from);

        TacticalModel prior = TacticalModel.Prior();

        // The features worth fitting: every general one seen, and every card seen often enough to say something.
        Dictionary<string, int> counts = new(StringComparer.Ordinal);
        foreach (Sample sample in samples)
        {
            foreach (string name in sample.Features.Keys)
            {
                counts[name] = counts.GetValueOrDefault(name) + 1;
            }
        }

        List<string> names = [.. counts
            .Where(pair => !pair.Key.StartsWith(TacticalModel.CardPrefix, StringComparison.Ordinal) || pair.Value >= Seen)
            .Select(pair => pair.Key)
            .Order(StringComparer.Ordinal)];

        Dictionary<string, int> index = names
            .Select((name, i) => (name, i))
            .ToDictionary(pair => pair.name, pair => pair.i, StringComparer.Ordinal);

        double[] target = [.. names.Select(prior.Weight)];
        double positions = Math.Max(samples.Count, 1);
        double[] strength = [.. names.Select(name =>
            (name.StartsWith(TacticalModel.CardPrefix, StringComparison.Ordinal) ? card : general) / positions)];
        double[] weights = [.. names.Select(from.Weight)];

        // Every tenth game is kept back to measure on: whole games, since both seats see the same position and a
        // game's turns resemble each other, and a position measured on must not have a twin among those fitted to.
        List<(int[] At, double[] Amount, bool Won)> train = [];
        List<(int[] At, double[] Amount, bool Won)> held = [];

        for (int i = 0; i < samples.Count; i++)
        {
            List<KeyValuePair<string, double>> known = [.. samples[i].Features.Where(pair => index.ContainsKey(pair.Key))];
            (int[], double[], bool) row = ([.. known.Select(pair => index[pair.Key])], [.. known.Select(pair => pair.Value)], samples[i].Won);
            (samples[i].Game / 2 % 10 == 9 ? held : train).Add(row);
        }

        double before = Loss(held, weights).Loss;

        // Nothing to fit to: every game failed, or was drawn. What came in goes out.
        if (train.Count == 0)
        {
            return (from, new FitReport(samples.Count, names.Count, before, before, 0));
        }

        // Adam over the whole of it each step: a few tens of features and a few hundred thousand rows.
        double[] mean = new double[names.Count];
        double[] variance = new double[names.Count];
        const double Rate = 0.02;
        const double Beta1 = 0.9;
        const double Beta2 = 0.999;

        for (int epoch = 1; epoch <= epochs; epoch++)
        {
            double[] gradient = new double[names.Count];
            double[] current = weights;

            Parallel.ForEach(
                System.Collections.Concurrent.Partitioner.Create(0, train.Count),
                () => new double[names.Count],
                (range, _, local) =>
                {
                    for (int row = range.Item1; row < range.Item2; row++)
                    {
                        (int[] at, double[] amount, bool won) = train[row];
                        double error = Chance(at, amount, current) - (won ? 1 : 0);
                        for (int k = 0; k < at.Length; k++)
                        {
                            local[at[k]] += error * amount[k];
                        }
                    }

                    return local;
                },
                local =>
                {
                    lock (gradient)
                    {
                        for (int k = 0; k < gradient.Length; k++)
                        {
                            gradient[k] += local[k];
                        }
                    }
                });

            for (int k = 0; k < weights.Length; k++)
            {
                double slope = (gradient[k] / Math.Max(train.Count, 1)) + (2 * strength[k] * (weights[k] - target[k]));
                mean[k] = (Beta1 * mean[k]) + ((1 - Beta1) * slope);
                variance[k] = (Beta2 * variance[k]) + ((1 - Beta2) * slope * slope);
                double step = mean[k] / (1 - Math.Pow(Beta1, epoch)) / (Math.Sqrt(variance[k] / (1 - Math.Pow(Beta2, epoch))) + 1e-8);
                weights[k] -= Rate * step;
            }
        }

        (double after, double accuracy) = Loss(held, weights);

        // The whole model is written: the fitted weights over what came in, so the policy weights, and the weight of
        // a card these games did not see enough of to refit, are carried across as they were.
        TacticalModel fitted = from.With(names.Select((name, i) => KeyValuePair.Create(name, weights[i])));

        return (fitted, new FitReport(samples.Count, names.Count, before, after, accuracy));
    }

    private static double Chance(int[] at, double[] amount, double[] weights)
    {
        double sum = 0;
        for (int k = 0; k < at.Length; k++)
        {
            sum += weights[at[k]] * amount[k];
        }

        return 1 / (1 + Math.Exp(-sum));
    }

    private static (double Loss, double Accuracy) Loss(List<(int[] At, double[] Amount, bool Won)> rows, double[] weights)
    {
        double loss = 0;
        int right = 0;

        foreach ((int[] at, double[] amount, bool won) in rows)
        {
            double chance = Math.Clamp(Chance(at, amount, weights), 1e-9, 1 - 1e-9);
            loss -= Math.Log(won ? chance : 1 - chance);
            right += (chance >= 0.5) == won ? 1 : 0;
        }

        return (loss / Math.Max(rows.Count, 1), right / (double)Math.Max(rows.Count, 1));
    }
}
