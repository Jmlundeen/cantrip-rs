using Windows.System;

namespace Cantrip.Client.Web;

/// <summary>The things a key can be bound to. The order here is the order the settings list them in.</summary>
public enum Hotkey
{
    Pass,
    Undo,

    /// <summary>Keeps priority after a spell goes on the stack, for as long as the key is held.</summary>
    HoldPriority,

    /// <summary>Ticks or clears the dock's Pass Turn box.</summary>
    PassTurn,

    /// <summary>Ticks or clears the dock's Auto Pass box.</summary>
    AutoPass,

    /// <summary>While held, a land with more than one mana ability is tapped for whichever the cost most needs.</summary>
    AutoMana,

    /// <summary>Sends every creature that can attack at the opponent, while none are declared yet.</summary>
    AttackAll,

    /// <summary>Presses the dock's Cancel button.</summary>
    Cancel,
}

/// <summary>
/// A key as a binding: the key itself and whichever modifiers are held with it, so Ctrl+Z and Z are different bindings.
/// A modifier on its own is a binding too, for the hotkeys that act while held (Ctrl holds priority).
/// </summary>
public readonly record struct KeyBinding(VirtualKey Key, bool Ctrl = false, bool Shift = false, bool Alt = false)
{
    /// <summary>A binding as the player reads it: "Space", "Z", "Ctrl+Shift+P".</summary>
    public override string ToString()
    {
        List<string> parts = [];
        if (Ctrl && !IsModifier(Key)) parts.Add("Ctrl");
        if (Shift && !IsModifier(Key)) parts.Add("Shift");
        if (Alt && !IsModifier(Key)) parts.Add("Alt");
        parts.Add(Name(Key));
        return string.Join('+', parts);
    }

    /// <summary>Written to storage and read back.</summary>
    public string Serialize() => $"{(int)Key}:{(Ctrl ? 1 : 0)}{(Shift ? 1 : 0)}{(Alt ? 1 : 0)}";

    public static KeyBinding? Parse(string? saved)
    {
        if (saved is not { Length: > 4 } || saved.IndexOf(':') is not (> 0 and var colon) || saved.Length != colon + 4
            || !int.TryParse(saved[..colon], out int key))
        {
            return null;
        }

        return new KeyBinding((VirtualKey)key, saved[colon + 1] == '1', saved[colon + 2] == '1', saved[colon + 3] == '1');
    }

    public static bool IsModifier(VirtualKey key) => key is VirtualKey.Control or VirtualKey.Shift or VirtualKey.Menu
        or VirtualKey.LeftControl or VirtualKey.RightControl or VirtualKey.LeftShift or VirtualKey.RightShift
        or VirtualKey.LeftMenu or VirtualKey.RightMenu or VirtualKey.LeftWindows or VirtualKey.RightWindows;

    /// <summary>A modifier named as the key it is, whichever side of the keyboard it was pressed on.</summary>
    public static VirtualKey Plain(VirtualKey key) => key switch
    {
        VirtualKey.LeftControl or VirtualKey.RightControl => VirtualKey.Control,
        VirtualKey.LeftShift or VirtualKey.RightShift => VirtualKey.Shift,
        VirtualKey.LeftMenu or VirtualKey.RightMenu => VirtualKey.Menu,
        _ => key,
    };

    private static string Name(VirtualKey key) => key switch
    {
        VirtualKey.Control => "Ctrl",
        VirtualKey.Shift => "Shift",
        VirtualKey.Menu => "Alt",
        VirtualKey.Space => "Space",
        VirtualKey.Escape => "Escape",
        VirtualKey.Enter => "Enter",
        VirtualKey.Tab => "Tab",
        VirtualKey.Back => "BackSpace",
        >= VirtualKey.Number0 and <= VirtualKey.Number9 => ((char)('0' + (key - VirtualKey.Number0))).ToString(),
        >= VirtualKey.NumberPad0 and <= VirtualKey.NumberPad9 => $"Kp {(char)('0' + (key - VirtualKey.NumberPad0))}",
        >= VirtualKey.A and <= VirtualKey.Z => ((char)('A' + (key - VirtualKey.A))).ToString(),
        >= VirtualKey.F1 and <= VirtualKey.F12 => $"F{key - VirtualKey.F1 + 1}",
        _ => key.ToString(),
    };
}

