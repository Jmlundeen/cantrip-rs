using Cantrip.Client;
using Cantrip.Client.Embedded;
using Cantrip.Client.Llm;
using Cantrip.Core;
using Cantrip.Protocol;
using Cantrip.Server.Rules;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// Draws by agreement (docs/rules-exceptions.md, RX-4): a player offers one while they have priority, which passes it; each
/// other player is asked to accept or decline, a player who declined may accept later at any priority of their own, and the
/// game is a draw once every other player still in it has accepted. The offer is withdrawn the moment its player does
/// anything but pass - any answer, but for declaring nothing - and while it stands each of their later priorities runs no clock for its first
/// three seconds. Every automated seat declines. Played on rules that only go round priority (<see cref="Round"/>), and on
/// time the test moves itself.
/// </summary>
public sealed class DrawOfferTests
{
    private static readonly DateTimeOffset Dealt = new(2026, 10, 21, 12, 0, 0, TimeSpan.Zero);

    private static readonly TimeSpan Hour = MatchClocks.DefaultPlayerClock;

    private static PlayerId Seat(int seat) => new(seat);

    private static (Round Rules, GameSession Session) Deal(int players)
    {
        var rules = new Round(players);
        var session = new GameSession(rules);
        session.Start();
        Drain(session);
        return (rules, session);
    }

    /// <summary>Everything each seat has been sent since the last look, by seat.</summary>
    private static Dictionary<int, List<ServerMessage>> Drain(GameSession session) =>
        session.Seats.ToDictionary(seat => seat.Value, seat => session.Drain(seat).ToList());

    private static void Pass(GameSession session, Round rules) =>
        session.Submit(Seat(rules.Asked), new AnswerMessage { RequestId = rules.RequestId, Answer = new PassAnswer() });

    private static GameOutcomeView? Outcome(List<ServerMessage> sent) => sent.OfType<GameOverMessage>().LastOrDefault()?.Outcome;

    [Fact]
    public void In_a_game_of_two_an_offer_passes_priority_and_an_acceptance_draws_the_game()
    {
        (Round rules, GameSession session) = Deal(2);

        session.Submit(Seat(1), new OfferDrawMessage());
        Assert.Equal([(1, nameof(PassAnswer))], rules.Answers);
        Assert.Equal(2, rules.Asked);
        Assert.Equal(1, session.DrawOfferedBy);
        Dictionary<int, List<ServerMessage>> sent = Drain(session);
        Assert.All(sent.Values, messages => Assert.Equal(1, messages.OfType<DrawOfferMessage>().Single().Offerer));
        Assert.Contains(sent[2], message => message is PromptMessage { Prompt: PriorityPromptMessage });

        session.Submit(Seat(2), new AnswerDrawMessage { Accept = true });
        Assert.True(session.IsOver);
        Assert.True(session.DrawnByAgreement);
        sent = Drain(session);
        Assert.All(sent.Values, messages => Assert.Equal(GameOutcomeView.Draw, Outcome(messages)));
        Assert.Empty(session.WinningSeats);
        Assert.Empty(session.WaitingOn);
    }

    [Fact]
    public void In_a_game_of_three_the_game_is_a_draw_only_once_every_other_player_has_accepted()
    {
        (Round rules, GameSession session) = Deal(3);

        session.Submit(Seat(1), new OfferDrawMessage());
        session.Submit(Seat(2), new AnswerDrawMessage { Accept = true });
        Assert.False(session.IsOver);
        DrawOfferMessage told = Drain(session)[3].OfType<DrawOfferMessage>().Last();
        Assert.Equal(1, told.Offerer);
        Assert.Equal([2], told.Accepted);
        Assert.Empty(told.Declined);

        // The offerer cannot accept its own offer, nor offer again.
        session.Submit(Seat(1), new AnswerDrawMessage { Accept = true });
        Assert.Contains(Drain(session)[1], message => message is ErrorMessage);

        session.Submit(Seat(3), new AnswerDrawMessage { Accept = true });
        Assert.True(session.IsOver);
        Assert.All(Drain(session).Values, messages => Assert.Equal(GameOutcomeView.Draw, Outcome(messages)));
        Assert.Equal(nameof(PassAnswer), Assert.Single(rules.Answers).Kind);
    }

