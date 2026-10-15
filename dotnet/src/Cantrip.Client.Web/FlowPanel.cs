using Windows.Foundation;

namespace Cantrip.Client.Web;

/// <summary>
/// Children left to right, onto a new line when the next would not fit, as the dock's buttons are laid out.
/// <see cref="AlignEnd"/> pushes each line to the right edge, as the choice window's buttons sit. A child aligned to
/// the centre vertically sits in the middle of its line.
/// </summary>
public sealed class FlowPanel : Panel
{
    public double HorizontalSpacing { get; init; } = 8;
    public double VerticalSpacing { get; init; } = 6;
    public bool AlignEnd { get; init; }

    protected override Size MeasureOverride(Size availableSize)
    {
        double limit = availableSize.Width;
        double x = 0;
        double lineHeight = 0;
        double width = 0;
        double height = 0;
        bool first = true;

        foreach (UIElement child in Children)
        {
            child.Measure(new Size(limit, double.PositiveInfinity));
            Size size = child.DesiredSize;
            if (child.Visibility == Visibility.Collapsed)
            {
                continue;
            }

            if (x > 0 && x + HorizontalSpacing + size.Width > limit)
            {
                width = Math.Max(width, x);
                height += lineHeight + VerticalSpacing;
                x = 0;
                lineHeight = 0;
            }

            x += (x > 0 ? HorizontalSpacing : 0) + size.Width;
            lineHeight = Math.Max(lineHeight, size.Height);
            first = false;
        }

        if (first)
        {
            return new Size(0, 0);
        }

        width = Math.Max(width, x);
        height += lineHeight;
        return new Size(double.IsInfinity(limit) ? width : Math.Min(width, limit), height);
    }

    protected override Size ArrangeOverride(Size finalSize)
    {
        List<List<UIElement>> lines = [[]];
        double x = 0;

        foreach (UIElement child in Children)
        {
            if (child.Visibility == Visibility.Collapsed)
            {
                continue;
            }

            double w = child.DesiredSize.Width;
            if (x > 0 && x + HorizontalSpacing + w > finalSize.Width)
            {
                lines.Add([]);
                x = 0;
            }

            lines[^1].Add(child);
            x += (x > 0 ? HorizontalSpacing : 0) + w;
        }

        double y = 0;
        foreach (List<UIElement> line in lines)
        {
            if (line.Count == 0)
            {
                continue;
            }

            double lineWidth = line.Sum(c => c.DesiredSize.Width) + (HorizontalSpacing * (line.Count - 1));
            double lineHeight = line.Max(c => c.DesiredSize.Height);
            x = AlignEnd ? Math.Max(0, finalSize.Width - lineWidth) : 0;

            foreach (UIElement child in line)
            {
                Size size = child.DesiredSize;
                double down = child is FrameworkElement { VerticalAlignment: VerticalAlignment.Center } ? (lineHeight - size.Height) / 2 : 0;
                child.Arrange(new Rect(x, y + down, size.Width, size.Height));
                x += size.Width + HorizontalSpacing;
            }

            y += lineHeight + VerticalSpacing;
        }

        return finalSize;
    }
}
