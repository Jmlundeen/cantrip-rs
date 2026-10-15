using Cantrip.Client;
using Cantrip.Protocol;
using Feeling = Cantrip.Client.Web.ResolutionEffectsElement.Feeling;
using Mark = Cantrip.Client.Web.ResolutionEffectsElement.Mark;

namespace Cantrip.Client.Web;

/// <summary>
/// Shows each ability as it resolves (<see cref="ResolutionEffectsElement"/>): its source lit, and an arrow to each other
/// card or player it touched, coloured by what it did to it. What it did is read from the events it made - the ones the
/// server says ran from its start to its end (<see cref="AbilityResolvedEvent.First"/>) - so nothing here knows a card.
/// </summary>
public sealed partial class MainPage
{
    private readonly ResolutionEffectsElement _resolutions = new();

    /// <summary>The latest events with their log sequences, as far back as an ability that asked a question may reach.</summary>
    private readonly List<(long Sequence, GameEventView Event)> _resolutionWindow = [];

    private const int ResolutionWindow = 400;

    /// <summary>Where each card was last drawn on the effects layer, so one gone by the time its effect is shown is found there.</summary>
    private readonly Dictionary<ViewHandle, ResolutionEffectsElement.Spot> _lastPlaces = [];

    private void BuildResolutions()
    {
        _resolutions.Locate = Locate;
        _resolutions.LastSeen = mark => _lastPlaces.TryGetValue(mark.Card, out ResolutionEffectsElement.Spot spot) ? spot : null;
        Grid.SetRowSpan(_resolutions, 3);
        Grid.SetColumnSpan(_resolutions, 2);
        Root.Children.Add(_resolutions);
    }

    /// <summary>
    /// Takes in the latest batch, and shows each ability it says resolved. Not while the page is being brought up to date
    /// (<paramref name="live"/> false): a reload's whole log again is what happened, not what is happening.
    /// </summary>
    private void FollowResolutions(GameClient client, bool live)
    {
        foreach (GameEventView entry in client.RecentEvents)
        {
            long sequence = client.SequenceOf(entry);
            _resolutionWindow.Add((sequence, entry));

            if (live && entry is AbilityResolvedEvent resolved && Resolution(client, resolved, sequence) is { } effect)
            {
                _resolutions.Play(effect);

                // The stack's arrow is the ability's aim; once it resolves, its own arrows say what it did.
                _arrows.Show([]);
                _arrowLabels.Children.Clear();
            }
        }

        if (_resolutionWindow.Count > ResolutionWindow)
        {
            _resolutionWindow.RemoveRange(0, _resolutionWindow.Count - ResolutionWindow);
        }
    }

    /// <summary>What an ability did to each thing it touched, as an effect to show, or null when there is nowhere to show it.</summary>
    private ResolutionEffectsElement.Effect? Resolution(GameClient client, AbilityResolvedEvent resolved, long sequence)
    {
        Dictionary<Mark, List<Feeling>> touched = [];
        List<Mark> order = [];
        HashSet<int> givenTokens = [];

        void Touch(Mark mark, Feeling? feeling)
        {
            if (!touched.TryGetValue(mark, out List<Feeling>? feelings))
            {
                touched[mark] = feelings = [];
                order.Add(mark);
            }

            if (feeling is { } felt)
            {
                feelings.Add(felt);
            }
        }

        foreach ((long at, GameEventView entry) in _resolutionWindow)
        {
            if (at < resolved.First || at >= sequence)
            {
                continue;
            }

            switch (entry)
            {
                case LifeChangedEvent life when life.Delta != 0:
                    Touch(Mark.OfPlayer(life.Player), life.Delta > 0 ? Feeling.Good : Feeling.Bad);
                    break;

                case DamageMarkedEvent damage when !damage.Object.IsNone && damage.Amount > 0:
                    Touch(Mark.Of(damage.Object), Feeling.Bad);
                    break;

                case CountersPutEvent counters when !counters.Object.IsNone:
                    Touch(Mark.Of(counters.Object), counters.Name switch
                    {
                        "+1/+1" => Feeling.Good,
                        "-1/-1" => Feeling.Bad,
                        _ => Feeling.Neutral,
                    });
                    break;

                case TapChangedEvent tap when !tap.Object.IsNone:
                    Touch(Mark.Of(tap.Object), Feeling.Neutral);
                    break;

                case TokenCreatedEvent token when !token.Object.IsNone:
                    Touch(Mark.Of(token.Object), Feeling.Good);
                    givenTokens.Add(token.Controller);
                    break;

                // A permanent taken off the battlefield for good is the plainest harm there is; one bounced or flickered
                // is only moved.
                case ObjectMovedEvent { From: ZoneKindView.Battlefield } moved when !moved.Object.IsNone:
                    Touch(Mark.Of(moved.Object), moved.Reason is ZoneChangeReasonView.Destroyed or ZoneChangeReasonView.Exiled or ZoneChangeReasonView.Sacrificed
                        ? Feeling.Bad
                        : Feeling.Neutral);
                    break;

                case ObjectMovedEvent { To: ZoneKindView.Battlefield } entered when !entered.Object.IsNone:
                    Touch(Mark.Of(entered.Object), Feeling.Neutral);
                    break;

                default:
                    break;
            }
        }

        foreach (PowerToughnessChangeView changed in resolved.PowerToughness)
        {
            Touch(Mark.Of(changed.Object), (changed.Power, changed.Toughness) switch
            {
                ( >= 0, >= 0) => Feeling.Good,
                ( <= 0, <= 0) => Feeling.Bad,
                _ => Feeling.Neutral,
            });
        }

        // A target is touched even when nothing it was given makes an event: it gained flying, or was told to attack. A
        // player whose part was only to be given tokens is pointed at through them: the arrows go to the tokens.
        foreach (TargetMessage target in resolved.Targets)
        {
            if (target.IsPlayer && givenTokens.Contains(target.Player) && !touched.ContainsKey(Mark.OfPlayer(target.Player)))
            {
                continue;
            }

            Touch(target.IsPlayer ? Mark.OfPlayer(target.Player) : Mark.Of(target.Object), null);
        }

        Mark source = Mark.Of(resolved.Source);
        List<(Mark To, Feeling Feeling)> arrows = [];
        List<Feeling> overall = [];

        foreach (Mark mark in order)
        {
            Feeling feeling = Sum(touched[mark]);
            overall.Add(feeling);

            if (mark != source)
            {
                arrows.Add((mark, feeling));
            }
        }

        // The source lit where it is on the table; one gone from it - the creature whose dying it was - has its arrows
        // set off from where the ability itself was on the stack.
        if (!resolved.Source.IsNone && Locate(source) is not null)
        {
            return new ResolutionEffectsElement.Effect(source, Sum(overall), arrows);
        }

        Mark ability = Mark.Of(resolved.Ability);
        return arrows.Count > 0 && !resolved.Ability.IsNone && (Locate(ability) is not null || _lastPlaces.ContainsKey(resolved.Ability))
            ? new ResolutionEffectsElement.Effect(source, Sum(overall), arrows, ability)
            : null;
    }