    /// <summary>
    /// A draw agreed to ends the rules' game too (<see cref="IRulesHost.End"/>), so what every end of a game reveals (CR 708.9: a
    /// face-down permanent) reaches each seat - its event, and its line in the log, still written as the game ends - before
    /// the one game-over message.
    /// </summary>
    [Fact]
    public void A_draw_agreed_to_tells_every_seat_what_the_end_of_the_game_reveals_before_it_is_over()
    {
        (Round rules, GameSession session) = Deal(2);
        rules.FaceDownAtEnd = new ObjectView { Handle = new ViewHandle(40), FaceDown = true, Known = true, FaceDownName = "Abzan Guide", Zone = ZoneKindView.Battlefield, Owner = 1 };

        session.Submit(Seat(1), new OfferDrawMessage());
        session.Submit(Seat(2), new AnswerDrawMessage { Accept = true });
        Assert.True(session.IsOver);
        Assert.True(session.DrawnByAgreement);
        foreach ((int seat, List<ServerMessage> sent) in Drain(session))
        {
            int revealed = sent.FindIndex(message => message is EventsMessage events && events.Events.OfType<CardsRevealedEvent>().Any());
            int logged = sent.FindIndex(message => message is LogMessage log && log.Entries.Any(entry => entry.Text == $"{(seat == 1 ? "You reveal" : "Seat 1 reveals")} [Abzan Guide]."));
            int over = sent.FindIndex(message => message is GameOverMessage);
            Assert.True(revealed >= 0 && logged > revealed && over > logged, $"seat {seat}: {string.Join(", ", sent.Select(message => message.GetType().Name))}");
            Assert.Single(sent.OfType<GameOverMessage>());
            Assert.Equal(GameOutcomeView.Draw, Outcome(sent));
        }
    }

    /// <summary>A game of three whose last player yet to accept leaves the game (its clock ran out): the rest have agreed.</summary>
    [Fact]
    public void A_draw_is_agreed_when_the_last_player_yet_to_accept_leaves_the_game()
    {
        (Round _, GameSession session) = Deal(3);

        session.Submit(Seat(1), new OfferDrawMessage());
        session.Submit(Seat(2), new AnswerDrawMessage { Accept = true });
        session.Submit(Seat(3), new AnswerDrawMessage { Accept = false });
        Assert.False(session.IsOver);

        session.TimeOut([3]);
        Assert.True(session.IsOver);
        Assert.True(session.DrawnByAgreement);
    }

    [Fact]
    public void A_player_who_declined_accepts_later_only_at_a_priority_of_their_own()
    {
        (Round rules, GameSession session) = Deal(2);

        session.Submit(Seat(1), new OfferDrawMessage());
        session.Submit(Seat(2), new AnswerDrawMessage { Accept = false });
        Assert.False(session.IsOver);
        DrawOfferMessage told = Drain(session)[1].OfType<DrawOfferMessage>().Last();
        Assert.Equal([2], told.Declined);

        // Declined once, it is not declined again.
        session.Submit(Seat(2), new AnswerDrawMessage { Accept = false });
        Assert.Contains(Drain(session)[2], message => message is ErrorMessage);

        // Seat two passes; seat one has priority, so seat two may not accept now.
        Pass(session, rules);
        Assert.Equal(1, rules.Asked);
        session.Submit(Seat(2), new AnswerDrawMessage { Accept = true });
        Assert.False(session.IsOver);
        Assert.Contains(Drain(session)[2], message => message is ErrorMessage { Code: ErrorCode.IllegalAnswer });

        // Seat one passes - the offer stands - and seat two, at its own priority, accepts.
        Pass(session, rules);
        Assert.Equal(1, session.DrawOfferedBy);
        Assert.Equal(2, rules.Asked);
        session.Submit(Seat(2), new AnswerDrawMessage { Accept = true });
        Assert.True(session.IsOver);
        Assert.True(session.DrawnByAgreement);
    }

