namespace Cantrip.Client.Presentation;

/// <summary>
/// A list of equal cells laid out in rows of a fixed number across, of which only the cells near a window onto it are
/// made: how tall the whole list stands, and which cells - first, and one past the last - the rows any part of which
/// lies within the window, and some rows either side of it, hold.
/// <para>
/// The deck editor's search is the list this is for: every card the site has, laid out at once, made each change of the
/// search cost what the whole pool costs; made only for the rows in view, it costs what a screenful costs, and every
/// card is still there to be scrolled to.
/// </para>
/// </summary>
/// <param name="Count">How many cells the list has.</param>
/// <param name="Across">How many cells a row holds; at least one.</param>
/// <param name="CellHeight">How tall a cell is.</param>
/// <param name="Gap">The space between one row and the next.</param>
public readonly record struct GridWindow(int Count, int Across, double CellHeight, double Gap)
{
    /// <summary>How many rows the cells fill.</summary>
    public int Rows => Count <= 0 ? 0 : (Count + Math.Max(1, Across) - 1) / Math.Max(1, Across);

    /// <summary>How tall the list stands: every row, and the gaps between them.</summary>
    public double Height => Rows == 0 ? 0 : (Rows * (CellHeight + Gap)) - Gap;

    /// <summary>Where a cell's row starts, from the top of the list.</summary>
    public double Top(int index) => index / Math.Max(1, Across) * (CellHeight + Gap);

    /// <summary>
    /// The cells to make for a window onto the list, from its top down so far: those of every row any part of which is
    /// in it, and of as many rows again above and below as <paramref name="spare"/> says. Empty, as (0, 0), when none is.
    /// </summary>
    public (int First, int End) Within(double top, double height, int spare = 0)
    {
        if (Count <= 0 || height < 0)
        {
            return (0, 0);
        }

        int across = Math.Max(1, Across);
        double pitch = CellHeight + Gap;
        if (pitch <= 0)
        {
            return (0, Count);
        }

        double bottom = top + height;

        // The first row that ends below the top, and the last that starts above the bottom.
        int firstRow = (int)Math.Floor((top + Gap) / pitch);
        int lastRow = (int)Math.Ceiling(bottom / pitch) - 1;
        firstRow = Math.Max(0, firstRow - Math.Max(0, spare));
        lastRow = Math.Min(Rows - 1, lastRow + Math.Max(0, spare));
        if (lastRow < firstRow)
        {
            return (0, 0);
        }

        return (firstRow * across, Math.Min(Count, (lastRow + 1) * across));
    }
}
