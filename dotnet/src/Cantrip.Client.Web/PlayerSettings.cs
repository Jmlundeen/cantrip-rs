namespace Cantrip.Client.Web;

/// <summary>How large card illustrations are fetched: the longest side kept.</summary>
public enum ArtQuality
{
    Low,
    Medium,
    High,
}

/// <summary>
/// The player's preferences: saved the moment they change, per browser. A preference is never worth refusing to start over, so anything unreadable is the default.
/// </summary>
public static class PlayerSettings
{
    public const float MinCardScale = 0.5f;
    public const float MaxCardScale = 4f;

    private const string Prefix = "cantrip-setting-";

    private static readonly Dictionary<Hotkey, KeyBinding> Bindings = Load();

    /// <summary>Raised after any preference changes, for whatever draws from it.</summary>
    public static event Action? Changed;

    /// <summary>Cards turn as they tap and grow as they are held up to read. On by default.</summary>
    public static bool AnimateCards
    {
        get => LocalSettings.Flag(Prefix + "animate", true);
        set => Set("animate", value);
    }

    /// <summary>A dual land is tapped for the colour the cost needs instead of asking. On by default.</summary>
    public static bool AutoManaByDefault
    {
        get => LocalSettings.Flag(Prefix + "auto-mana", true);
        set => Set("auto-mana", value);
    }

    /// <summary>Permanents drawn as large as the two rows of a half have room for. On by default.</summary>
    public static bool AutoCardScale
    {
        get => LocalSettings.Flag(Prefix + "auto-scale", true);
        set => Set("auto-scale", value);
    }

    /// <summary>The battlefield card size while it is the player's to choose, as a multiple of the standard size.</summary>
    public static float CardScale
    {
        get => Math.Clamp(LocalSettings.Number(Prefix + "scale", 1f), MinCardScale, MaxCardScale);
        set
        {
            LocalSettings.Store(Prefix + "scale", Math.Clamp(value, MinCardScale, MaxCardScale));
            Changed?.Invoke();
        }
    }

    private static ArtQuality? _artQuality;

    /// <summary>The art quality changed: every face drawn is drawn with art of the wrong size for it now.</summary>
    public static event Action? ArtQualityChanged;

    /// <summary>How large illustrations are fetched. The sharpest by default.</summary>
    public static ArtQuality ArtQuality
    {
        // Read once and kept: art sizes are worked out for every face drawn, and each read of storage crosses into the page.
        get => _artQuality ??= Enum.TryParse(LocalSettings.Text(Prefix + "art-quality"), out ArtQuality saved) ? saved : ArtQuality.High;
        set
        {
            _artQuality = value;
            LocalSettings.Store(Prefix + "art-quality", value.ToString());
            ArtQualityChanged?.Invoke();
            Changed?.Invoke();
        }
    }

    /// <summary>Whether the side panel is open.</summary>
    public static bool PanelOpen
    {
        get => LocalSettings.Flag(Prefix + "panel-open", true);
        set => Set("panel-open", value);
    }

    /// <summary>The graveyard dock's width before the player has dragged it, and the narrowest and widest it may be.</summary>
    public const float DefaultGraveyardWidth = 150f;
    public const float MinGraveyardWidth = 90f;
    public const float MaxGraveyardWidth = 420f;

    /// <summary>
    /// How wide the graveyard dock is, as its edge was last dragged. Kept to its own
    /// bounds here; the table it is next shown beside keeps it to what that table leaves.
    /// </summary>
    public static float GraveyardWidth
    {
        get => Math.Clamp(LocalSettings.Number(Prefix + "graveyard-width", DefaultGraveyardWidth), MinGraveyardWidth, MaxGraveyardWidth);
        set => LocalSettings.Store(Prefix + "graveyard-width", Math.Clamp(value, MinGraveyardWidth, MaxGraveyardWidth));
    }

    /// <summary>The longest side an illustration is fetched at for a quality (CardArt.Pixels).</summary>
    public static int Pixels(ArtQuality quality) => quality switch
    {
        ArtQuality.Low => 128,
        ArtQuality.Medium => 256,
        _ => 512,
    };

    /// <summary>The name and the size together, so the slider says what it costs as well as what it is.</summary>
    public static string Describe(ArtQuality quality) => $"{quality} ({Pixels(quality)} px)";

    public static KeyBinding KeyFor(Hotkey hotkey) => Bindings[hotkey];

    /// <summary>
    /// Binds a hotkey to a key. A key already bound to another hotkey is swapped rather than stolen, so a rebinding never
    /// leaves anything without a key the player did not choose to lose.
    /// </summary>
    /// <returns>The hotkey that was swapped, if any.</returns>
    public static Hotkey? Bind(Hotkey hotkey, KeyBinding key)
    {
        KeyBinding old = Bindings[hotkey];
        if (old == key)
        {
            return null;
        }

        Hotkey? swapped = null;
        foreach (Hotkey other in Hotkeys.All)
        {
            if (other != hotkey && Bindings[other] == key)
            {
                Bindings[other] = old;
                swapped = other;
            }
        }

        Bindings[hotkey] = key;
        SaveBindings();
        return swapped;
    }

    public static bool HotkeysAreDefault => Hotkeys.All.All(hotkey => Bindings[hotkey] == Hotkeys.Default(hotkey));

    public static void ResetHotkeys()
    {
        foreach (Hotkey hotkey in Hotkeys.All)
        {
            Bindings[hotkey] = Hotkeys.Default(hotkey);
        }

        SaveBindings();
    }

    private static void Set(string name, bool value)
    {
        LocalSettings.Store(Prefix + name, value);
        Changed?.Invoke();
    }

    private static Dictionary<Hotkey, KeyBinding> Load() =>
        Hotkeys.All.ToDictionary(
            hotkey => hotkey,
            hotkey => KeyBinding.Parse(LocalSettings.Text($"{Prefix}hotkey-{hotkey}")) is { } saved && Hotkeys.WhyReserved(saved) is null
                ? saved
                : Hotkeys.Default(hotkey));

    private static void SaveBindings()
    {
        foreach ((Hotkey hotkey, KeyBinding key) in Bindings)
        {
            LocalSettings.Store($"{Prefix}hotkey-{hotkey}", key.Serialize());
        }

        Changed?.Invoke();
    }
}