    /// <summary>Good only when everything was, bad only when everything was, and otherwise neither.</summary>
    private static Feeling Sum(List<Feeling> feelings) =>
        feelings.Count > 0 && feelings.All(feeling => feeling == Feeling.Good) ? Feeling.Good
        : feelings.Count > 0 && feelings.All(feeling => feeling == Feeling.Bad) ? Feeling.Bad
        : Feeling.Neutral;

    /// <summary>Where a card or a player is on the effects layer now: a card on any row, or the box that stands for a player.</summary>
    private ResolutionEffectsElement.Spot? Locate(Mark mark)
    {
        if (_gameClient is not { } client)
        {
            return null;
        }

        if (mark.IsPlayer)
        {
            PlayerRail? rail = mark.Player == client.Seat ? _myHalf.Rail
                : client.Opponents.FirstOrDefault()?.Seat == mark.Player ? _opponentHalf.Rail
                : null;

            return rail is not null && BoundsOf(rail.Avatar, new Windows.Foundation.Rect(0, 0, rail.Avatar.ActualWidth, rail.Avatar.ActualHeight)) is { } box
                ? new ResolutionEffectsElement.Spot(box, 0f)
                : null;
        }

        if (mark.Card.IsNone)
        {
            return null;
        }

        foreach (CardBoardElement board in PlacedBoards())
        {
            if (board.Visibility == Visibility.Visible && board.Placement(mark.Card) is { } placed && BoundsOf(board, placed.Face) is { } face)
            {
                return new ResolutionEffectsElement.Spot(face, placed.Angle);
            }
        }

        return null;
    }

    private CardBoardElement[] PlacedBoards() => [_opponentHalf.Board, _myHalf.Board, _opponentHalf.Graveyard.Board, _myHalf.Graveyard.Board, _handBoard, _stackBoard];

    /// <summary>Remembers where every card is drawn now, after a layout pass.</summary>
    private void RememberPlaces()
    {
        foreach (CardBoardElement board in PlacedBoards())
        {
            if (board.XamlRoot is null || board.ActualWidth <= 0 || !IsShown(board))
            {
                continue;
            }

            Microsoft.UI.Xaml.Media.GeneralTransform toLayer = board.TransformToVisual(_resolutions);
            foreach ((ViewHandle handle, ObjectView _, float _, float _, float _, float _) in board.Laid())
            {
                if (board.Placement(handle) is { } placed)
                {
                    Windows.Foundation.Rect r = toLayer.TransformBounds(placed.Face);
                    _lastPlaces[handle] = new ResolutionEffectsElement.Spot(SkiaSharp.SKRect.Create((float)r.X, (float)r.Y, (float)r.Width, (float)r.Height), placed.Angle);
                }
            }
        }

        // A card gone from the table a while is not looked for again.
        if (_lastPlaces.Count > 600)
        {
            _lastPlaces.Clear();
        }
    }
}
