using Cantrip.Protocol;
using MessagePack;
using Microsoft.UI.Text;

namespace Cantrip.Client.Web;

/// <summary>
/// A moment at the start of a game, over the whole window, that gets ready what every face of this player's deck is
/// drawn with: the typefaces and template, and each frame the deck's cards wear, decoded from the page's own files. A
/// deck whose frames are all decoded already - a reconnect, a reload into the same game - is not shown at all.
/// <para>
/// The cards themselves are left to the table. A face in view is drawn when it is first wanted; the rest of the deck's
/// faces are drawn as frames have time to spare, once their art is in (<see cref="Faces.Urgency.Ahead"/>). The deck's
/// art is asked for a couple of cards at a time, so a card on screen asking for its own is never queued behind the
/// whole library; then again at the best size the Card art quality setting allows, for the card held up to read.
/// </para>
/// <para>
/// Every other card, first seen during the game - an opponent's, a card revealed - is sent as printed too
/// (<see cref="GameClient.PrintedSeen"/>): its art is asked for at once, since it is likely on screen, and its face is
/// drawn the same way as frames have time.
/// </para>
/// </summary>
public sealed class DeckLoadingPanel : Grid
{
    /// <summary>
    /// How long each frame may spend decoding, past the first. Decoded once a frame, from the frame itself, so the page
    /// paints between steps and the progress shows.
    /// </summary>
    private static readonly TimeSpan FrameBudget = TimeSpan.FromMilliseconds(12);

    private readonly TextBlock _progress;
    private readonly Border _bar;
    private readonly Border _track;
    private bool _stepping;

    /// <summary>The frames the deck wears that are still to decode.</summary>
    private List<string> _frames = [];
    private IReadOnlyList<DeckCardView>? _deck;

    /// <summary>The faces of cards seen, the deck's among them, drawn as frames have time for them.</summary>
    private readonly List<(ObjectView Card, string Key, string Look, string Slug)> _seen = [];

    /// <summary>The client whose newly seen cards are listened for.</summary>
    private GameClient? _client;
    private int _decoded;
    private DateTime _started;

    public DeckLoadingPanel()
    {
        HorizontalAlignment = HorizontalAlignment.Stretch;
        VerticalAlignment = VerticalAlignment.Stretch;
        Background = Theme.Brush(Theme.Bar);
        Visibility = Visibility.Collapsed;

        _progress = new TextBlock { FontSize = 13, Foreground = Theme.Brush(Theme.FgMuted), HorizontalAlignment = HorizontalAlignment.Center };
        _track = new Border { Height = 4, Width = 320, CornerRadius = new CornerRadius(2), Background = Theme.Brush(Theme.Line) };
        _bar = new Border { Height = 4, Width = 0, CornerRadius = new CornerRadius(2), Background = Theme.Brush(Theme.Fg), HorizontalAlignment = HorizontalAlignment.Left };

        Children.Add(new StackPanel
        {
            Spacing = 12,
            HorizontalAlignment = HorizontalAlignment.Center,
            VerticalAlignment = VerticalAlignment.Center,
            Children =
            {
                new TextBlock
                {
                    Text = "Getting your deck ready",
                    FontSize = 20,
                    FontWeight = FontWeights.SemiBold,
                    Foreground = Theme.Brush(Theme.Fg),
                    HorizontalAlignment = HorizontalAlignment.Center,
                },
                new Grid { Children = { _track, _bar } },
                _progress,
            },
        });

    }

    /// <summary>Steps once a frame while there is something to decode or draw.</summary>
    private void Stepping(bool on)
    {
        if (on != _stepping)
        {
            _stepping = on;
            if (on)
            {
                Microsoft.UI.Xaml.Media.CompositionTarget.Rendering += OnRendering;
            }
            else
            {
                Microsoft.UI.Xaml.Media.CompositionTarget.Rendering -= OnRendering;
            }
        }
    }

    private void OnRendering(object? sender, object e) => Step();