/// <summary>What each hotkey is called, what it does, and the key it starts on.</summary>
public static class Hotkeys
{
    /// <summary>Shows and hides the side panel. Not rebindable, and so reserved.</summary>
    public static readonly KeyBinding TogglePanel = new(VirtualKey.L);

    /// <summary>Stops choosing a key in the settings; it can still be chosen for Cancel, whose default it is.</summary>
    public const VirtualKey StopChoosing = VirtualKey.Escape;

    public static IReadOnlyList<Hotkey> All { get; } = Enum.GetValues<Hotkey>();

    public static KeyBinding Default(Hotkey hotkey) => hotkey switch
    {
        Hotkey.Pass => new(VirtualKey.Space),
        Hotkey.Undo => new(VirtualKey.Z),
        Hotkey.HoldPriority => new(VirtualKey.Control),
        Hotkey.PassTurn => new(VirtualKey.Number6),
        Hotkey.AutoPass => new(VirtualKey.Number8),
        Hotkey.AutoMana => new(VirtualKey.Shift),
        Hotkey.AttackAll => new(VirtualKey.A),
        Hotkey.Cancel => new(VirtualKey.Escape),
        _ => throw new ArgumentOutOfRangeException(nameof(hotkey), hotkey, null),
    };

    public static string Title(Hotkey hotkey) => hotkey switch
    {
        Hotkey.Pass => "Pass",
        Hotkey.Undo => "Undo",
        Hotkey.HoldPriority => "Hold priority",
        Hotkey.PassTurn => "Pass turn",
        Hotkey.AutoPass => "Auto pass",
        Hotkey.AutoMana => "Auto-mana",
        Hotkey.AttackAll => "Attack with all",
        Hotkey.Cancel => "Cancel",
        _ => throw new ArgumentOutOfRangeException(nameof(hotkey), hotkey, null),
    };

    public static string Description(Hotkey hotkey) => hotkey switch
    {
        Hotkey.Pass =>
            "Press the dock's main button: pass priority, lock in attackers, blockers, or a division of combat damage, "
            + "or put your simultaneous triggers on the stack in the order shown.",
        Hotkey.AttackAll => "While declaring attackers with none declared yet, send every creature that can attack.",
        Hotkey.Undo => "Take back the last mana ability.",
        Hotkey.Cancel => "Press the dock's Cancel button: back out of a spell or ability being cast, or of choosing a target.",
        Hotkey.PassTurn =>
            "Tick or clear the dock's Pass Turn box, which passes priority at every chance, stops included, until " +
            "the turn ends. You are still asked to declare blockers.",
        Hotkey.AutoPass =>
            "Tick or clear the dock's Auto Pass box, which passes priority whenever there is nothing you could do " +
            "but make mana. The steps you stop in still stop you.",
        Hotkey.AutoMana =>
            "While held, a land that could be tapped for more than one colour is tapped for whichever colour the " +
            "cost needs and the rest of your board is least able to supply, instead of asking. With Auto-mana on " +
            "below, this is the other way round: hold it to be asked.",
        Hotkey.HoldPriority =>
            "While held, keep priority for as long as a spell or ability of yours is on top of the stack, so you can " +
            "act again before your opponent may respond. Otherwise those windows are passed for you, and you are asked " +
            "again the moment your opponent puts something on top. The Hold priority box in the dock keeps it held.",
        _ => throw new ArgumentOutOfRangeException(nameof(hotkey), hotkey, null),
    };

    /// <summary>Whether a hotkey acts for as long as its key is held rather than when it is pressed; such a key may be a bare modifier.</summary>
    public static bool IsHeld(Hotkey hotkey) => hotkey is Hotkey.HoldPriority or Hotkey.AutoMana;

    /// <summary>The bracketed key a tooltip ends with.</summary>
    public static string Suffix(KeyBinding key) => $"  [{key}]";

    /// <summary>Why a key cannot be bound to anything, or null when it can.</summary>
    public static string? WhyReserved(KeyBinding key) =>
        key == TogglePanel ? $"{TogglePanel} already shows and hides this panel." : null;
}
