namespace Cantrip.Client.Presentation;

/// <summary>
/// Where each row of a long list stands, for a list that only makes the rows in view: each row's height - a guess until
/// the row has been shown and measured - and from those where each one starts, how tall the whole list is, and which rows
/// a window onto it covers.
/// <para>
/// The game log is the list this is for. Every line of a long game made as a control, kept and laid out whether or not
/// it was scrolled to, made each action cost more the longer the game went on; made only for the lines in view, the
/// log costs the same on the thousandth line as on the first.
/// </para>
/// </summary>
public sealed class RowExtents
{
    private readonly List<double> _heights = [];
    private readonly List<bool> _measured = [];
    private readonly List<double> _tops = [];
    private int _settledUpTo;

    /// <param name="gap">The space between one row and the next.</param>
    public RowExtents(double gap = 0)
    {
        ArgumentOutOfRangeException.ThrowIfNegative(gap);
        Gap = gap;
    }

    public double Gap { get; }

    public int Count => _heights.Count;

    /// <summary>How tall the list stands: every row, and the gaps between them.</summary>
    public double Total
    {
        get
        {
            if (Count == 0)
            {
                return 0;
            }

            Settle();
            return _tops[^1] + _heights[^1];
        }
    }

    /// <summary>Adds a row at the end, as tall as it is guessed to be until it is measured.</summary>
    public void Add(double guess)
    {
        _heights.Add(Math.Max(0, guess));
        _measured.Add(false);
        _tops.Add(0);
    }

    /// <summary>Forgets every row.</summary>
    public void Clear()
    {
        _heights.Clear();
        _measured.Clear();
        _tops.Clear();
        _settledUpTo = 0;
    }

    /// <summary>Whether a row has been measured since it was added or last guessed again.</summary>
    public bool IsMeasured(int index) => _measured[index];

    public double Height(int index) => _heights[index];

    /// <summary>Where a row starts, from the top of the list.</summary>
    public double Top(int index)
    {
        ArgumentOutOfRangeException.ThrowIfNegative(index);
        ArgumentOutOfRangeException.ThrowIfGreaterThanOrEqual(index, Count);
        Settle();
        return _tops[index];
    }

    /// <summary>Records a row's measured height. Returns how much taller it turned out than it was taken to be.</summary>
    public double Measure(int index, double height)
    {
        double change = Math.Max(0, height) - _heights[index];
        _heights[index] = Math.Max(0, height);
        _measured[index] = true;
        if (change != 0)
        {
            _settledUpTo = Math.Min(_settledUpTo, index + 1);
        }

        return change;
    }

    /// <summary>
    /// Takes every row as unmeasured again, keeping its last height as the guess: for when the rows are given another
    /// width, and those that wrapped may no longer.
    /// </summary>
    public void Remeasure()
    {
        for (int i = 0; i < _measured.Count; i++)
        {
            _measured[i] = false;
        }
    }

    /// <summary>
    /// The rows any part of which lies between two heights, first and last; <c>(0, -1)</c> when none does.
    /// </summary>
    public (int First, int Last) Within(double top, double bottom)
    {
        if (Count == 0 || bottom < top)
        {
            return (0, -1);
        }

        Settle();

        // The first row that ends below the top, then the last that starts above the bottom.
        int first = FirstAtOrAfter(index => _tops[index] + _heights[index] > top);
        if (first == Count)
        {
            return (0, -1);
        }

        int last = FirstAtOrAfter(index => _tops[index] >= bottom) - 1;
        return last < first ? (0, -1) : (first, last);
    }

    /// <summary>The first row from which a test that only ever turns from false to true holds; the count if none.</summary>
    private int FirstAtOrAfter(Func<int, bool> holds)
    {
        int low = 0;
        int high = Count;
        while (low < high)
        {
            int middle = low + ((high - low) / 2);
            if (holds(middle))
            {
                high = middle;
            }
            else
            {
                low = middle + 1;
            }
        }

        return low;
    }

    /// <summary>Works out where the rows start, from the first whose place a change may have moved.</summary>
    private void Settle()
    {
        for (int i = Math.Max(1, _settledUpTo); i < Count; i++)
        {
            _tops[i] = _tops[i - 1] + _heights[i - 1] + Gap;
        }

        _settledUpTo = Count;
    }
}
