namespace Cantrip.Client.Presentation;

/// <summary>An upright box: where its top left corner is, and how wide and tall it is.</summary>
public readonly record struct Box(double X, double Y, double Width, double Height)
{
    public double Right => X + Width;

    public double Bottom => Y + Height;

    /// <summary>Whether two boxes share any area: boxes that only touch along an edge do not.</summary>
    public bool Overlaps(Box other) => X < other.Right && other.X < Right && Y < other.Bottom && other.Y < Bottom;

    /// <summary>The box both share; null when they share no area.</summary>
    public Box? Intersect(Box other)
    {
        if (!Overlaps(other))
        {
            return null;
        }

        double left = Math.Max(X, other.X);
        double top = Math.Max(Y, other.Y);
        return new Box(left, top, Math.Min(Right, other.Right) - left, Math.Min(Bottom, other.Bottom) - top);
    }

    /// <summary>What is left of this box with another taken out of it: up to four boxes, none overlapping.</summary>
    public IEnumerable<Box> Without(Box cut)
    {
        if (Intersect(cut) is not { } shared)
        {
            yield return this;
            yield break;
        }

        // Above and below the cut, the full width; beside it, only as tall as the cut is here.
        if (shared.Y > Y)
        {
            yield return new Box(X, Y, Width, shared.Y - Y);
        }

        if (shared.Bottom < Bottom)
        {
            yield return new Box(X, shared.Bottom, Width, Bottom - shared.Bottom);
        }

        if (shared.X > X)
        {
            yield return new Box(X, shared.Y, shared.X - X, shared.Height);
        }

        if (shared.Right < Right)
        {
            yield return new Box(shared.Right, shared.Y, Right - shared.Right, shared.Height);
        }
    }
}

/// <summary>
/// A rectangle dragged out over cards laid out in a pile or a column, to pick several at once: which cards it touches.
/// A card counts when the rectangle touches any part of it that can be seen - not only the cards wholly inside it - and a
/// card covered where the rectangle crosses it, by the cards drawn over it, is not touched there: a column of cards
/// showing only their names is picked by the names the rectangle crosses.
/// </summary>
public static class Marquee
{
    /// <summary>The box between where the drag started and where it is now, whichever way it went; never thinner than one.</summary>
    public static Box Between(double fromX, double fromY, double toX, double toY) =>
        new(Math.Min(fromX, toX), Math.Min(fromY, toY), Math.Max(1, Math.Abs(toX - fromX)), Math.Max(1, Math.Abs(toY - fromY)));

    /// <summary>
    /// The cards a rectangle touches, by their place in <paramref name="drawOrder"/> - each card's box, the first drawn
    /// first, so a later one covers an earlier one - in that order.
    /// </summary>
    public static IReadOnlyList<int> Touched(IReadOnlyList<Box> drawOrder, Box marquee)
    {
        ArgumentNullException.ThrowIfNull(drawOrder);
        List<int> touched = [];
        for (int i = 0; i < drawOrder.Count; i++)
        {
            if (drawOrder[i].Intersect(marquee) is not { } crossed)
            {
                continue;
            }

            // What of the crossed part no card drawn over it covers.
            List<Box> showing = [crossed];
            for (int over = i + 1; over < drawOrder.Count && showing.Count > 0; over++)
            {
                Box cover = drawOrder[over];
                showing = [.. showing.SelectMany(part => part.Without(cover))];
            }

            if (showing.Count > 0)
            {
                touched.Add(i);
            }
        }

        return touched;
    }
}