    /// <summary>Starts getting a deck ready the first time the client has it; a deck already started, or already ready, is left.</summary>
    public void Show(GameClient client)
    {
        IReadOnlyList<DeckCardView> deck = client.Decklist;
        if (deck.Count > 0 && !ReferenceEquals(deck, _deck))
        {
            _deck = deck;

            // A double-faced card wears its other face's frame as well, and a preparation card its prepare spell's.
            _frames = [.. deck
                .SelectMany(entry => entry.Card.AlsoPrinted().Prepend(entry.Card))
                .Where(card => card.Known && !card.FaceDown)
                .SelectMany(CardFace.FramesOf)
                .Distinct(StringComparer.Ordinal)
                .Where(frame => !CardAssets.HasFrame(frame))];

            if (_frames.Count > 0)
            {
                _decoded = 0;
                _started = DateTime.UtcNow;
                Visibility = Visibility.Visible;
                ShowProgress();
                Stepping(true);
            }
            else
            {
                _ = FetchDeckArt(deck);
            }
        }

        Follow(client);
    }

    /// <summary>
    /// Listens for the cards a client is sent as printed: each is noted as the card before the game touched it, and has
    /// its face made. The ones sent before this client was followed are taken as sent.
    /// </summary>
    private void Follow(GameClient client)
    {
        if (ReferenceEquals(client, _client))
        {
            return;
        }

        if (_client is not null)
        {
            _client.PrintedSeen -= Seen;
        }

        _client = client;
        client.PrintedSeen += Seen;
        foreach (ObjectView card in client.Printed.Values)
        {
            Seen(card);
        }
    }

    /// <summary>A card sent as printed: noted as printed, and its face made unless it is already, or is waiting to be.</summary>
    private void Seen(ObjectView card)
    {
        Faces.NotePrinted(card);

        // The client has its deck before it says the deck's cards are seen, which is at the welcome.
        bool ours = _client?.Decklist.Any(entry => entry.Card.CardId == card.CardId) == true;
        foreach (var face in Looks(card).Where(face => FaceCache.Peek(face.Key) is null && _seen.All(waiting => waiting.Key != face.Key)))
        {
            _seen.Add(face);
            if (!ours)
            {
                foreach (string name in CardFace.ArtNames(face.Card))
                {
                    _ = CardArt.Get(name, ArtPixelsOf(face.Card));
                }
            }
        }

        if (_seen.Count > 0)
        {
            Stepping(true);
        }
    }

    /// <summary>The art size a face at the smallest size asks for.</summary>
    private static int ArtPixels => CardFace.ArtPixels(Faces.MinWidth, Faces.PixelScale);

    /// <summary>The art size a card's face at the smallest size asks for: a Saga's is larger, for its taller art box.</summary>
    private static int ArtPixelsOf(ObjectView card) => CardFace.ArtPixels(card, Faces.MinWidth, Faces.PixelScale);

    /// <summary>
    /// The faces a card will be seen with before anything happens to it: as it reads in the hand or the library, and for
    /// a creature, summoning sick as it reads on the battlefield the turn it arrives.
    /// </summary>
    private static IEnumerable<(ObjectView Card, string Key, string Look, string Slug)> Looks(ObjectView card)
    {
        if (!card.Known || card.FaceDown)
        {
            yield break;
        }

        yield return Face(card);

        if (card.CardTypes.Contains(CardTypeView.Creature))
        {
            // A copy, since the client's printed card is its to keep as it was sent.
            ObjectView arriving = MessagePackSerializer.Deserialize<ObjectView>(MessagePackSerializer.Serialize(card));
            arriving.SummoningSick = true;
            yield return Face(arriving);
        }
    }

    private static (ObjectView Card, string Key, string Look, string Slug) Face(ObjectView card)
    {
        string look = Faces.Look(Faces.LookOf(card), dimmed: false, annotation: null);
        return (card, Faces.Key(Faces.MinWidth, Faces.MinHeight, look), look, CardFace.ArtSlug(card));
    }