    [Fact]
    public void The_offer_is_withdrawn_the_moment_its_player_does_anything_with_priority_but_pass()
    {
        (Round rules, GameSession session) = Deal(2);

        session.Submit(Seat(1), new OfferDrawMessage());
        Pass(session, rules);
        Drain(session);

        session.Submit(Seat(1), new AnswerMessage { RequestId = rules.RequestId, Answer = new TakeActionAnswer() });
        Assert.Equal(0, session.DrawOfferedBy);
        Assert.All(Drain(session).Values, messages => Assert.Equal(0, messages.OfType<DrawOfferMessage>().Single().Offerer));

        session.Submit(Seat(2), new AnswerDrawMessage { Accept = true });
        Assert.False(session.IsOver);
        Assert.Contains(Drain(session)[2], message => message is ErrorMessage);
    }

    /// <summary>
    /// Any answer but a pass withdraws the offer, a question the game forces on its player included: seat one, its offer
    /// standing, is asked for a target and names one.
    /// </summary>
    [Fact]
    public void The_offer_is_withdrawn_by_an_answer_to_a_question_other_than_priority()
    {
        (Round rules, GameSession session) = Deal(2);

        session.Submit(Seat(1), new OfferDrawMessage());
        rules.Next = Asking.Target;
        Pass(session, rules);
        Assert.Equal(1, rules.Asked);
        Drain(session);

        session.Submit(Seat(1), new AnswerMessage { RequestId = rules.RequestId, Answer = new TargetsChosenAnswer() });
        Assert.Equal(0, session.DrawOfferedBy);
        Assert.All(Drain(session).Values, messages => Assert.Equal(0, messages.OfType<DrawOfferMessage>().Single().Offerer));
    }

    /// <summary>
    /// A declaration of nothing is a pass: seat one, its offer standing, is asked to declare attackers and declares none, and
    /// its offer still stands; declaring one would have withdrawn it.
    /// </summary>
    [Fact]
    public void An_empty_attack_leaves_the_offer_standing()
    {
        (Round rules, GameSession session) = Deal(2);

        session.Submit(Seat(1), new OfferDrawMessage());
        rules.Next = Asking.Attackers;
        Pass(session, rules);
        Assert.Equal(1, rules.Asked);
        Drain(session);

        session.Submit(Seat(1), new AnswerMessage { RequestId = rules.RequestId, Answer = new AttackersDeclaredAnswer() });
        Assert.Equal(1, session.DrawOfferedBy);
        Assert.DoesNotContain(Drain(session)[2], message => message is DrawOfferMessage);

        session.Submit(Seat(1), new AnswerMessage
        {
            RequestId = rules.RequestId,
            Answer = new AttackersDeclaredAnswer { Attacks = [new AttackMessage()] },
        });
        Assert.Equal(0, session.DrawOfferedBy);
    }

    [Fact]
    public void A_draw_is_offered_only_while_the_player_has_priority_and_no_offer_stands()
    {
        (Round rules, GameSession session) = Deal(2);

        session.Submit(Seat(2), new OfferDrawMessage());
        Assert.Equal(0, session.DrawOfferedBy);
        Assert.Contains(Drain(session)[2], message => message is ErrorMessage);

        session.Submit(Seat(1), new OfferDrawMessage());
        Assert.Equal(2, rules.Asked);
        Drain(session);
        session.Submit(Seat(2), new OfferDrawMessage());
        Assert.Equal(1, session.DrawOfferedBy);
        Assert.Equal(2, rules.Asked);

        // Refused, the seat is asked again what it was asked: its client let go of the question as it offered.
        List<ServerMessage> told = Drain(session)[2];
        Assert.Contains(told, message => message is ErrorMessage);
        Assert.Equal(rules.RequestId, Assert.Single(told.OfType<PromptMessage>()).RequestId);
    }

