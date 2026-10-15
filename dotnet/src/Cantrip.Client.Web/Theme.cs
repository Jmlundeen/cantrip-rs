using Windows.UI;

namespace Cantrip.Client.Web;

/// <summary>
/// The client's look: its surfaces, its meaning-colors (whose turn it is, what is ready, what is in danger),
/// and a dark-and-neutral palette that leaves the cards as the only saturated color on screen.
/// </summary>
public static class Theme
{
    // --- Surfaces, darkest to lightest ---

    public static readonly Color Window = FromHex("#111318");
    public static readonly Color Bar = FromHex("#191c22");
    public static readonly Color Panel = FromHex("#1d2127");
    public static readonly Color HalfIdle = FromHex("#1c2026");
    public static readonly Color HalfActive = FromHex("#2b313a");
    public static readonly Color RailIdle = FromHex("#171a1f");
    public static readonly Color RailActive = FromHex("#242930");
    public static readonly Color Line = FromHex("#2c313a");
    public static readonly Color ControlFill = FromHex("#2a303a");
    public static readonly Color ControlHover = FromHex("#353c48");
    public static readonly Color ControlPressed = FromHex("#20252d");
    public static readonly Color ControlBorder = FromHex("#3a4250");

    // --- Text ---

    public static readonly Color Fg = FromHex("#e8eaee");
    public static readonly Color FgSoft = FromHex("#a6adb8");
    public static readonly Color FgMuted = FromHex("#6c7482");
    public static readonly Color FgOnAccent = FromHex("#17130a");

    // --- Meaning ---

    public static readonly Color Accent = FromHex("#f1b44c");
    public static readonly Color Ready = FromHex("#6fcf8f");
    public static readonly Color Danger = FromHex("#e5534b");
    public static readonly Color Hover = FromHex("#8ec9ff");

    /// <summary>A card named in a line of text - the game log, a draft's picks - that can be pointed at to read it.</summary>
    public static readonly Color CardName = FromHex("#9fd0ff");

    /// <summary>This player's turn.</summary>
    public static readonly Color Mine = FromHex("#5fae75");

    /// <summary>Somebody else's turn.</summary>
    public static readonly Color Theirs = FromHex("#c9834f");

    /// <summary>
    /// One brush for each colour, shared by everything painted in it. A brush is never changed once made, and a control
    /// given the brush it already has is not changed either: a new brush on every redraw made every control it was set
    /// on look changed, and be laid out and drawn again, when it was not.
    /// </summary>
    public static SolidColorBrush Brush(Color color)
    {
        if (!Brushes.TryGetValue(color, out SolidColorBrush? brush))
        {
            brush = new SolidColorBrush(color);
            Brushes[color] = brush;
        }

        return brush;
    }

    private static readonly Dictionary<Color, SolidColorBrush> Brushes = [];

    /// <summary>
    /// Rounds a border's ends into a pill: straight sides, semicircular ends. A radius of 999, which a browser clamps into
    /// that shape, Uno clamps against the width and the height separately, and the corners come out as ellipses - an oval.
    /// So the radius follows the border's own height instead.
    /// </summary>
    public static T Pill<T>(T border)
        where T : Border
    {
        ArgumentNullException.ThrowIfNull(border);
        border.SizeChanged += (_, e) => border.CornerRadius = new CornerRadius(e.NewSize.Height / 2);
        return border;
    }

    public static Color WithAlpha(Color color, double alpha) =>
        Color.FromArgb((byte)Math.Round(alpha * 255), color.R, color.G, color.B);

    public static Color FromHex(string hex)
    {
        hex = hex.TrimStart('#');
        byte r = Convert.ToByte(hex[..2], 16);
        byte g = Convert.ToByte(hex[2..4], 16);
        byte b = Convert.ToByte(hex[4..6], 16);
        return Color.FromArgb(255, r, g, b);
    }
}