    /// <summary>
    /// Draws the faces of cards seen whose art has come, as many as the frame allows faces out of view - never ahead of
    /// one in view.
    /// </summary>
    private void StepSeen()
    {
        for (int i = 0; i < _seen.Count; i++)
        {
            (ObjectView card, string key, string look, string slug) = _seen[i];
            if (FaceCache.Peek(key) is not null)
            {
                _seen.RemoveAt(i--);
                continue;
            }

            if (!CardFace.ArtNames(card).All(name => CardArt.Settled(name, ArtPixelsOf(card))))
            {
                continue;
            }

            if (!Faces.MayDraw(Faces.Urgency.Ahead))
            {
                return;
            }

            Faces.Draw(card, Faces.MinWidth, Faces.MinHeight, dimmed: false, annotation: null, key, look, slug,
                Faces.IsDurable(card, Faces.LookOf(card), dimmed: false, annotation: null));
            _seen.RemoveAt(i--);
        }

        // Only the ones waiting on their art are left: art arriving draws a row again, and the next card seen starts this.
        if (_seen.All(face => !CardFace.ArtNames(face.Card).All(name => CardArt.Settled(name, ArtPixelsOf(face.Card)))))
        {
            Stepping(false);
        }
    }

    /// <summary>Decodes the deck's frames for most of a frame, or, with them done, draws faces ahead.</summary>
    private void Step()
    {
        if (Visibility != Visibility.Visible)
        {
            StepSeen();
            return;
        }

        // One a frame at least, and more while the frame has time.
        DateTime started = DateTime.UtcNow;
        bool decodedOne = false;
        while (_frames.Count > 0 && (!decodedOne || DateTime.UtcNow - started < FrameBudget))
        {
            CardAssets.PrepareFrame(_frames[0]);
            _frames.RemoveAt(0);
            _decoded++;
            decodedOne = true;
        }

        ShowProgress();

        if (_frames.Count == 0)
        {
            Visibility = Visibility.Collapsed;
            FacePerf.DeckReady(_decoded, DateTime.UtcNow - _started);
            _ = FetchDeckArt(_deck ?? []);
        }
    }

    /// <summary>How many cards' art is asked for at once in the background.</summary>
    private const int ArtAtOnce = 2;

    /// <summary>
    /// A pause after each picture arrives. It is decoded on the page's thread as it lands, so spacing them keeps a deck's
    /// worth from landing in one stall.
    /// </summary>
    private static readonly TimeSpan ArtSpacing = TimeSpan.FromMilliseconds(100);

    /// <summary>
    /// Asks for every deck card's art, a few at a time, in deck order: at the size its faces are drawn at, then at the best
    /// size the setting allows. What else is printed on a card - a double-faced card's other face, a prepare spell - comes
    /// with it, for a close look to show beside it. A card already asked for - one on screen - is passed over.
    /// </summary>
    private static async Task FetchDeckArt(IReadOnlyList<DeckCardView> deck)
    {
        string[] names = [.. deck
            .Where(entry => entry.Card.Known)
            .SelectMany(entry => entry.Card.AlsoPrinted().Prepend(entry.Card).SelectMany(CardFace.ArtNames))
            .Distinct(StringComparer.Ordinal)];
        var waiting = new Queue<(string Name, int Pixels)>([
            .. names.Select(name => (name, ArtPixels)),
            .. names.Select(name => (name, CardArt.Best)),
        ]);

        async Task Worker()
        {
            while (waiting.TryDequeue(out var next))
            {
                Task fetching = CardArt.Fetch(next.Name, next.Pixels);
                if (!fetching.IsCompleted)
                {
                    await fetching;
                    await Task.Delay(ArtSpacing);
                }
            }
        }

        await Task.WhenAll(Enumerable.Range(0, ArtAtOnce).Select(_ => Worker()));
    }

    private void ShowProgress()
    {
        int all = _decoded + _frames.Count;
        _progress.Text = $"{_decoded} of {all}";
        _bar.Width = all == 0 ? 0 : _track.Width * _decoded / all;
    }
}