    /// <summary>
    /// The courtesy (MatchClocks.DrawOfferCourtesy): while seat one's offer stands, each of its later priorities runs no clock
    /// for three seconds, then runs as usual - charged from the third second, not from the next look - and seat two's clock
    /// does not run for it. Each priority is given it once; seat two's priorities are not given it at all.
    /// </summary>
    [Fact]
    public void Each_later_priority_of_the_offerer_runs_no_clock_for_three_seconds_and_never_runs_the_opponents()
    {
        (Round rules, GameSession session) = Deal(2);
        var clocks = new MatchClocks(new Dictionary<int, TimeSpan> { [1] = Hour, [2] = Hour }, Dealt);
        DateTimeOffset now = Dealt;
        void Tick(TimeSpan after)
        {
            now += after;
            clocks.Tick(now, session.WaitingOn, session.Courtesies);
        }

        Tick(TimeSpan.Zero);
        Assert.Equal([1], clocks.Running);

        // Seat one thinks for five seconds, then offers: no courtesy for the priority the offer is made at.
        Tick(TimeSpan.FromSeconds(5));
        session.Submit(Seat(1), new OfferDrawMessage());
        Tick(TimeSpan.Zero);
        Assert.Equal(Hour - TimeSpan.FromSeconds(5), clocks.Left[1]);
        Assert.Equal([2], clocks.Running);

        // Seat two thinks for ten seconds, all on its own clock, and passes.
        Tick(TimeSpan.FromSeconds(10));
        Pass(session, rules);
        Tick(TimeSpan.Zero);
        Assert.Equal(Hour - TimeSpan.FromSeconds(10), clocks.Left[2]);
        Assert.Empty(clocks.Running);

        // Two seconds into seat one's priority: nobody's clock has run.
        Tick(TimeSpan.FromSeconds(2));
        Assert.Empty(clocks.Running);
        Assert.Equal(Hour - TimeSpan.FromSeconds(5), clocks.Left[1]);
        Assert.Equal(Hour - TimeSpan.FromSeconds(10), clocks.Left[2]);

        // Four seconds in: seat one's clock has run for one, from the third second.
        Tick(TimeSpan.FromSeconds(2));
        Assert.Equal([1], clocks.Running);
        Assert.Equal(Hour - TimeSpan.FromSeconds(6), clocks.Left[1]);
        Assert.Equal(Hour - TimeSpan.FromSeconds(10), clocks.Left[2]);

        // Looking again at the same priority gives no more courtesy.
        Tick(TimeSpan.FromSeconds(4));
        Assert.Equal(Hour - TimeSpan.FromSeconds(10), clocks.Left[1]);

        // Seat one passes (the offer stands); seat two passes at once; seat one's next priority has the courtesy again.
        Pass(session, rules);
        Tick(TimeSpan.Zero);
        Assert.Equal([2], clocks.Running);
        Pass(session, rules);
        Tick(TimeSpan.Zero);
        Assert.Empty(clocks.Running);
        Tick(TimeSpan.FromSeconds(3));
        Assert.Equal([1], clocks.Running);
        Assert.Equal(Hour - TimeSpan.FromSeconds(10), clocks.Left[1]);
        Assert.Equal(Hour - TimeSpan.FromSeconds(10), clocks.Left[2]);
    }

    /// <summary>
    /// The courtesy ends with the offer: seat one, given it, acts at once - the offer is withdrawn - and its next priority runs
    /// its clock from the start.
    /// </summary>
    [Fact]
    public void The_courtesy_ends_when_the_offer_is_withdrawn()
    {
        (Round rules, GameSession session) = Deal(2);
        var clocks = new MatchClocks(new Dictionary<int, TimeSpan> { [1] = Hour, [2] = Hour }, Dealt);
        DateTimeOffset now = Dealt;
        void Tick(TimeSpan after)
        {
            now += after;
            clocks.Tick(now, session.WaitingOn, session.Courtesies);
        }

        session.Submit(Seat(1), new OfferDrawMessage());
        Pass(session, rules);
        Tick(TimeSpan.Zero);
        Assert.Empty(clocks.Running);

        Tick(TimeSpan.FromSeconds(1));
        session.Submit(Seat(1), new AnswerMessage { RequestId = rules.RequestId, Answer = new TakeActionAnswer() });
        Assert.Equal(1, rules.Asked);
        Tick(TimeSpan.Zero);
        Assert.Equal([1], clocks.Running);
        Tick(TimeSpan.FromSeconds(1));
        Assert.Equal(Hour - TimeSpan.FromSeconds(1), clocks.Left[1]);
    }

