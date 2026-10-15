using Cantrip.Cards.Model;

namespace Cantrip.Cards;

/// <summary>
/// Reads a printed type line such as <c>Basic Land — Plains</c> into structured types.
/// </summary>
public static class TypeLineParser
{
    private const char EmDash = '—';

    /// <summary>
    /// Parses <paramref name="text"/>, optionally overriding the subtypes.
    /// </summary>
    /// <param name="explicitSubtypes">
    /// Supplied when a subtype contains a space, which a space-separated type line cannot express.
    /// </param>
    /// <exception cref="TypeLineFormatException">A word is not a known supertype or card type.</exception>
    public static TypeLine Parse(string? text, IReadOnlyList<string>? explicitSubtypes = null)
    {
        var typeLine = new TypeLine();
        if (string.IsNullOrWhiteSpace(text))
        {
            throw new TypeLineFormatException(text ?? string.Empty, "a type line is required.");
        }

        string input = text.Trim();
        string left = input;
        string right = string.Empty;

        int split = input.IndexOf(EmDash);
        if (split >= 0)
        {
            left = input[..split];
            right = input[(split + 1)..];
        }
        else
        {
            // A spaced hyphen is accepted so the files can be typed without an em dash. The test
            // is for " - " rather than "-" so hyphenated subtypes survive.
            int hyphen = input.IndexOf(" - ", StringComparison.Ordinal);
            if (hyphen >= 0)
            {
                left = input[..hyphen];
                right = input[(hyphen + 3)..];
            }
        }

        foreach (string word in Words(left))
        {
            if (Enum.TryParse(word, ignoreCase: false, out Supertype supertype))
            {
                typeLine.Supertypes.Add(supertype);
            }
            else if (Enum.TryParse(word, ignoreCase: false, out CardType cardType))
            {
                typeLine.CardTypes.Add(cardType);
            }
            else
            {
                throw new TypeLineFormatException(
                    input,
                    $"'{word}' is not a known supertype or card type. Subtypes belong after the dash.");
            }
        }

        if (typeLine.CardTypes.Count == 0)
        {
            throw new TypeLineFormatException(input, "at least one card type is required.");
        }

        if (explicitSubtypes is not null)
        {
            typeLine.Subtypes.AddRange(explicitSubtypes);
        }
        else
        {
            typeLine.Subtypes.AddRange(Words(right));
        }

        return typeLine;
    }

    private static IEnumerable<string> Words(string text) =>
        text.Split(' ', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
}

/// <summary>A type line could not be parsed.</summary>
public sealed class TypeLineFormatException : Exception
{
    public TypeLineFormatException(string input, string reason)
        : base($"Bad type line \"{input}\": {reason}")
    {
        Input = input;
    }

    public string Input { get; }
}