    /// <summary>
    /// An automated seat (LoopGuard, which every one is played through) declines a draw offered, once, whatever it is asked -
    /// the deterministic one, the random one and a language model's seat alike - and plays on.
    /// </summary>
    [Fact]
    public void Every_automated_seat_declines_a_draw()
    {
        foreach (Func<IAutoPlayer> bot in (Func<IAutoPlayer>[])[() => new AutoPlayer(), () => new RandomPlay(7), () => new LlmPlay(new Mute())])
        {
            (Round _, GameSession session) = Deal(2);
            var client = new GameClient(new LoopbackTransport(session, Seat(2)));
            session.Resynchronize(Seat(2));
            var seat = new LoopGuard(bot());

            session.Submit(Seat(1), new OfferDrawMessage());
            client.Pump();
            Assert.True(client.DrawAskedOfMe);
            Assert.True(seat.Act(client));
            client.Pump();
            Assert.Equal([2], client.DrawOffer!.Declined);
            Assert.False(client.DrawAskedOfMe);
            Assert.False(session.IsOver);
        }
    }

    /// <summary>A language model's seat asked about draws (LlmPlay.AsksAboutDraws) accepts or declines as the model says.</summary>
    [Fact]
    public void A_language_model_asked_about_a_draw_accepts_or_declines_as_it_says()
    {
        foreach ((string reply, bool accepted) in (ValueTuple<string, bool>[])[("A", true), ("B", false)])
        {
            (Round _, GameSession session) = Deal(2);
            var client = new GameClient(new LoopbackTransport(session, Seat(2)));
            session.Resynchronize(Seat(2));
            var model = new Mute { Reply = reply };
            var seat = new LoopGuard(new LlmPlay(model) { AsksAboutDraws = true });

            session.Submit(Seat(1), new OfferDrawMessage());
            client.Pump();
            Assert.True(seat.Act(client));
            Assert.Equal(1, model.Usage.Calls);
            Assert.Equal(accepted, session.IsOver);
        }
    }

    /// <summary>
    /// The rules taking out the last player yet to accept draws the game: in a game of three, seat one offers, seat two
    /// accepts, and seat two's next answer takes seat three out (an attack that kills them) - everyone left has accepted.
    /// </summary>
    [Fact]
    public void A_draw_is_agreed_when_the_rules_take_out_the_last_player_yet_to_accept()
    {
        (Round rules, GameSession session) = Deal(3);

        session.Submit(Seat(1), new OfferDrawMessage());
        session.Submit(Seat(2), new AnswerDrawMessage { Accept = true });
        Assert.False(session.IsOver);

        rules.Loses = 3;
        Pass(session, rules);
        Assert.True(session.IsOver);
        Assert.True(session.DrawnByAgreement);
        Assert.All(Drain(session).Values, messages => Assert.Equal(GameOutcomeView.Draw, Outcome(messages)));
    }

    /// <summary>
    /// An offerer the rules take out of the game takes its offer with it: nobody is asked about it any more, and another
    /// offer may be made.
    /// </summary>
    [Fact]
    public void The_offer_is_withdrawn_when_the_rules_take_its_offerer_out()
    {
        (Round rules, GameSession session) = Deal(3);

        session.Submit(Seat(1), new OfferDrawMessage());
        Drain(session);
        rules.Loses = 1;
        Pass(session, rules);
        Assert.False(session.IsOver);
        Assert.Equal(0, session.DrawOfferedBy);
        Assert.All(Drain(session).Values, messages => Assert.Equal(0, messages.OfType<DrawOfferMessage>().Single().Offerer));

        session.Submit(Seat(3), new AnswerDrawMessage { Accept = true });
        Assert.Contains(Drain(session)[3], message => message is ErrorMessage);
        Assert.Null(session.CannotOfferDraw(Seat(rules.Asked)));
    }

    /// <summary>An offerer who leaves the game - out of time - withdraws its offer, and the game goes on.</summary>
    [Fact]
    public void The_offer_is_withdrawn_when_its_offerer_leaves_the_game()
    {
        (Round _, GameSession session) = Deal(3);

        session.Submit(Seat(1), new OfferDrawMessage());
        session.Submit(Seat(2), new AnswerDrawMessage { Accept = true });
        session.Submit(Seat(3), new AnswerDrawMessage { Accept = false });
        session.TimeOut([1]);
        Assert.False(session.IsOver);
        Assert.False(session.DrawnByAgreement);
        Assert.Equal(0, session.DrawOfferedBy);
    }

    /// <summary>
    /// Seats out of time at the same look all leave before the offer is judged, whatever their numbers: in a game of four,
    /// seat three offers, seats one and four accept, and seats two (undecided) and three (the offerer) run out together. The
    /// offer goes with its offerer - an offerer never draws the game by running out - and seats one and four play on.
    /// </summary>
    [Fact]
    public void Seats_out_of_time_together_all_leave_before_the_offer_is_judged()
    {
        (Round rules, GameSession session) = Deal(4);

        Pass(session, rules);
        Pass(session, rules);
        Assert.Equal(3, rules.Asked);
        session.Submit(Seat(3), new OfferDrawMessage());
        session.Submit(Seat(1), new AnswerDrawMessage { Accept = true });
        session.Submit(Seat(4), new AnswerDrawMessage { Accept = true });

        session.TimeOut([2, 3]);
        Assert.False(session.IsOver);
        Assert.False(session.DrawnByAgreement);
        Assert.Equal(0, session.DrawOfferedBy);
        Assert.Equal([1, 4], session.InGame);
    }

    /// <summary>A declaration of no blockers is a pass: the offerer, asked to block, declares none, and its offer stands.</summary>
    [Fact]
    public void An_empty_block_leaves_the_offer_standing()
    {
        (Round rules, GameSession session) = Deal(2);

        session.Submit(Seat(1), new OfferDrawMessage());
        rules.Next = Asking.Blockers;
        Pass(session, rules);
        session.Submit(Seat(1), new AnswerMessage { RequestId = rules.RequestId, Answer = new BlockersDeclaredAnswer() });
        Assert.Equal(1, session.DrawOfferedBy);
    }

    /// <summary>
    /// Only a pass of priority is a pass: the offerer, asked something else - a target it may decline to choose - and
    /// answering it with a pass, has acted, and its offer is withdrawn.
    /// </summary>
    [Fact]
    public void A_pass_of_anything_but_priority_withdraws_the_offer()
    {
        (Round rules, GameSession session) = Deal(2);

        session.Submit(Seat(1), new OfferDrawMessage());
        rules.Next = Asking.Target;
        Pass(session, rules);
        session.Submit(Seat(1), new AnswerMessage { RequestId = rules.RequestId, Answer = new PassAnswer() });
        Assert.Equal(0, session.DrawOfferedBy);
    }

    /// <summary>A seat brought back up to date is sent the offer standing, and one brought back after a draw is told it was one.</summary>
    [Fact]
    public void A_seat_brought_back_up_to_date_is_sent_the_offer_and_then_the_draw()
    {
        (Round _, GameSession session) = Deal(2);

        session.Submit(Seat(1), new OfferDrawMessage());
        Drain(session);
        session.Resynchronize(Seat(2));
        Assert.Equal(1, Drain(session)[2].OfType<DrawOfferMessage>().Single().Offerer);

        session.Submit(Seat(2), new AnswerDrawMessage { Accept = true });
        Drain(session);
        session.Resynchronize(Seat(1));
        List<ServerMessage> told = Drain(session)[1];
        Assert.Equal(GameOutcomeView.Draw, Outcome(told));
        Assert.DoesNotContain(told, message => message is DrawOfferMessage);
    }

    /// <summary>
    /// A seat's answer to a draw is sent once, though another seat's answer comes back before its own: over a connection
    /// that delivers late, seat three declines, seat two's decline comes back without it, and seat three does not decline
    /// again - which would be refused.
    /// </summary>
    [Fact]
    public void An_automated_seat_answers_a_draw_once_while_its_answer_is_on_its_way()
    {
        (Round _, GameSession session) = Deal(3);
        var transport = new Late(session, Seat(3));
        var client = new GameClient(transport);
        session.Resynchronize(Seat(3));
        var seat = new LoopGuard(new AutoPlayer());

        session.Submit(Seat(1), new OfferDrawMessage());
        client.Pump();
        Assert.True(seat.Act(client));
        session.Submit(Seat(2), new AnswerDrawMessage { Accept = false });
        client.Pump();
        Assert.False(client.DrawAskedOfMe);
        Assert.False(seat.Act(client));
        Assert.Equal(1, transport.Held);

        transport.Deliver();
        client.Pump();
        Assert.DoesNotContain(client.Errors, error => error.Code == ErrorCode.IllegalAnswer);
        Assert.Equal([2, 3], client.DrawOffer!.Declined);
    }

    /// <summary>A transport whose sends wait until the test delivers them, as a slow connection's do.</summary>
    private sealed class Late(GameSession session, PlayerId seat) : IGameTransport
    {
        private readonly Queue<ClientMessage> _held = new();

        public int Held => _held.Count;

        public bool IsConnected => true;

        public void Send(ClientMessage message) => _held.Enqueue(message);

        public void Deliver()
        {
            while (_held.TryDequeue(out ClientMessage? message))
            {
                session.Submit(seat, message);
            }
        }

        public IReadOnlyList<ServerMessage> Receive() => session.Drain(seat);

        public void Dispose()
        {
        }
    }

    /// <summary>A model that says what it is told to, counting each call; "B" unless told otherwise.</summary>
    private sealed class Mute : ILanguageModel
    {
        public string Reply { get; init; } = "B";

        public ModelUsage Usage { get; } = new();

        public string Continue(IReadOnlyList<ChatTurn> conversation)
        {
            Usage.Record(1, 1);
            return Reply;
        }
    }

    /// <summary>
    /// Rules that only go round priority: the seat asked passes it on to the next seat still in, or takes an action and is
    /// asked again. Every answer is a new request.
    /// </summary>
    private sealed class Round(int players) : IRulesHost
    {
        private readonly HashSet<int> _left = [];

        /// <summary>The seats the rules took out of the game (<see cref="Loses"/>).</summary>
        private readonly HashSet<int> _lost = [];

        /// <summary>A seat the next answer takes out of the game, as an attack that kills a player does; zero for none.</summary>
        public int Loses { get; set; }

        public List<(int Seat, string Kind)> Answers { get; } = [];

        public int Asked { get; private set; } = 1;

        public int RequestId { get; private set; } = 1;

        /// <summary>
        /// What the seat priority comes round to is asked next, once: priority, unless a test wants a target or attackers
        /// first. Its answer asks that seat priority again.
        /// </summary>
        public Asking Next { get; set; }

        private Asking _asking;

        public IReadOnlyList<int> Seats => [.. Enumerable.Range(1, players)];

        public IReadOnlyList<string> SeatNames => [.. Seats.Select(seat => $"Seat {seat}")];

        public int Team(int seat) => seat;

        public IReadOnlyList<DeckCardView> Decklist(int seat) => [];

        public RulesStep Start() => Step();

        public RulesStep Answer(int requestId, int seat, AnswerPayload answer)
        {
            if (requestId != RequestId || seat != Asked)
            {
                throw new RulesRejectedException(ErrorCode.StaleReply, $"request {requestId} is not outstanding.");
            }

            Answers.Add((seat, answer.GetType().Name));
            if (_asking != Asking.Priority)
            {
                _asking = Asking.Priority;
            }
            else if (answer is PassAnswer)
            {
                Asked = NextSeat(Asked);
                _asking = Next;
                Next = Asking.Priority;
            }

            if (Loses != 0)
            {
                _lost.Add(Loses);
                if (Asked == Loses)
                {
                    Asked = NextSeat(Asked);
                }

                Loses = 0;
            }

            RequestId++;
            return Step();
        }

        private int NextSeat(int seat)
        {
            do
            {
                seat = seat % players + 1;
            }
            while (_left.Contains(seat) || _lost.Contains(seat));

            return seat;
        }

        public bool CanConcede(int seat) => false;

        public RulesStep Concede(int seat) => throw new NotSupportedException();

        public RulesStep Leave(int seat)
        {
            _left.Add(seat);
            if (Asked == seat)
            {
                Asked = NextSeat(seat);
                RequestId++;
            }

            return Step();
        }

        /// <summary>
        /// A face-down permanent of seat 1's the end of the game reveals (CR 708.9), which makes these rules end their game
        /// when it is ended from outside them (<see cref="End"/>); null for rules that end nothing.
        /// </summary>
        public ObjectView? FaceDownAtEnd { get; set; }

        private bool _ended;

        public RulesStep? End(int? winner)
        {
            if (FaceDownAtEnd is not { } revealed)
            {
                return null;
            }

            _ended = true;
            return new RulesStep(
                [],
                Seats.ToDictionary(seat => seat, seat =>
                {
                    PlayerView view = ViewFor(seat);
                    view.Objects.Add(revealed);
                    return view;
                }),
                Seats.ToDictionary(seat => seat, IReadOnlyList<(long, GameEventView)> (_) =>
                [
                    (1, new CardsRevealedEvent { Player = 1, Cards = [revealed.Handle] }),
                    (2, new GameEndedEvent { Outcome = GameOutcomeView.Draw }),
                ]),
                2,
                true);
        }

        public PlayerView View(int seat) => ViewFor(seat);

        public ObjectView? Oracle(string cardId) => null;

        private IEnumerable<int> Standing => Seats.Where(seat => !_left.Contains(seat) && !_lost.Contains(seat));

        public bool IsOver => _ended || Standing.Count() <= 1;

        public GameOutcomeView Outcome => _ended ? GameOutcomeView.Draw : IsOver ? GameOutcomeView.TeamWon : GameOutcomeView.InProgress;

        public int WinningTeam => !_ended && IsOver ? Standing.FirstOrDefault() : 0;

        public IReadOnlyList<RulesAnswer> History => [];

        public void Dispose()
        {
        }

        private RulesStep Step() => new(
            IsOver ? [] : [new RulesRequest(RequestId, Asked, _asking switch
            {
                Asking.Target => new ChooseTargetPromptMessage { Player = Asked },
                Asking.Attackers => new DeclareAttackersPromptMessage { Player = Asked },
                Asking.Blockers => new DeclareBlockersPromptMessage { Player = Asked },
                _ => new PriorityPromptMessage { Player = Asked },
            }, false, null)],
            Seats.ToDictionary(seat => seat, ViewFor),
            Seats.ToDictionary(seat => seat, IReadOnlyList<(long, GameEventView)> (_) => []),
            0,
            IsOver);

        private PlayerView ViewFor(int viewer) => new()
        {
            Viewer = viewer,
            Turn = 1,
            Step = TurnStepView.PrecombatMain,
            ActivePlayer = 1,
            PriorityPlayer = Asked,
            Players = [.. Seats.Select(seat => new PlayerSummaryView
            {
                Seat = seat, Name = $"Seat {seat}", Team = seat, Life = 20,
                Status = _left.Contains(seat) ? PlayerStatusView.Conceded : _lost.Contains(seat) ? PlayerStatusView.Lost : PlayerStatusView.Active,
            })],
        };
    }
}

/// <summary>What <c>DrawOfferTests.Round</c> asks a seat.</summary>
internal enum Asking
{
    Priority,
    Target,
    Attackers,
    Blockers,
}
