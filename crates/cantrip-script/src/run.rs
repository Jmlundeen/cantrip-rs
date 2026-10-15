//! Plays a `GameScript` against the native engine with the C# runner's semantics
//! (`src/Cantrip.Scripting/ScriptRunner.cs`): one cursor over the whole list, each line tried against whatever the
//! engine asks, the simplest answer given whenever the line under the cursor does not apply.

use std::collections::VecDeque;

use cantrip_engine::undo::Undo;
use cantrip_engine::*;

use crate::dump::{StateDump, bot};
use crate::parse::{self, ActionStep, Expectation, GameScript, HandoverStep, ScriptStep, ScriptVerb, WaitStep, bot_id, split_copy};

/// How many turns a line may wait to become possible before the script is taken to be stuck.
pub const TURNS_TO_WAIT: u32 = 4;
const ASK_LIMIT: usize = 20_000;

pub struct ScriptResult {
    pub game: Game,
    pub dump: StateDump,
    pub trace: Vec<String>,
    pub handed_over: bool,
}

#[derive(Debug, Clone)]
pub struct ScriptFailure {
    pub message: String,
    pub report: String,
}

impl std::fmt::Display for ScriptFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.report)
    }
}

struct Fail(String);

type R<T> = Result<T, Fail>;

fn fail<T>(message: impl Into<String>) -> R<T> {
    Err(Fail(message.into()))
}

/// A cast or activation in progress, from the line that began it: what `Spend:` still has to say.
struct Casting {
    player: PlayerId,
    /// The X or kicker count the line asked for (`X:`, `Kicked:`).
    x: Option<u16>,
    spend: VecDeque<(Mana, Option<ManaRestriction>)>,
    /// The creatures a `Convoke:` taps, in the order the payment asks (CR 702.51a), and how many cards `Delve:` exiles
    /// from the graveyard, the first offered each time (CR 702.66a).
    convoke: VecDeque<String>,
    delve: u32,
}

impl Casting {
    fn new(player: PlayerId, x: Option<u16>, spend: VecDeque<(Mana, Option<ManaRestriction>)>) -> Casting {
        Casting { player, x, spend, convoke: VecDeque::new(), delve: 0 }
    }
}

/// What the script means a player to answer a question with.
#[derive(Clone, Debug, PartialEq)]
pub enum Intent {
    Answer(Answer),
    /// Take back the latest mana ability (`Bot1.Undo`).
    Undo,
}

/// Where playing on stopped.
enum Stop<'s> {
    Done,
    Asked(Request),
    Check(&'s Expectation),
}

/// An intention, and the line it comes from (none for a default or a payment the line in progress implies).
struct Decision<'s> {
    intent: Intent,
    line: Option<&'s ActionStep>,
}

struct Run<'s> {
    script: &'s GameScript,
    game: Game,
    cursor: usize,
    since: u32,
    casting: Option<Casting>,
    trace: Vec<String>,
    handed_over: bool,
    /// The colors an activation line named (`Colors: Red, Red, Green`) for the "any combination" question its mana
    /// ability asks next, as the C# runner takes them.
    colors: Option<Vec<Color>>,
    /// The mana abilities that can be taken back (`Bot1.Undo`), as the host offers them.
    undo: Undo<()>,
    /// A driven player's question, and what the script means them to answer, until they do.
    asked: Option<(Request, Decision<'s>)>,
}

pub fn run(script: &GameScript) -> Result<ScriptResult, ScriptFailure> {
    let mut run = Run::deal(script)?;
    match run.play() {
        Ok(()) => {
            let dump = run.judge()?;
            Ok(ScriptResult { game: run.game, dump, trace: run.trace, handed_over: run.handed_over })
        }
        Err(Fail(message)) => Err(run.failure(message)),
    }
}

/// A scenario played through a table rather than by the runner alone (`docs/scenarios-in-the-browser.md`): the runner
/// answers for the seats it plays, and stops on each question to a driven seat - a browser page, say - with what the
/// script means it to answer, which the seat's answer must then be.
pub struct Scripted {
    run: Run<'static>,
}

/// Where a scenario played through a table stopped.
pub enum Advance {
    /// A driven seat is asked something, and the script means this answer.
    Ask(Expected),
    /// A check of what a client shows (`? Client.Bot1.Stack = ...`), to be judged by the driven seats' pages now.
    Check(Expectation),
    /// The script is over: [`Scripted::verdict`] judges the game, and [`Scripted::client_expectations`] are for the pages.
    Done,
}

/// A driven seat's question, and what the script means it to answer.
pub struct Expected {
    pub request: Request,
    pub intent: Intent,
    /// The line it comes from, `12: Bot1.Hand.Island.Play`, or none for a default or a payment the line in progress implies.
    pub line: Option<String>,
}

impl Scripted {
    /// Deals the scenario's game. The script lives as long as the process: a table plays few.
    pub fn deal(script: GameScript) -> Result<Scripted, ScriptFailure> {
        let script: &'static GameScript = Box::leak(Box::new(script));
        Ok(Scripted { run: Run::deal(script)? })
    }

    pub fn script(&self) -> &GameScript {
        self.run.script
    }

    /// The game the scenario is played in, which a table may hold in its stead between steps (`std::mem::swap`).
    pub fn game(&mut self) -> &mut Game {
        &mut self.run.game
    }

    /// Plays on, answering for every player but the `driven` ones, until a driven player is asked (`Some`) or the
    /// script ends (`None`; [`Scripted::verdict`] then judges it).
    pub fn advance(&mut self, driven: &[PlayerId]) -> Result<Advance, ScriptFailure> {
        match self.run.play_until(driven) {
            Ok(Stop::Done) => Ok(Advance::Done),
            Ok(Stop::Check(check)) => Ok(Advance::Check(check.clone())),
            Ok(Stop::Asked(request)) => {
                let (_, decision) = self.run.asked.as_ref().expect("held");
                let line = decision.line.map(|step| format!("{}: {}", step.line, step.text));
                Ok(Advance::Ask(Expected { request, intent: decision.intent.clone(), line }))
            }
            Err(Fail(message)) => Err(self.run.failure(message)),
        }
    }

    /// The permanent whose mana ability `player` may take back now, as the script's game has it.
    pub fn undo_offered(&self, player: PlayerId) -> Option<ObjectId> {
        self.run.undo.offered(player)
    }

    /// The `[Expect]` lines about what a client shows, for the pages to judge once the script is over.
    pub fn client_expectations(&self) -> Vec<Expectation> {
        self.run.script.expectations.iter().filter(|e| e.is_client()).cloned().collect()
    }

    /// The driven player's answer to the question `advance` stopped on. A failure if it is not the one the script means.
    pub fn answered(&mut self, given: Intent) -> Result<(), ScriptFailure> {
        self.run.answered(given).map_err(|Fail(message)| self.run.failure(message))
    }

    /// Whether the game ended as the scenario expects, once `advance` has said it is over.
    pub fn verdict(&mut self) -> Result<StateDump, ScriptFailure> {
        self.run.judge()
    }

    /// A failure of the scenario found outside the runner, with the runner's report.
    pub fn failure(&self, message: String) -> ScriptFailure {
        self.run.failure(message)
    }
}

impl<'s> Run<'s> {
    fn deal(script: &'s GameScript) -> Result<Run<'s>, ScriptFailure> {
        let decks: Vec<Vec<String>> = (0..script.decks.len()).map(|i| script.decklist(i)).collect();
        let chooser = if script.opening { Some(PlayerId((script.chooser.max(1) - 1) as u8)) } else { None };
        let mut game = deal(&decks, 1, false, chooser).map_err(|m| ScriptFailure { message: m.clone(), report: m })?;
        // CANTRIP_VERIFY=1: every rollback is checked to restore the state exactly.
        game.verify = std::env::var_os("CANTRIP_VERIFY").is_some();
        game.state.force_rolls(&script.dice);
        Ok(Run {
            script,
            game,
            cursor: 0,
            since: 1,
            casting: None,
            trace: Vec::new(),
            handed_over: false,
            colors: None,
            undo: Undo::default(),
            asked: None,
        })
    }

    /// The game as it ended, judged against `[Expect]`.
    fn judge(&self) -> Result<StateDump, ScriptFailure> {
        let dump = StateDump::of(&self.game);
        let mismatches = dump.mismatches(&self.script.expectations);
        if !mismatches.is_empty() {
            return Err(self.failure(format!("the game did not end as expected:
  {}", mismatches.join("
  "))));
        }
        Ok(dump)
    }
}

fn mana_symbol(text: &str) -> Option<Mana> {
    match text.trim().to_ascii_uppercase().as_str() {
        "W" => Some(Mana::W),
        "U" => Some(Mana::U),
        "B" => Some(Mana::B),
        "R" => Some(Mana::R),
        "G" => Some(Mana::G),
        "C" => Some(Mana::C),
        _ => None,
    }
}

fn is_colorless(text: &str) -> bool {
    let t = text.trim();
    t.eq_ignore_ascii_case("C") || t.eq_ignore_ascii_case("Colorless")
}

fn color_named(text: &str, short: bool) -> Option<Color> {
    let t = text.trim();
    for c in Color::ALL {
        if t.eq_ignore_ascii_case(c.name()) || (short && t.len() == 1 && t.eq_ignore_ascii_case(&c.symbol().to_string())) {
            return Some(c);
        }
    }
    None
}

fn engine_zone(zone: parse::ZoneKind) -> Option<ZoneKind> {
    Some(match zone {
        parse::ZoneKind::Library => ZoneKind::Library,
        parse::ZoneKind::Hand => ZoneKind::Hand,
        parse::ZoneKind::Battlefield => ZoneKind::Battlefield,
        parse::ZoneKind::Graveyard => ZoneKind::Graveyard,
        parse::ZoneKind::Stack => ZoneKind::Stack,
        parse::ZoneKind::Exile => ZoneKind::Exile,
        _ => return None,
    })
}

fn engine_step(step: parse::TurnStep) -> Step {
    use parse::TurnStep as T;
    match step {
        T::Untap => Step::Untap,
        T::Upkeep => Step::Upkeep,
        T::Draw => Step::Draw,
        T::PrecombatMain => Step::PrecombatMain,
        T::BeginCombat => Step::BeginCombat,
        T::DeclareAttackers => Step::DeclareAttackers,
        T::DeclareBlockers => Step::DeclareBlockers,
        T::CombatDamage => Step::CombatDamage,
        T::EndOfCombat => Step::EndOfCombat,
        T::PostcombatMain => Step::PostcombatMain,
        T::End => Step::End,
        T::Cleanup => Step::Cleanup,
    }
}

fn player_of(bot: i32) -> PlayerId {
    PlayerId((bot - 1) as u8)
}

/// Whether a verb answers a kind of question (`Answers` in the C# runner).
fn answers(verb: ScriptVerb, kind: &RequestKind) -> bool {
    use ScriptVerb as V;
    match verb {
        V::Play | V::Cast | V::Activate | V::Foretell | V::Suspend | V::Pass => matches!(kind, RequestKind::Priority { .. }),
        V::PayCost => matches!(kind, RequestKind::PayCosts { .. }),
        V::Undo => matches!(kind, RequestKind::Priority { .. } | RequestKind::PayCosts { .. }),
        V::Accept | V::Decline => matches!(kind, RequestKind::May { .. } | RequestKind::UnlessPays { .. }),
        V::Choose => matches!(kind, RequestKind::ChooseCards { .. } | RequestKind::TapForPower { .. } | RequestKind::CollectEvidence { .. }),
        V::ChoosePlayer => matches!(kind, RequestKind::ChoosePlayer { .. }),
        V::Discard => matches!(kind, RequestKind::DiscardToHandSize { .. }),
        V::Attack => matches!(kind, RequestKind::DeclareAttackers { .. }),
        V::Block => matches!(kind, RequestKind::DeclareBlockers { .. }),
        V::AssignDamage => matches!(kind, RequestKind::AssignCombatDamage { .. }),
        V::Modes => matches!(kind, RequestKind::ChooseModes { .. } | RequestKind::ChooseSubtype { .. }),
        V::Target => matches!(kind, RequestKind::ChooseTarget { .. }),
        V::Divide => matches!(kind, RequestKind::ChooseDividedTargets { .. }),
        V::Colors => matches!(kind, RequestKind::ChooseColors { .. }),
        V::PlayFirst | V::DrawFirst => matches!(kind, RequestKind::ChooseStartingPlayer { .. }),
        V::Mulligan | V::Keep => matches!(kind, RequestKind::Mulligan { .. }),
        V::Triggers => matches!(kind, RequestKind::OrderTriggers { .. }),
        V::Arrange => matches!(kind, RequestKind::DistributeCards { .. }),
        V::Search => matches!(kind, RequestKind::SearchLibrary { .. }),
        V::Replace => matches!(kind, RequestKind::ChooseReplacement { .. }),
        V::Name => matches!(kind, RequestKind::ChooseName { .. } | RequestKind::ChooseModes { .. }),
        _ => false,
    }
}

fn kind_name(kind: &RequestKind) -> &'static str {
    match kind {
        RequestKind::Priority { .. } => "Priority",
        RequestKind::PayCosts { .. } => "PayCosts",
        RequestKind::UnlessPays { .. } => "UnlessPays",
        RequestKind::ChooseCards { .. } => "ChooseCards",
        RequestKind::ChooseTarget { .. } => "ChooseTarget",
        RequestKind::ChooseModes { .. } => "ChooseModes",
        RequestKind::ChooseColors { .. } => "ChooseColors",
        RequestKind::May { .. } => "May",
        RequestKind::ChoosePlayer { .. } => "ChoosePlayer",
        RequestKind::TapForPower { .. } => "TapForPower",
        RequestKind::CollectEvidence { .. } => "CollectEvidence",
        RequestKind::DeclareAttackers { .. } => "DeclareAttackers",
        RequestKind::DeclareBlockers { .. } => "DeclareBlockers",
        RequestKind::AssignCombatDamage { .. } => "AssignCombatDamage",
        RequestKind::DiscardToHandSize { .. } => "DiscardToHandSize",
        RequestKind::Mulligan { .. } => "Mulligan",
        RequestKind::ChooseStartingPlayer { .. } => "ChooseStartingPlayer",
        RequestKind::OrderTriggers { .. } => "OrderTriggers",
        RequestKind::DistributeCards { .. } => "DistributeCards",
        RequestKind::SearchLibrary { .. } => "SearchLibrary",
        RequestKind::ChooseReplacement { .. } => "ChooseReplacement",
        RequestKind::ChooseName { .. } => "ChooseName",
        RequestKind::ChooseSubtype { .. } => "ChooseSubtype",
        RequestKind::ChooseDividedTargets { .. } => "ChooseDividedTargets",
    }
}

fn pays(owed: &ManaCost, mana: Mana) -> bool {
    owed.generic > 0 || owed.symbols[mana as usize] > 0 || owed.hybrid.iter().flatten().any(|(a, b)| *a == mana || *b == mana)
}

impl<'s> Run<'s> {
    fn current(&self) -> Option<&'s ScriptStep> {
        self.script.steps.get(self.cursor)
    }

    fn turn(&self) -> u32 {
        self.game.state.turn.number
    }

    fn next(&mut self) {
        self.cursor += 1;
        self.since = self.turn().max(1);
    }

    fn failure(&self, message: String) -> ScriptFailure {
        let mut report = format!("{}: {message}\n", self.script.name);
        if let Some(r) = self.game.pending() {
            report.push_str(&format!("\nOutstanding: {} asked {}\n", bot(r.player), self.describe_request(r)));
        }
        report.push_str("\nTrace:\n");
        for line in self.trace.iter().rev().take(40).rev() {
            report.push_str(&format!("  {line}\n"));
        }
        report.push_str("\nState:\n");
        report.push_str(&StateDump::of(&self.game).render());
        ScriptFailure { message, report }
    }

    fn describe_request(&self, r: &Request) -> String {
        let g = &self.game;
        let name = |id: ObjectId| g.name(id).to_string();
        match &r.kind {
            RequestKind::Priority { options } | RequestKind::PayCosts { options, .. } | RequestKind::UnlessPays { options, .. } => {
                let list: Vec<String> = options
                    .iter()
                    .map(|o| {
                        let src = o.source.map(name).unwrap_or_default();
                        let desc = describe(g.cards, &g.state, o);
                        format!("{:?} {src} #{} {desc}{}", o.kind, o.index, o.mana.map(|m| format!(" {m:?}")).unwrap_or_default())
                    })
                    .collect();
                format!("{} [{}]", kind_name(&r.kind), list.join(" | "))
            }
            RequestKind::ChooseCards { eligible, count, .. } => {
                format!("ChooseCards {count} of [{}]", eligible.iter().map(|&i| name(i)).collect::<Vec<_>>().join(", "))
            }
            other => format!("{} {other:?}", kind_name(other)),
        }
    }

    fn deliver(&mut self, request: &Request, answer: Answer, step: Option<&ActionStep>) -> R<()> {
        let who = bot(request.player);
        match step {
            Some(s) => self.trace.push(format!("T{} {}: line {} {}", self.turn(), who, s.line, s.text)),
            None => {
                if !matches!(answer, Answer::Pass) {
                    self.trace.push(format!("T{} {}: (default) {answer:?} to {}", self.turn(), who, kind_name(&request.kind)));
                }
            }
        }
        let noted = Undo::<()>::before(&self.game, request.player, &answer);
        match self.game.answer(request.id, request.player, answer.clone()) {
            Ok(()) => {
                if self.undo.may_take_back(&self.game, &noted) {
                    self.undo.push(noted, ());
                }
                Ok(())
            }
            Err(Rejected(why)) => match step {
                Some(s) => fail(format!("line {}: the engine refused {}: {why}", s.line, s.text)),
                None => fail(format!("the engine refused {who}'s unscripted answer ({answer:?}): {why}")),
            },
        }
    }

    fn play(&mut self) -> R<()> {
        match self.play_until(&[])? {
            Stop::Done => Ok(()),
            _ => unreachable!("no player is driven"),
        }
    }

    /// Plays on, answering for every player but the `driven` ones, until the script ends, a driven player is asked (its
    /// intention held in `asked` until [`Run::answered`]), or, with a player driven, a check of what a client shows comes up.
    fn play_until(&mut self, driven: &[PlayerId]) -> R<Stop<'s>> {
        let mut asks = 0;
        loop {
            if let Some(check) = self.run_checks(!driven.is_empty())? {
                return Ok(Stop::Check(check));
            }
            if self.game.is_over() {
                if let Some(step) = self.current() {
                    return fail(format!("the game ended before line {}: {}", step.line(), step.text()));
                }
                return Ok(Stop::Done);
            }
            asks += 1;
            if asks > ASK_LIMIT {
                return fail("the game went on too long: looping?");
            }
            let Some(request) = self.game.pending().cloned() else {
                return fail("the engine stopped with nothing to ask and the game not over");
            };

            let stack_empty = self.game.state.stack.is_empty();
            if self.current().is_none()
                && self.casting.is_none()
                && stack_empty
                && matches!(request.kind, RequestKind::Priority { .. })
            {
                return Ok(Stop::Done);
            }

            if let Some(ScriptStep::Handover(h)) = self.current()
                && self.waiting(h, &request)
            {
                self.handed_over = true;
                self.next();
                return Ok(Stop::Done);
            }

            // `! Bot1.Accept`, `! Bot1.Choose`, a bare `! Bot1.Target` or `! Bot2.Block`...: the player is not asked that kind of question
            // before their next priority (a bare Block: no block is possible, so the declaration is never asked for).
            if let Some(ScriptStep::Refused { action, line, text }) = self.current()
                && ((action.verb.is_question() && !(matches!(action.verb, ScriptVerb::Choose | ScriptVerb::Colors) && action.positional().next().is_some()))
                    || (matches!(action.verb, ScriptVerb::Target | ScriptVerb::Modes | ScriptVerb::Block) && action.positional().next().is_none()))
                && request.player == player_of(action.bot)
            {
                match request.kind {
                    ref k if answers(action.verb, k) => return fail(format!("line {line} says {text}, but the player was asked that question")),
                    RequestKind::Priority { .. } => {
                        self.next();
                        continue;
                    }
                    _ => {}
                }
            }

            // A player who has lost has left the game and is asked nothing, priority included (CR 800.4a): the refusal of a
            // question to them holds once the stack they might have been asked about has emptied, with no such question put.
            if let Some(ScriptStep::Refused { action, .. }) = self.current()
                && action.verb.is_question()
                && self.game.state.player(player_of(action.bot)).lost
                && stack_empty
                && matches!(request.kind, RequestKind::Priority { .. })
            {
                self.next();
                continue;
            }

            // `! Bot2.Block(Blocker > Attacker)` when the declare-blockers question never came: an effect (Headliner Scarlett's) left
            // the defender no legal block at all, so the engine asks nothing. The refusal holds if, with priority in the
            // declare-blockers step, the blocker is an untapped creature of that player, the attacker is attacking that player,
            // and no block was made.
            if let Some(ScriptStep::Refused { action, line, text }) = self.current()
                && matches!(action.verb, ScriptVerb::Block)
                && matches!(request.kind, RequestKind::Priority { .. })
                && self.game.state.turn.step() == Step::DeclareBlockers
            {
                self.check_block_never_asked(action, *line, text)?;
                self.next();
                continue;
            }

            // `! Bot1.Hand.Fire Covenant.Cast(X: 6)` after the cast that announces X: X cannot be announced that high. The announcement
            // is raised one at a time; the refusal holds if raising stops short of 6 (life to pay, creatures to sacrifice) and fails if
            // 6 is reached. The card announcing must be the one named, or the refusal proves nothing.
            if let Some(ScriptStep::Refused { action, line, text }) = self.current()
                && matches!(action.verb, ScriptVerb::Cast | ScriptVerb::Activate)
                && let Some(wanted) = action.named("X")
                && request.player == player_of(action.bot)
                && let RequestKind::PayCosts { announcing: true, source, x, options, .. } = &request.kind
            {
                let Ok(wanted) = wanted.trim().parse::<u16>() else { return fail(format!("line {line}: {wanted} is not a number")) };
                if !action.card.as_deref().is_some_and(|card| self.game.name(*source).eq_ignore_ascii_case(card)) {
                    return fail(format!("line {line}: {text} proves nothing: {} is the one announcing X", self.game.name(*source)));
                }
                if *x >= wanted {
                    return fail(format!("line {line} says {text} must not be allowed, but X was announced up to {wanted}"));
                }
                match options.iter().find(|o| o.kind == ActionKind::IncreaseX) {
                    Some(raise) => {
                        let answer = Answer::Act(Action::of(raise));
                        self.deliver(&request, answer, None)?;
                    }
                    None => self.next(),
                }
                continue;
            }

            // `! Bot1.Undo`: at the player's next priority or payment, there is no mana ability to take back.
            if let Some(ScriptStep::Refused { action, line, text }) = self.current()
                && matches!(action.verb, ScriptVerb::Undo)
                && request.player == player_of(action.bot)
                && answers(action.verb, &request.kind)
            {
                if let Some(source) = self.undo.offered(request.player) {
                    return fail(format!("line {line} says {text}, but {}'s mana ability may be taken back", self.game.name(source)));
                }
                self.next();
                continue;
            }

            if let Some(ScriptStep::Refused { action, line, text }) = self.current()
                && request.player == player_of(action.bot)
                && answers(action.verb, &request.kind)
            {
                if let RequestKind::Priority { options } = &request.kind {
                    let mut offered = self.offered(action, options, true)?;
                    // A refusal that names targets (`! Bot2.Hand.Lightning Bolt.Cast(Bot1.Battlefield.Emrakul)`) refuses only
                    // the option that could be announced with them, and proves nothing when no such object exists.
                    let specs: Vec<&str> = action.positional().collect();
                    if !specs.is_empty() {
                        let all: Vec<ObjectId> = (0..self.game.state.objects.len()).map(|i| ObjectId(i as u32)).collect();
                        for spec in &specs {
                            if bot_id(spec).is_none() && !spec.to_ascii_lowercase().starts_with("stack.") && self.objects(action, spec, &all)?.is_empty() {
                                return fail(format!("line {line}: {text} proves nothing: nothing is named {spec}"));
                            }
                        }
                        let mut kept = Vec::new();
                        for o in offered {
                            let mut every = true;
                            for spec in &specs {
                                every &= self.target(action, spec, &o.target_candidates)?.is_some();
                            }
                            if every {
                                kept.push(o);
                            }
                        }
                        offered = kept;
                    }
                    if !offered.is_empty() {
                        return fail(format!("line {line} says {text} must not be offered, but it is"));
                    }
                }
                self.check_refused_declaration(action, *line, text, &request.kind)?;
                self.next();
                continue;
            }

            self.fail_if_stuck()?;

            if let Some(ScriptStep::Wait(w)) = self.current()
                && self.reached(w)
            {
                self.next();
                continue;
            }

            if driven.contains(&request.player) {
                let decision = self.intend(&request)?;
                self.asked = Some((request.clone(), decision));
                return Ok(Stop::Asked(request));
            }
            self.decide(&request)?;
        }
    }

    /// Runs the checks under the cursor. A check of what a client shows is handed out when a client is watching
    /// (`watched`) and skipped when none is.
    fn run_checks(&mut self, watched: bool) -> R<Option<&'s Expectation>> {
        while let Some(ScriptStep::Check { expectation, .. }) = self.current() {
            if expectation.is_client() {
                self.next();
                if watched {
                    return Ok(Some(expectation));
                }
                continue;
            }
            let dump = StateDump::of(&self.game);
            let mismatches = dump.mismatches(std::slice::from_ref(expectation));
            if let Some(m) = mismatches.first() {
                return fail(format!("a check did not hold: {m}"));
            }
            self.next();
        }
        Ok(None)
    }

    fn waiting(&self, h: &HandoverStep, request: &Request) -> bool {
        match h.verb {
            None => true,
            Some(verb) => (h.bot == 0 || request.player == player_of(h.bot)) && answers(verb, &request.kind),
        }
    }

    fn reached(&self, w: &WaitStep) -> bool {
        let t = &self.game.state.turn;
        t.active == player_of(w.bot) && t.step() == engine_step(w.step) && w.turn.is_none_or(|n| n == t.number)
    }

    fn fail_if_stuck(&self) -> R<()> {
        let turn = self.turn();
        match self.current() {
            Some(ScriptStep::Wait(w)) => {
                if let Some(t) = w.turn {
                    if turn > t {
                        return fail(format!("turn {t} went by without reaching line {}: {}", w.line, w.text));
                    }
                } else if turn > self.since + TURNS_TO_WAIT {
                    return fail(format!("line {} was not reached in {TURNS_TO_WAIT} turns: {}", w.line, w.text));
                }
            }
            Some(step @ (ScriptStep::Handover(_) | ScriptStep::Refused { .. } | ScriptStep::Action(_))) => {
                if turn > self.since + TURNS_TO_WAIT {
                    return fail(format!(
                        "line {} did not become possible in {TURNS_TO_WAIT} turns: {}",
                        step.line(),
                        step.text()
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// A driven player answered the question `play_until` stopped on: it must be the answer the script means, which is
    /// then given (`None` is a mana ability taken back).
    fn answered(&mut self, given: Intent) -> R<()> {
        let Some((request, decision)) = self.asked.take() else {
            return fail("an answer came with nothing asked");
        };
        if !self.same_but_for_interchangeable(&given, &decision.intent) {
            let what = match decision.line {
                Some(step) => format!("line {}: {}", step.line, step.text),
                None => "the default".to_string(),
            };
            return fail(format!("{} answered {given:?} to {}, but {what} means {:?}", bot(request.player), kind_name(&request.kind), decision.intent));
        }
        self.carry_out(&request, decision)
    }

    /// Whether two answers are the same once objects nobody could tell apart are taken for one another: two untapped
    /// Forests of one player, two Islands in a hand (scenarios/README.md: same-named cards in one place are
    /// interchangeable). A page clicking the top of a pile of Forests answers what the script means by "a Forest".
    fn same_but_for_interchangeable(&self, given: &Intent, meant: &Intent) -> bool {
        match (given, meant) {
            (Intent::Undo, Intent::Undo) => true,
            (Intent::Answer(a), Intent::Answer(b)) => a == b || self.canonical(a) == self.canonical(b),
            _ => false,
        }
    }

    /// The answer with each object it names replaced by the first object it is indistinguishable from.
    fn canonical(&self, answer: &Answer) -> Answer {
        let s = &self.game.state;
        let signature = |id: ObjectId| {
            let o = s.obj(id);
            // Summoning sickness tells creatures apart (CR 302.6), and nothing else.
            let sick = o.controlled_since == s.turn.number && self.game.chars(id).is(CardType::Creature);
            (self.game.name(id), o.owner, o.controller, o.zone, o.token, o.tapped, o.damage, o.counters, sick)
        };
        let one = |id: ObjectId| -> ObjectId {
            if id.index() >= s.objects.len() {
                return id;
            }
            let wanted = signature(id);
            (0..s.objects.len() as u32).map(ObjectId).find(|&other| live(s, other) && signature(other) == wanted).unwrap_or(id)
        };
        let target = |t: Target| match t {
            Target::Object(id) => Target::Object(one(id)),
            other => other,
        };
        match answer {
            Answer::Act(a) => {
                let mut a = a.clone();
                a.source = a.source.map(one);
                a.targets = a.targets.iter().map(|&t| target(t)).collect();
                Answer::Act(a)
            }
            Answer::Cards(ids) => Answer::Cards(ids.iter().map(|&id| one(id)).collect()),
            Answer::Targets(ts) => Answer::Targets(ts.iter().map(|&t| target(t)).collect()),
            Answer::Attack(v) => Answer::Attack(v.iter().map(|&(a, t)| (one(a), target(t))).collect()),
            Answer::Block(v) => Answer::Block(v.iter().map(|&(b, a)| (one(b), one(a))).collect()),
            Answer::Shares(v) => Answer::Shares(v.iter().map(|&(a, c, n)| (one(a), c.map(one), n)).collect()),
            Answer::Placements { placements, shuffle } => {
                Answer::Placements { placements: placements.iter().map(|p| p.iter().map(|&id| one(id)).collect()).collect(), shuffle: *shuffle }
            }
            Answer::Divided(v) => Answer::Divided(v.iter().map(|&(t, n)| (target(t), n)).collect()),
            other => other.clone(),
        }
    }

    fn decide(&mut self, request: &Request) -> R<()> {
        let decision = self.intend(request)?;
        self.carry_out(request, decision)
    }

    /// What the script means the asked player to answer: the line under the cursor if it applies, else the payment the
    /// line in progress implies, else the default. Called once per question: it moves the payment in progress on.
    fn intend(&mut self, request: &Request) -> R<Decision<'s>> {
        // `Bot1.Undo`: the latest mana ability the player activated is taken back, and they are asked again.
        if let Some(ScriptStep::Action(step)) = self.current()
            && matches!(step.verb, ScriptVerb::Undo)
            && request.player == player_of(step.bot)
            && answers(step.verb, &request.kind)
        {
            if self.undo.offered(request.player).is_none() {
                return fail(format!("line {}: {} has no mana ability to take back", step.line, bot(request.player)));
            }
            return Ok(Decision { intent: Intent::Undo, line: Some(step) });
        }

        let paying = self.casting.as_ref().is_some_and(|c| c.player == request.player)
            && matches!(request.kind, RequestKind::PayCosts { .. });
        if !paying {
            if matches!(request.kind, RequestKind::Priority { .. }) {
                self.casting = None;
            }
        } else if let Some(auto) = self.auto_pay(request)? {
            return Ok(Decision { intent: Intent::Answer(auto), line: None });
        }

        if let Some(ScriptStep::Action(step)) = self.current()
            && request.player == player_of(step.bot)
            && let Some(answer) = self.matches(step, request)?
        {
            return Ok(Decision { intent: Intent::Answer(answer), line: Some(step) });
        }

        Ok(Decision { intent: Intent::Answer(self.default(request)?), line: None })
    }

    /// Gives the answer `intend` decided on, and moves the cursor past its line.
    fn carry_out(&mut self, request: &Request, decision: Decision<'s>) -> R<()> {
        let answer = match decision.intent {
            Intent::Undo => {
                let step = decision.line.expect("an Undo line");
                if self.undo.take_back(&mut self.game, request.player).is_none() {
                    return fail(format!("line {}: {} has no mana ability to take back", step.line, bot(request.player)));
                }
                self.trace.push(format!("T{} {}: line {} {}", self.turn(), bot(request.player), step.line, step.text));
                self.next();
                return Ok(());
            }
            Intent::Answer(answer) => answer,
        };
        let Some(step) = decision.line else {
            return self.deliver(request, answer, None);
        };
        self.next();
        self.deliver(request, answer.clone(), Some(step))?;
        let act = match &answer {
            Answer::Act(a) => Some(a.kind),
            _ => None,
        };
        match (&request.kind, act) {
            (RequestKind::Priority { .. }, Some(ActionKind::CastSpell | ActionKind::ActivateAbility)) => {
                self.colors = colors_of(step)?;
                let x = x_of(step)?;
                let mut casting = Casting::new(request.player, x, spend_of(step)?);
                casting.convoke = names_of(step, "Convoke");
                casting.delve = match step.named("Delve") {
                    None => 0,
                    Some(text) => match text.trim().parse::<u32>() {
                        Ok(n) => n,
                        Err(_) => return fail(format!("line {}: {text} is not a number", step.line)),
                    },
                };
                self.casting = Some(casting);
            }
            (RequestKind::UnlessPays { .. }, Some(ActionKind::PayCost)) => {
                self.casting = Some(Casting::new(request.player, None, spend_of(step)?));
            }
            (_, Some(ActionKind::ActivateAbility)) => {
                if self.casting.is_some() {
                    let more = spend_of(step)?;
                    self.casting.as_mut().unwrap().spend.extend(more);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Paying from the pool without being told (`AutoPay`): `Spend:` first, then any pool mana that pays what is owed.
    fn auto_pay(&mut self, request: &Request) -> R<Option<Answer>> {
        let RequestKind::PayCosts { remaining, options, announcing, x, .. } = &request.kind else { return Ok(None) };
        let offers = |kind: ActionKind| options.iter().find(|o| o.kind == kind);
        if *announcing {
            // X, or how many times it is kicked, raised or lowered to what the line said, then confirmed.
            let want = self.casting.as_ref().and_then(|c| c.x).unwrap_or(0);
            if *x < want && let Some(o) = offers(ActionKind::IncreaseX) {
                return Ok(Some(Answer::Act(Action::of(o))));
            }
            if *x > want && let Some(o) = offers(ActionKind::DecreaseX) {
                return Ok(Some(Answer::Act(Action::of(o))));
            }
            if *x == want && let Some(o) = offers(ActionKind::ConfirmCasting) {
                return Ok(Some(Answer::Act(Action::of(o))));
            }
            return Ok(None);
        }
        // Convoke taps the creatures it names, one per question, and Delve exiles that many cards from the graveyard, the
        // first offered each time: what pays the cost instead of mana (CR 601.2h).
        if let Some(casting) = self.casting.as_mut() {
            if !casting.convoke.is_empty() && options.iter().any(|o| o.kind == ActionKind::TapToPay) {
                let wanted = casting.convoke.pop_front().unwrap();
                let tap = options.iter().find(|o| {
                    o.kind == ActionKind::TapToPay && o.source.is_some_and(|s| self.game.name(s).eq_ignore_ascii_case(&wanted))
                });
                return match tap {
                    Some(o) => Ok(Some(Answer::Act(Action::of(o)))),
                    None => fail(format!(
                        "Convoke: names {wanted}, but the creatures offered are {:?}",
                        options.iter().filter(|o| o.kind == ActionKind::TapToPay).filter_map(|o| o.source).map(|s| self.game.name(s)).collect::<Vec<_>>()
                    )),
                };
            }
            if casting.delve > 0
                && let Some(o) = options.iter().find(|o| o.kind == ActionKind::ExileToPay)
            {
                casting.delve -= 1;
                return Ok(Some(Answer::Act(Action::of(o))));
            }
        }
        let assign: Vec<&ActionOption> = options.iter().filter(|o| o.kind == ActionKind::AssignMana).collect();
        if let Some(casting) = self.casting.as_mut()
            && !assign.is_empty()
            && let Some(wanted) = casting.spend.pop_front()
        {
            return match assign.iter().find(|o| (o.mana, o.restriction) == (Some(wanted.0), wanted.1)) {
                Some(o) => Ok(Some(Answer::Act(Action::of(o)))),
                None => fail(format!("Spend: names {wanted:?}, but the pool mana offered is {:?}", assign.iter().map(|o| o.mana).collect::<Vec<_>>())),
            };
        }
        if !remaining.is_zero()
            && let Some(o) = assign.iter().find(|o| o.mana.is_some_and(|m| pays(remaining, m)))
        {
            return Ok(Some(Answer::Act(Action::of(o))));
        }
        if let Some(o) = options.iter().find(|o| o.kind == ActionKind::ConfirmCasting) {
            return Ok(Some(Answer::Act(Action::of(o))));
        }
        Ok(None)
    }

    fn default(&mut self, request: &Request) -> R<Answer> {
        Ok(match &request.kind {
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::PayCosts { source, remaining, .. } => {
                if self.casting.is_none() {
                    self.casting = Some(Casting::new(request.player, None, VecDeque::new()));
                    if let Some(auto) = self.auto_pay(request)? {
                        return Ok(auto);
                    }
                    self.casting = None;
                }
                return fail(format!(
                    "{} is paying for {} with {remaining} still owed, and the next line does not pay it",
                    bot(request.player),
                    self.game.name(*source)
                ));
            }
            RequestKind::UnlessPays { options, .. } => {
                if let Some(ScriptStep::Action(step)) = self.current()
                    && step.verb == ScriptVerb::PayCost
                    && player_of(step.bot) == request.player
                {
                    return fail(format!("line {}: cannot pay yet: {}", step.line, step.text));
                }
                let decline = options.iter().find(|o| o.kind == ActionKind::DeclineCost).expect("decline offered");
                Answer::Act(Action::of(decline))
            }
            RequestKind::May { .. } => Answer::No,
            RequestKind::ChoosePlayer { candidates, .. } => Answer::Player(candidates[0]),
            // The first creatures offered, until their power is enough.
            RequestKind::TapForPower { cards, powers, minimum, .. } => {
                let mut total = 0u32;
                let mut chosen = Vec::new();
                for (&card, &power) in cards.iter().zip(powers) {
                    if total >= *minimum as u32 {
                        break;
                    }
                    total += power as u32;
                    chosen.push(card);
                }
                Answer::Cards(chosen)
            }
            // The first cards offered, until their mana value is enough.
            RequestKind::CollectEvidence { cards, values, minimum, .. } => {
                let mut total = 0u32;
                let mut chosen = Vec::new();
                for (&card, &value) in cards.iter().zip(values) {
                    if total >= *minimum as u32 {
                        break;
                    }
                    total += value as u32;
                    chosen.push(card);
                }
                Answer::Cards(chosen)
            }
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand[hand.len() - *count as usize..].to_vec()),
            RequestKind::ChooseCards { eligible, count, .. } => Answer::Cards(eligible.iter().take(*count as usize).copied().collect()),
            // A creature that attacks each combat if able attacks whether or not the script says so (CR 508.1d).
            RequestKind::DeclareAttackers { requirements, players, .. } => match players.first() {
                Some(&p) => Answer::Attack(requirements.iter().map(|&r| (r, Target::Player(p))).collect()),
                None => Answer::Attack(Vec::new()),
            },
            RequestKind::AssignCombatDamage { divisions } => Answer::Shares(DamageDivision::suggested_shares(divisions)),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            // The first eligible modes, the affordable ones first where the question advises (`FirstEligible`).
            // The first eligible mode as many times as are wanted, where a mode may be chosen more than once.
            RequestKind::ChooseModes { eligible, count, repeat: true, .. } => {
                Answer::Modes((0..*count as usize).map(|i| eligible[i.min(eligible.len() - 1)]).collect())
            }
            RequestKind::ChooseModes { eligible, count, affordable, .. } => {
                let mut modes = eligible.clone();
                if !affordable.is_empty() {
                    modes.sort_by_key(|m| !affordable.contains(m));
                }
                Answer::Modes(modes.into_iter().take(*count as usize).collect())
            }
            RequestKind::ChooseTarget { candidates, optional, count, .. } => {
                if *optional {
                    Answer::Targets(Vec::new())
                } else {
                    Answer::Targets(candidates.iter().take(*count as usize).copied().collect())
                }
            }
            // The whole total on the first candidate, whatever the slot allows: a legal division, not a good one.
            RequestKind::ChooseDividedTargets { candidates, total, .. } => {
                Answer::Divided(if *total > 0 && !candidates.is_empty() { vec![(candidates[0], *total)] } else { Vec::new() })
            }
            RequestKind::ChooseColors { choices, count, .. } => match self.colors.take() {
                Some(named) if named.len() == *count as usize && named.iter().all(|&c| choices.has(c)) => Answer::Colors(named),
                _ => Answer::Colors(choices.iter().take(*count as usize).collect()),
            },
            RequestKind::ChooseStartingPlayer { .. } => Answer::Player(request.player),
            RequestKind::Mulligan { .. } => Answer::No,
            RequestKind::OrderTriggers { sources, .. } => Answer::Order((0..sources.len() as u8).collect()),
            RequestKind::ChooseReplacement { .. } => Answer::Index(0),
            RequestKind::ChooseName { suggestion, .. } => Answer::Name(*suggestion),
            RequestKind::ChooseSubtype { choices, .. } => Answer::Subtype(choices[0]),
            RequestKind::DistributeCards { cards, destinations, .. } => {
                Answer::Placements { placements: greedy_distribution(cards, destinations), shuffle: false }
            }
            RequestKind::SearchLibrary { eligible, count, optional, .. } => {
                if *optional { Answer::Cards(Vec::new()) } else { Answer::Cards(eligible.iter().take(*count as usize).copied().collect()) }
            }
        })
    }

    fn matches(&self, step: &ActionStep, request: &Request) -> R<Option<Answer>> {
        use ScriptVerb as V;
        let options = match &request.kind {
            RequestKind::Priority { options } | RequestKind::PayCosts { options, .. } | RequestKind::UnlessPays { options, .. } => {
                Some(options)
            }
            _ => None,
        };
        Ok(match (step.verb, &request.kind) {
            // A cast an effect makes as it resolves (CR 608.2g) asks for its targets with the spell already on the
            // stack (CR 601.2a); the `Cast(...)` line that names it gives them.
            (V::Cast, RequestKind::ChooseTarget { source, purpose: Purpose::Targets, candidates, optional, count, .. })
                if self.game.state.stack.iter().any(|i| i.kind == StackKind::Spell && i.object == *source) =>
            {
                self.choose_target(step, candidates, *optional, *count)?
            }
            (V::Play | V::Cast | V::Activate | V::Foretell | V::Suspend, _) => match options {
                Some(options) => self.match_option(step, options)?,
                None => None,
            },
            (V::Pass, RequestKind::Priority { .. }) => Some(Answer::Pass),
            (V::PayCost, RequestKind::UnlessPays { options, .. }) => {
                options.iter().find(|o| o.kind == ActionKind::PayCost).map(|o| Answer::Act(Action::of(o)))
            }
            (V::Decline, RequestKind::UnlessPays { options, .. }) => {
                options.iter().find(|o| o.kind == ActionKind::DeclineCost).map(|o| Answer::Act(Action::of(o)))
            }
            (V::Decline, RequestKind::May { .. }) => Some(Answer::No),
            (V::Accept, RequestKind::May { .. }) => Some(Answer::Yes),
            (V::Cancel, RequestKind::PayCosts { options, .. }) => {
                options.iter().find(|o| o.kind == ActionKind::CancelCasting).map(|o| Answer::Act(Action::of(o)))
            }
            (V::Cancel, _) if request.may_cancel => Some(Answer::Cancel),
            (V::Choose, RequestKind::ChooseCards { eligible, count, up_to, .. }) => Some(self.pick(step, eligible, *count, !up_to)?),
            (V::Choose, RequestKind::TapForPower { cards, .. }) => Some(self.pick(step, cards, cards.len() as u8, false)?),
            (V::Choose, RequestKind::CollectEvidence { cards, .. }) => Some(self.pick(step, cards, cards.len() as u8, false)?),
            (V::ChoosePlayer, RequestKind::ChoosePlayer { candidates, .. }) => {
                let Some(spec) = step.positional().next() else {
                    return fail(format!("line {}: ChoosePlayer needs a player", step.line));
                };
                match bot_id(spec).map(player_of) {
                    Some(p) if candidates.contains(&p) => Some(Answer::Player(p)),
                    _ => return fail(format!("line {}: {spec} is not a player who may be chosen", step.line)),
                }
            }
            (V::Discard, RequestKind::DiscardToHandSize { hand, count }) => Some(self.pick(step, hand, *count, true)?),
            (V::Attack, RequestKind::DeclareAttackers { eligible, players, planeswalkers, battles, .. }) => {
                let both: Vec<ObjectId> = planeswalkers.iter().chain(battles.iter()).copied().collect();
                self.attack(step, eligible, players, &both)?
            },
            (V::Block, RequestKind::DeclareBlockers { eligible, attackers, allowed }) => self.block(step, eligible, attackers, allowed)?,
            (V::AssignDamage, RequestKind::AssignCombatDamage { divisions }) => self.assign_damage(step, divisions)?,
            // "Choose odd or even" is a name in the C# engine's scripts: `Name(Odd)`.
            (V::Modes | V::Name, RequestKind::ChooseModes { modes, eligible, count, at_most, .. }) => {
                Some(self.modes(step, modes, eligible, *count, (*at_most).max(*count))?)
            }
            (V::Target, RequestKind::ChooseTarget { candidates, optional, count, .. }) => {
                self.choose_target(step, candidates, *optional, *count)?
            }
            (V::Divide, RequestKind::ChooseDividedTargets { candidates, .. }) => Some(self.divide(step, candidates)?),
            (V::Colors, RequestKind::ChooseColors { choices, count, .. }) => {
                let mut colors = Vec::new();
                for spec in step.positional() {
                    match color_named(spec, false) {
                        Some(c) if choices.has(c) => colors.push(c),
                        _ => return fail(format!("line {}: {spec} is not a color on offer", step.line)),
                    }
                }
                if colors.len() != *count as usize {
                    return fail(format!("line {}: {} colors wanted", step.line, count));
                }
                Some(Answer::Colors(colors))
            }
            (V::PlayFirst, RequestKind::ChooseStartingPlayer { .. }) => Some(Answer::Player(request.player)),
            (V::DrawFirst, RequestKind::ChooseStartingPlayer { candidates }) => {
                let at = candidates.iter().position(|&p| p == request.player).unwrap_or(0);
                Some(Answer::Player(candidates[(at + 1) % candidates.len()]))
            }
            (V::Triggers, RequestKind::OrderTriggers { sources, abilities }) => Some(self.order_triggers(step, sources, abilities)?),
            (V::Replace, RequestKind::ChooseReplacement { sources, hints, .. }) => Some(self.replacement(step, sources, hints)?),
            (V::Arrange, RequestKind::DistributeCards { cards, destinations, may_shuffle, .. }) => {
                Some(self.arrange(step, cards, destinations, *may_shuffle)?)
            }
            (V::Search, RequestKind::SearchLibrary { eligible, count, optional, .. }) => {
                Some(self.pick(step, eligible, *count, !*optional)?)
            }
            // "Choose a creature type" is a modes question in the C# engine: `Modes(Elf)`.
            (V::Modes, RequestKind::ChooseSubtype { choices, .. }) => {
                let spec = step.positional().next().unwrap_or("").trim();
                match choices.iter().copied().find(|s| s.name().eq_ignore_ascii_case(spec)) {
                    Some(s) => Some(Answer::Subtype(s)),
                    None => return fail(format!("line {}: {spec} is not a type the question offers", step.line)),
                }
            }
            (V::Name, RequestKind::ChooseName { .. }) => {
                let spec = step.positional().next().unwrap_or("").trim();
                match cards().find(spec) {
                    Some(card) => Some(Answer::Name(Some(card))),
                    None => return fail(format!("line {}: no card is named {spec}", step.line)),
                }
            }
            (V::Mulligan, RequestKind::Mulligan { .. }) => Some(Answer::Yes),
            (V::Keep, RequestKind::Mulligan { .. }) => Some(Answer::No),
            _ => None,
        })
    }

    // --- Names ------------------------------------------------------------------------------------------------

    fn holder(&self, id: ObjectId) -> PlayerId {
        let obj = self.game.state.obj(id);
        if obj.zone.is(ZoneKind::Battlefield) || obj.zone.is(ZoneKind::Stack) { obj.controller } else { obj.owner }
    }

    /// A zone's objects as the C# engine lists them: libraries, graveyards and the stack top first.
    fn zone_objects(&self, kind: ZoneKind) -> Vec<ObjectId> {
        let s = &self.game.state;
        let mut ids: Vec<ObjectId> = match kind {
            ZoneKind::Battlefield => s.battlefield().to_vec(),
            ZoneKind::Stack => s.zone(Zone::STACK).iter().rev().copied().collect(),
            ZoneKind::Exile => {
                let mut all: Vec<ObjectId> = s.seats().flat_map(|p| s.zone(Zone::exile(p)).to_vec()).collect();
                all.sort();
                all
            }
            ZoneKind::Graveyard => s.seats().flat_map(|p| s.zone(Zone::graveyard(p)).iter().rev().copied().collect::<Vec<_>>()).collect(),
            ZoneKind::Library => s.seats().flat_map(|p| s.zone(Zone::library(p)).to_vec()).collect(),
            ZoneKind::Hand => s.seats().flat_map(|p| s.zone(Zone::hand(p)).to_vec()).collect(),
            ZoneKind::Command => s.zone(Zone::COMMAND).to_vec(),
            ZoneKind::Gone => Vec::new(),
        };
        ids.retain(|&id| live(s, id));
        ids
    }

    fn same_named(&self, kind: ZoneKind, name: &str, holder: PlayerId) -> Vec<ObjectId> {
        self.zone_objects(kind)
            .into_iter()
            .filter(|&id| self.holder(id) == holder && self.game.name(id).eq_ignore_ascii_case(name))
            .collect()
    }

    /// A mana ability the player has of their own (CR 605.1a, 611.2a), named by the card that gave it and no zone,
    /// `Bot1.Channel.Activate`: it has no source, so it is offered wherever that card has gone.
    fn is_players_ability(&self, step: &ActionStep, option: &ActionOption) -> bool {
        let (Some(source), Some(card)) = (option.source, &step.card) else { return false };
        option.index >= PLAYER_MANA_ABILITY
            && step.zone.is_none()
            && self.game.cards.get(self.game.state.obj(source).card).name.eq_ignore_ascii_case(card)
    }

    fn is_source(&self, step: &ActionStep, source: Option<ObjectId>) -> bool {
        let Some(id) = source else { return false };
        let s = &self.game.state;
        if !live(s, id) {
            return false;
        }
        let Some(card) = &step.card else { return false };
        if !self.game.name(id).eq_ignore_ascii_case(card) {
            return false;
        }
        let zone = s.obj(id).zone.kind();
        if let Some(z) = step.zone.and_then(engine_zone)
            && z != zone
        {
            return false;
        }
        if let Some(n) = step.copy {
            let list = self.same_named(zone, card, self.holder(id));
            return list.get(n - 1) == Some(&id);
        }
        true
    }

    /// The objects of `pool` a spec names (`Objects` in the C# runner).
    fn objects(&self, step: &ActionStep, spec: &str, pool: &[ObjectId]) -> R<Vec<ObjectId>> {
        let mut parts: Vec<&str> = spec.split('.').collect();
        let mut holder = None;
        let mut zone = None;
        if parts.len() > 1
            && let Some(b) = bot_id(parts[0])
        {
            holder = Some(player_of(b));
            parts.remove(0);
        }
        if parts.len() > 1
            && let Some(z) = parse::ZoneKind::parse(parts[0])
        {
            zone = engine_zone(z);
            parts.remove(0);
        }
        let mut name = parts.join(".").trim().to_string();
        let mut copy = None;
        if let Some((base, n)) = split_copy(&name) {
            name = base;
            copy = Some(n);
        }
        let known = card(&name).is_some() || self.game.state.objects.iter().enumerate().any(|(i, _)| self.game.name(ObjectId(i as u32)).eq_ignore_ascii_case(&name));
        if !known {
            return fail(format!("line {}: no card is named \"{name}\"", step.line));
        }
        let s = &self.game.state;
        Ok(pool
            .iter()
            .copied()
            .filter(|&id| {
                if !live(s, id) || !self.game.name(id).eq_ignore_ascii_case(&name) {
                    return false;
                }
                let kind = s.obj(id).zone.kind();
                if zone.is_some_and(|z| z != kind) || holder.is_some_and(|h| h != self.holder(id)) {
                    return false;
                }
                match copy {
                    Some(n) => self.same_named(kind, &name, self.holder(id)).get(n - 1) == Some(&id),
                    None => true,
                }
            })
            .collect())
    }

    fn target(&self, step: &ActionStep, spec: &str, candidates: &[Target]) -> R<Option<Target>> {
        if let Some(b) = bot_id(spec) {
            let p = player_of(b);
            return Ok(candidates.iter().copied().find(|&t| t == Target::Player(p)));
        }
        // An ability on the stack is named for its source, as `Stack.Name`; the topmost of that name.
        if let Some((zone, name)) = spec.split_once('.')
            && zone.eq_ignore_ascii_case("Stack")
        {
            let stack = &self.game.state.stack;
            let found = candidates.iter().copied().filter(|t| matches!(t, Target::Ability(_))).rev().find(|t| {
                matches!(t, Target::Ability(n) if stack.iter().any(|i| i.serial == *n && self.game.name(i.object).eq_ignore_ascii_case(name.trim())))
            });
            if found.is_some() {
                return Ok(found);
            }
        }
        let pool: Vec<ObjectId> = candidates.iter().filter_map(|t| t.object()).collect();
        let found = self.objects(step, spec, &pool)?;
        let Some(&first) = found.first() else { return Ok(None) };
        let place = |id: ObjectId| (self.game.state.obj(id).zone.kind(), self.holder(id));
        if found.iter().any(|&id| place(id) != place(first)) {
            return fail(format!("line {}: {spec} could be more than one card; write the one meant", step.line));
        }
        Ok(Some(Target::Object(first)))
    }

    fn pick(&self, step: &ActionStep, pool: &[ObjectId], count: u8, exactly: bool) -> R<Answer> {
        let mut chosen: Vec<ObjectId> = Vec::new();
        for spec in step.positional() {
            let found = self.objects(step, spec, pool)?;
            match found.into_iter().find(|id| !chosen.contains(id)) {
                Some(id) => chosen.push(id),
                None => return fail(format!("line {}: {spec} is not among the cards that may be chosen", step.line)),
            }
        }
        if (exactly && chosen.len() != count as usize) || chosen.len() > count as usize {
            return fail(format!("line {}: {} cards wanted, the line names {}", step.line, count, chosen.len()));
        }
        Ok(Answer::Cards(chosen))
    }

    /// The refused block `Blocker > Attacker` (see the loop above) when no blockers question was asked in the step.
    fn check_block_never_asked(&self, action: &ActionStep, line: usize, text: &str) -> R<()> {
        let defender = player_of(action.bot);
        let battlefield: Vec<ObjectId> = self.game.state.battlefield().to_vec();
        for spec in action.positional() {
            let parts: Vec<&str> = spec.split('>').collect();
            if parts.len() != 2 {
                return fail(format!("line {line}: write a refused block as \"Blocker > Attacker\""));
            }
            let blockers: Vec<ObjectId> = self
                .objects(action, parts[0].trim(), &battlefield)?
                .into_iter()
                .filter(|&b| self.game.state.obj(b).controller == defender && !self.game.state.obj(b).tapped && self.game.chars(b).is(CardType::Creature))
                .collect();
            let attacking: Vec<ObjectId> = self
                .game
                .state
                .attackers
                .iter()
                .filter(|a| matches!(a.target, Target::Player(p) if p == defender))
                .map(|a| a.id)
                .collect();
            let attackers = self.objects(action, parts[1].trim(), &attacking)?;
            if blockers.is_empty() || attackers.is_empty() {
                return fail(format!("line {line}: {text} proves nothing: the blocker must be an untapped creature of the defender and the attacker must be attacking that player"));
            }
            if self.game.state.blocks.iter().any(|b| blockers.contains(&b.blocker) && attackers.contains(&b.attacker)) {
                return fail(format!("line {line} says {text} must not be allowed, but the block was made"));
            }
        }
        Ok(())
    }

    /// `Blocker > Attacker` per block; only a pair the request allows.
    /// A refused block or attack (`! Bot2.Block(Bear > Ox)`, `! Bot1.Attack(Bear > Bot2.Battlefield.Karn)`): checked at the
    /// player's next declaration, failing if that pair is allowed. Both sides must be in the question, or the refusal
    /// proves nothing.
    fn check_refused_declaration(&self, action: &ActionStep, line: usize, text: &str, kind: &RequestKind) -> R<()> {
        match kind {
            RequestKind::DeclareBlockers { eligible, attackers, allowed } => {
                // The line names a whole declaration: an attacker that needs several blockers (menace, CR 702.110b) is not
                // legally blocked by fewer of them than the line gives it, however each pair stands alone.
                let mut given: Vec<(ObjectId, usize)> = Vec::new();
                for spec in action.positional() {
                    if let Some((_, atk)) = spec.split_once('>') {
                        for a in self.objects(action, atk.trim(), attackers)? {
                            match given.iter_mut().find(|g| g.0 == a) {
                                Some(g) => g.1 += 1,
                                None => given.push((a, 1)),
                            }
                        }
                    }
                }
                for spec in action.positional() {
                    let parts: Vec<&str> = spec.split('>').collect();
                    if parts.len() != 2 {
                        return fail(format!("line {line}: write a refused block as \"Blocker > Attacker\""));
                    }
                    let blockers = self.objects(action, parts[0].trim(), eligible)?;
                    let targets = self.objects(action, parts[1].trim(), attackers)?;
                    if targets.is_empty() {
                        return fail(format!("line {line}: {text} proves nothing: the attacker must be in the question"));
                    }
                    // A blocker left out of the question is a refusal only if it is an untapped creature of the player
                    // asked, so that it is an effect ("can't block") that keeps it out and not a tapped or absent creature.
                    if blockers.is_empty() {
                        let mine: Vec<ObjectId> = self
                            .game
                            .state
                            .battlefield()
                            .iter()
                            .copied()
                            .filter(|&id| !self.game.state.obj(id).tapped && self.game.state.obj(id).controller == player_of(action.bot))
                            .collect();
                        if self.objects(action, parts[0].trim(), &mine)?.is_empty() {
                            return fail(format!("line {line}: {text} proves nothing: the blocker must be an untapped creature of the defender"));
                        }
                        continue;
                    }
                    let short = targets.iter().any(|&a| {
                        given.iter().find(|g| g.0 == a).is_some_and(|g| (g.1 as u8) < self.game.chars(a).minimum_blockers())
                    });
                    if !short && blockers.iter().any(|&b| targets.iter().any(|&a| allowed.contains(&(b, a)))) {
                        return fail(format!("line {line} says {text} must not be allowed, but it is"));
                    }
                }
            }
            RequestKind::DeclareAttackers { eligible, players, planeswalkers, battles, .. } => {
                for spec in action.positional() {
                    let parts: Vec<&str> = spec.split('>').collect();
                    if parts.len() != 2 {
                        return fail(format!("line {line}: write a refused attack as \"Attacker > Target\""));
                    }
                    if self.objects(action, parts[0].trim(), eligible)?.is_empty() {
                        // An attacker the question leaves out is a refusal worth proving when only a restriction (Defender,
                        // "can't attack", summoning sickness) can be the reason: it is the asked player's own and untapped,
                        // and another creature of theirs is offered, so the declaration is being asked at all.
                        let on_battlefield: Vec<ObjectId> = self.game.state.battlefield().to_vec();
                        let named = self.objects(action, parts[0].trim(), &on_battlefield)?;
                        let player = player_of(action.bot);
                        let restricted = !eligible.is_empty()
                            && named.iter().any(|&id| {
                                let obj = self.game.state.obj(id);
                                self.game.controller(id) == player && !obj.tapped
                            });
                        if !restricted {
                            return fail(format!("line {line}: {text} proves nothing: the attacker must be able to attack"));
                        }
                        continue;
                    }
                    let on_battlefield: Vec<ObjectId> = self.game.state.battlefield().to_vec();
                    let offered = match bot_id(parts[1]) {
                        Some(b) => players.contains(&player_of(b)),
                        None => {
                            let found = self.objects(action, parts[1].trim(), &on_battlefield)?;
                            if found.is_empty() {
                                return fail(format!("line {line}: {text} proves nothing: nothing of that name to attack"));
                            }
                            found.iter().any(|f| planeswalkers.contains(f) || battles.contains(f))
                        }
                    };
                    if offered {
                        return fail(format!("line {line} says {text} must not be allowed, but it is"));
                    }
                }
            }
            RequestKind::ChooseModes { modes, eligible, count, at_most, .. } => {
                let mut chosen = Vec::new();
                for spec in action.positional() {
                    let index = match spec.trim().parse::<usize>() {
                        Ok(n) if n >= 1 => n - 1,
                        _ => match modes.iter().position(|m| label_text(m).to_lowercase().contains(&spec.trim().to_lowercase())) {
                            Some(i) => i,
                            None => return fail(format!("line {line}: no mode reads \"{spec}\"")),
                        },
                    };
                    if index >= modes.len() {
                        return fail(format!("line {line}: there is no mode {spec}"));
                    }
                    chosen.push(index as u8);
                }
                let allowed = chosen.len() >= *count as usize && chosen.len() <= *at_most as usize && chosen.iter().all(|m| eligible.contains(m));
                if allowed {
                    return fail(format!("line {line} says {text} must not be a choice the question allows, but it is"));
                }
            }
            RequestKind::ChooseTarget { candidates, .. } => {
                let everything: Vec<ObjectId> = (0..self.game.state.objects.len() as u32).map(ObjectId).collect();
                for spec in action.positional() {
                    if bot_id(spec).is_none() && self.objects(action, spec.trim(), &everything)?.is_empty() {
                        return fail(format!("line {line}: {text} proves nothing: no such card is in the game"));
                    }
                    if self.target(action, spec.trim(), candidates)?.is_some() {
                        return fail(format!("line {line} says {text} must not be a legal target, but it is"));
                    }
                }
            }
            RequestKind::ChooseCards { eligible, .. } => {
                let everything: Vec<ObjectId> = (0..self.game.state.objects.len() as u32).map(ObjectId).collect();
                for spec in action.positional() {
                    if self.objects(action, spec.trim(), &everything)?.is_empty() {
                        return fail(format!("line {line}: {text} proves nothing: no such card is in the game"));
                    }
                    if !self.objects(action, spec.trim(), eligible)?.is_empty() {
                        return fail(format!("line {line} says {text} must not be choosable, but it is"));
                    }
                }
            }
            RequestKind::ChooseColors { choices, .. } => {
                for spec in action.positional() {
                    match color_named(spec.trim(), false) {
                        Some(c) if choices.has(c) => {
                            return fail(format!("line {line} says {text} must not be a color on offer, but it is"));
                        }
                        Some(_) => {}
                        None => return fail(format!("line {line}: {spec} is not a color")),
                    }
                }
            }
            RequestKind::SearchLibrary { library, eligible, .. } => {
                for spec in action.positional() {
                    let in_library = self.objects(action, spec.trim(), library)?;
                    if in_library.is_empty() {
                        return fail(format!("line {line}: {text} proves nothing: no such card is in the library searched"));
                    }
                    if in_library.iter().any(|c| eligible.contains(c)) {
                        return fail(format!("line {line} says {text} must not be findable, but it is"));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn block(
        &self,
        step: &ActionStep,
        eligible: &[ObjectId],
        attackers: &[ObjectId],
        allowed: &[(ObjectId, ObjectId)],
    ) -> R<Option<Answer>> {
        let mut blocks: Vec<(ObjectId, ObjectId)> = Vec::new();
        for spec in step.positional() {
            let parts: Vec<&str> = spec.split('>').collect();
            if parts.len() != 2 {
                return fail(format!("line {}: write a block as \"Blocker > Attacker\", not \"{spec}\"", step.line));
            }
            let blockers: Vec<ObjectId> =
                self.objects(step, parts[0].trim(), eligible)?.into_iter().filter(|b| !blocks.iter().any(|x| x.0 == *b)).collect();
            let Some(attacker) = self.objects(step, parts[1].trim(), attackers)?.into_iter().next() else { return Ok(None) };
            if blockers.is_empty() {
                return Ok(None);
            }
            let Some(blocker) = blockers.into_iter().find(|&b| allowed.contains(&(b, attacker))) else {
                return fail(format!("line {}: no {} may block {}", step.line, parts[0].trim(), parts[1].trim()));
            };
            blocks.push((blocker, attacker));
        }
        Ok(Some(Answer::Block(blocks)))
    }

    /// `Source > Creature: n` per share, or `Source > BotN: n` for the player a trampler attacks; a creature the line
    /// leaves out keeps the engine's own division.
    fn assign_damage(&self, step: &ActionStep, divisions: &[DamageDivision]) -> R<Option<Answer>> {
        let mut shares: Vec<(ObjectId, Option<ObjectId>, u16)> = Vec::new();
        let mut written: Vec<ObjectId> = Vec::new();
        for spec in step.positional() {
            let parts: Vec<&str> = spec.split('>').collect();
            let parsed = (parts.len() == 2)
                .then(|| parts[1].rfind(':').map(|c| (parts[1][..c].trim(), parts[1][c + 1..].trim().parse::<u16>())))
                .flatten();
            let Some((to, Ok(amount))) = parsed else {
                return fail(format!(
                    "line {}: write a share as \"Source > Creature: amount\" or \"Source > Bot2: amount\", not \"{spec}\"",
                    step.line
                ));
            };
            let source_name = parts[0].trim();
            let mut division = None;
            for d in divisions {
                if !self.objects(step, source_name, &[d.source])?.is_empty() {
                    division = Some(d);
                    break;
                }
            }
            let Some(d) = division else { return Ok(None) };
            let creature = match bot_id(to) {
                Some(b) => {
                    if !d.may_assign_to_defender || d.defender != Target::Player(player_of(b)) {
                        return fail(format!("line {}: {source_name} cannot assign damage to {to}", step.line));
                    }
                    None
                }
                None => match self.objects(step, to, &d.creatures)?.into_iter().next() {
                    Some(c) => Some(c),
                    None => return Ok(None),
                },
            };
            written.push(d.source);
            shares.push((d.source, creature, amount));
        }
        for share in DamageDivision::suggested_shares(divisions) {
            if !written.contains(&share.0) {
                shares.push(share);
            }
        }
        Ok(Some(Answer::Shares(shares)))
    }

    fn attack(&self, step: &ActionStep, eligible: &[ObjectId], players: &[PlayerId], battles: &[ObjectId]) -> R<Option<Answer>> {
        let mut attacks = Vec::new();
        for spec in step.positional() {
            let parts: Vec<&str> = spec.split('>').collect();
            let found = self.objects(step, parts[0].trim(), eligible)?;
            let Some(attacker) = found.into_iter().find(|id| !attacks.iter().any(|(a, _)| a == id)) else {
                return Ok(None);
            };
            let defender = if parts.len() > 1 {
                match bot_id(parts[1]) {
                    Some(b) if players.contains(&player_of(b)) => Target::Player(player_of(b)),
                    // A battle a defending player protects: named as any permanent is.
                    _ => match self.objects(step, parts[1].trim(), battles)?.first() {
                        Some(&battle) => Target::Object(battle),
                        None => return fail(format!("line {}: {} cannot be attacked", step.line, parts[1].trim())),
                    },
                }
            } else if players.len() == 1 {
                Target::Player(players[0])
            } else {
                return fail(format!("line {}: say who {} attacks", step.line, parts[0].trim()));
            };
            attacks.push((attacker, defender));
        }
        Ok(Some(Answer::Attack(attacks)))
    }

    fn modes(&self, step: &ActionStep, modes: &[Label], eligible: &[u8], count: u8, most: u8) -> R<Answer> {
        let mut chosen = Vec::new();
        for spec in step.positional() {
            let index = match spec.trim().parse::<usize>() {
                Ok(n) if n >= 1 => n - 1,
                _ => match modes.iter().position(|m| label_text(m).to_lowercase().contains(&spec.trim().to_lowercase())) {
                    Some(i) => i,
                    None => return fail(format!("line {}: no mode reads \"{spec}\"", step.line)),
                },
            };
            if index >= modes.len() {
                return fail(format!("line {}: there is no mode {spec}", step.line));
            }
            chosen.push(index as u8);
        }
        if chosen.len() < count as usize || chosen.len() > most as usize || !chosen.iter().all(|m| eligible.contains(m)) {
            return fail(format!("line {}: not a choice the question allows", step.line));
        }
        Ok(Answer::Modes(chosen))
    }

    /// `Divide(Guerrilla Gorilla: 1; Llanowar Elves: 1)`: each target named, with its share of the total.
    fn divide(&self, step: &ActionStep, candidates: &[Target]) -> R<Answer> {
        let mut shares = Vec::new();
        for piece in step.positional() {
            let Some((who, amount)) = piece.rsplit_once(':') else {
                return fail(format!("line {}: Divide wants \"target: amount\", not \"{piece}\"", step.line));
            };
            let Ok(amount) = amount.trim().parse::<u16>() else {
                return fail(format!("line {}: {} is not a share", step.line, amount.trim()));
            };
            match self.target(step, who.trim(), candidates)? {
                Some(t) => shares.push((t, amount)),
                None => return fail(format!("line {}: {} is not a target on offer", step.line, who.trim())),
            }
        }
        Ok(Answer::Divided(shares))
    }

    fn choose_target(&self, step: &ActionStep, candidates: &[Target], optional: bool, count: u8) -> R<Option<Answer>> {
        let specs: Vec<&str> = step.positional().collect();
        if specs.is_empty() {
            if !optional {
                return fail(format!("line {}: this needs a target", step.line));
            }
            return Ok(Some(Answer::Targets(Vec::new())));
        }
        if (optional && specs.len() > count as usize) || (!optional && specs.len() != count as usize) {
            return fail(format!("line {}: takes {count} target(s) but the line gives {}", step.line, specs.len()));
        }
        let mut targets = Vec::new();
        for spec in specs {
            match self.target(step, spec, candidates)? {
                Some(t) => targets.push(t),
                None => return Ok(None),
            }
        }
        Ok(Some(Answer::Targets(targets)))
    }

    /// `Arrange(A; B | C)`: one `|`-separated group per destination, in the order the question lists them, each
    /// card named where it goes; every card looked at must be placed. `Arrange(Shuffle)` shuffles where allowed.
    fn arrange(&self, step: &ActionStep, cards: &[ObjectId], destinations: &[Destination], may_shuffle: bool) -> R<Answer> {
        let raw = step.raw_arguments.trim();
        if raw.eq_ignore_ascii_case("shuffle") {
            if !may_shuffle {
                return fail(format!("line {}: shuffling is not allowed here", step.line));
            }
            return Ok(Answer::Placements { placements: vec![Vec::new(); destinations.len()], shuffle: true });
        }
        let groups: Vec<&str> = step.raw_arguments.split('|').collect();
        if groups.len() > destinations.len() {
            return fail(format!("line {}: {} groups for {} destinations", step.line, groups.len(), destinations.len()));
        }
        let mut placements: Vec<Vec<ObjectId>> = vec![Vec::new(); destinations.len()];
        for (i, group) in groups.iter().enumerate() {
            for piece in parse::ScriptArgument::split(group) {
                let found = self.objects(step, &piece.value, cards)?;
                match found.into_iter().find(|id| !placements.iter().any(|p| p.contains(id))) {
                    Some(id) => placements[i].push(id),
                    None => return fail(format!("line {}: {} is not among the cards looked at", step.line, piece.value)),
                }
            }
        }
        if let Some(problem) = distribution_problem(cards, destinations, &placements) {
            return fail(format!("line {}: {problem}", step.line));
        }
        Ok(Answer::Placements { placements, shuffle: false })
    }

    /// `Triggers(A; B)`: each trigger named by its source's name or part of its text, in the order they go on the
    /// stack; every one must be named.
    /// Which applicable replacement effect applies first (CR 616.1), named by its source card or by part of the hint
    /// that effect supplies.
    fn replacement(&self, step: &ActionStep, sources: &[ObjectId], hints: &[&'static str]) -> R<Answer> {
        let named: Vec<&str> = step.positional().collect();
        let offered = || {
            sources.iter().zip(hints).map(|(&s, h)| format!("{} ({h})", self.game.name(s))).collect::<Vec<_>>().join(" | ")
        };
        let [spec] = named[..] else {
            return fail(format!("line {}: Replace takes one effect, one of: {}", step.line, offered()));
        };
        let wanted = spec.trim().to_lowercase();
        let found = (0..sources.len())
            .find(|&i| self.game.name(sources[i]).to_lowercase() == wanted || hints[i].to_lowercase().contains(&wanted));
        match found {
            Some(i) => Ok(Answer::Index(i as u8)),
            None => fail(format!("line {}: \"{spec}\" is not one of: {}", step.line, offered())),
        }
    }

    fn order_triggers(&self, step: &ActionStep, sources: &[ObjectId], abilities: &[AbilityRef]) -> R<Answer> {
        let mut order: Vec<u8> = Vec::new();
        for spec in step.positional() {
            let wanted = spec.trim().to_lowercase();
            let found = (0..sources.len()).find(|&i| {
                !order.contains(&(i as u8))
                    && (self.game.name(sources[i]).to_lowercase() == wanted
                        || self.game.cards.ability(abilities[i]).text().to_lowercase().contains(&wanted))
            });
            match found {
                Some(i) => order.push(i as u8),
                None => return fail(format!("line {}: \"{spec}\" is not one of the triggers to order", step.line)),
            }
        }
        if order.len() != sources.len() {
            return fail(format!("line {}: Triggers takes every one of the {} triggers", step.line, sources.len()));
        }
        Ok(Answer::Order(order))
    }

    fn offered(&self, step: &ActionStep, options: &[ActionOption], every_cost: bool) -> R<Vec<ActionOption>> {
        let kind = match step.verb {
            ScriptVerb::Play => ActionKind::PlayLand,
            ScriptVerb::Cast => ActionKind::CastSpell,
            ScriptVerb::Activate => ActionKind::ActivateAbility,
            // A special action (CR 116.2) is an ability of the card that is done at once, named by its keyword.
            ScriptVerb::Foretell | ScriptVerb::Suspend => ActionKind::ActivateAbility,
            _ => return Ok(Vec::new()),
        };
        let g = &self.game;
        let mut offered: Vec<ActionOption> =
            options.iter().filter(|o| o.kind == kind && (self.is_source(step, o.source) || self.is_players_ability(step, o))).cloned().collect();
        if matches!(step.verb, ScriptVerb::Foretell | ScriptVerb::Suspend) {
            let word = step.verb.name();
            offered.retain(|o| o.source.is_some_and(|s| {
                g.chars(s).abilities().nth(o.index as usize).is_some_and(|a| {
                    matches!(g.cards.ability(a), AbilityDef::Activated(d) if d.special_action && d.text.starts_with(word))
                })
            }));
        }
        if let Some(ability) = step.named("Ability") {
            let needle = ability.to_lowercase();
            if let Some(n) = needle.strip_prefix('#').and_then(|n| n.parse::<usize>().ok()) {
                offered.retain(|o| o.index as usize == n - 1);
            } else {
                offered.retain(|o| {
                    // An Adventure says which half of the card is cast, as the C# engine's description does.
                    let text = describe(g.cards, &g.state, o);
                    let text = match o.index {
                        WAY_ADVENTURE => format!("as its Adventure, {text}"),
                        WAY_BACK_FACE => format!("as its back face, {text}"),
                        _ => text.to_string(),
                    };
                    text.to_lowercase().contains(&needle)
                });
            }
        } else if kind == ActionKind::CastSpell && !every_cost {
            offered.retain(|o| o.index == 0);
        }
        // A refused `Activate(Color: Green)` is offered only where green is among the colors the ability may add; a
        // refused `Activate(Color: Colorless)`, only where colorless is among the mana it may add.
        if every_cost && let Some(text) = step.named("Color") {
            let colorless = is_colorless(text);
            let color = if colorless { None } else { color_named(text, true) };
            if colorless || color.is_some() {
                offered.retain(|o| self.may_add(o, color));
            }
        }
        Ok(offered)
    }

    /// Whether an offered mana ability may add mana of `color` (colorless for `None`): one that adds a color of the
    /// player's choosing, from its choices; one that adds fixed mana, from what it adds. Anything else is not judged.
    fn may_add(&self, option: &ActionOption, color: Option<Color>) -> bool {
        if !option.color_choices.is_empty() {
            return color.is_some_and(|c| option.color_choices.has(c)) || (color.is_none() && option.mana == Some(Mana::C));
        }
        let g = &self.game;
        let Some(source) = option.source else { return true };
        let Some(ability) = g.chars(source).abilities().nth(option.index as usize) else { return true };
        match g.cards.ability(ability) {
            AbilityDef::Activated(d) if d.mana.is_some() && d.adds.total() > 0 => {
                d.adds.get(color.map_or(Mana::C, |c| c.mana())) > 0
            }
            // An ability that adds no mana (a land's sacrifice-to-draw) is not one that adds this color.
            AbilityDef::Activated(d) if d.mana.is_none() => false,
            _ => true,
        }
    }

    fn is_mana_ability(&self, option: &ActionOption) -> bool {
        let Some(source) = option.source else { return false };
        let g = &self.game;
        let ability = g.chars(source).abilities().nth(option.index as usize);
        matches!(ability.map(|a| g.cards.ability(a)), Some(AbilityDef::Activated(d)) if d.mana.is_some())
    }

    fn match_option(&self, step: &ActionStep, options: &[ActionOption]) -> R<Option<Answer>> {
        let offered = self.offered(step, options, false)?;
        let Some(first) = offered.first() else { return Ok(None) };
        let on_source: Vec<&ActionOption> = offered.iter().filter(|o| o.source == first.source).collect();
        let mut indices: Vec<u8> = on_source.iter().map(|o| o.index).collect();
        indices.dedup();
        if indices.len() > 1 {
            return fail(format!(
                "line {}: {} has {} abilities to activate; say which with Ability: ...",
                step.line,
                step.card.as_deref().unwrap_or(""),
                indices.len()
            ));
        }
        let option = on_source[0];
        // A mana ability's sacrifice comes with the activation: `Sacrifice:` names it, a lone candidate is taken.
        if option.kind == ActionKind::ActivateAbility && option.targets_required > 0 && self.is_mana_ability(option) {
            let targets = match step.named("Sacrifice").or_else(|| step.named("Tap")) {
                Some(name) => match self.target(step, name, &option.target_candidates)? {
                    Some(t) => vec![t],
                    None => return fail(format!("line {}: {} cannot sacrifice {name}", step.line, step.text)),
                },
                None if option.target_candidates.len() == option.targets_required as usize => option.target_candidates.clone(),
                None => {
                    return fail(format!(
                        "line {}: {} sacrifices one of several permanents; add Sacrifice: (a card name)",
                        step.line, step.text
                    ));
                }
            };
            let mut action = Action::of(option);
            action.targets = targets;
            // The option's own `mana` says colorless may be named; only naming it puts it in the action (CR 106.7).
            action.mana = None;
            action.color = match step.named("Color") {
                Some(text) if is_colorless(text) => {
                    action.mana = Some(Mana::C);
                    None
                }
                Some(text) => match color_named(text, true) {
                    Some(c) => Some(c),
                    None => return fail(format!("line {}: {text} is not a color", step.line)),
                },
                None => None,
            };
            return Ok(Some(Answer::Act(action)));
        }
        let specs: Vec<&str> = step
            .positional()
            .chain(step.arguments.iter().filter(|a| a.key.as_deref() == Some("Target")).map(|a| a.value.as_str()))
            .collect();
        if option.targets_required > 0 && !option.targets_optional && specs.is_empty() {
            return fail(format!("line {}: {} needs a target", step.line, step.text));
        }
        if option.targets_required == 0 && !specs.is_empty() {
            return fail(format!("line {}: {} takes no target", step.line, step.text));
        }
        let mut targets = Vec::new();
        for spec in specs {
            match self.target(step, spec, &option.target_candidates)? {
                Some(t) => targets.push(t),
                None => return Ok(None),
            }
        }
        let color = match step.named("Color") {
            Some(text) => {
                if option.color_choices.is_empty() {
                    return fail(format!("line {}: {} adds no mana of a chosen color", step.line, step.text));
                }
                match color_named(text, true) {
                    Some(c) => Some(c),
                    None => return fail(format!("line {}: {text} is not a color", step.line)),
                }
            }
            None => {
                if !option.color_choices.is_empty() {
                    return fail(format!("line {}: {} adds mana of a color it has to be told; add Color:", step.line, step.text));
                }
                None
            }
        };
        let mut action = Action::of(option);
        action.mana = None;
        action.targets = targets;
        action.color = color;
        Ok(Some(Answer::Act(action)))
    }
}

/// The X or kicker count a line names (`X: 3`, `Kicked: 2`).
fn x_of(step: &ActionStep) -> R<Option<u16>> {
    match step.named("X").or_else(|| step.named("Kicked")) {
        None => Ok(None),
        Some(text) => match text.trim().parse::<u16>() {
            Ok(n) => Ok(Some(n)),
            Err(_) => fail(format!("line {}: {text} is not a number", step.line)),
        },
    }
}

/// The comma-separated names a line's key lists (`Convoke: Llanowar Elves, Birds of Paradise`).
fn names_of(step: &ActionStep, key: &str) -> VecDeque<String> {
    step.named(key).map(|t| t.split(',').map(|n| n.trim().to_string()).filter(|n| !n.is_empty()).collect()).unwrap_or_default()
}

/// The colors a line names with `Colors:` (`Colors: Red, Red, Green`).
fn colors_of(step: &ActionStep) -> R<Option<Vec<Color>>> {
    let Some(text) = step.named("Colors") else { return Ok(None) };
    let mut out = Vec::new();
    for piece in text.split(',') {
        match color_named(piece.trim(), true) {
            Some(c) => out.push(c),
            None => return fail(format!("line {}: Colors: {piece} is not a color", step.line)),
        }
    }
    Ok(Some(out))
}

/// `Spend:` names pool mana by its symbol, and for mana that carries a restriction its name after it: `R`, `R GainsHaste`.
/// A restriction's name has a comma in it (`LegendarySpells, CantBeCountered`), so pieces are split where a symbol
/// begins.
fn spend_of(step: &ActionStep) -> R<VecDeque<(Mana, Option<ManaRestriction>)>> {
    let mut out = VecDeque::new();
    if let Some(text) = step.named("Spend") {
        let mut pieces: Vec<String> = Vec::new();
        for piece in text.split(',') {
            let piece = piece.trim();
            if piece.split_whitespace().next().is_some_and(|w| mana_symbol(w).is_some()) || pieces.is_empty() {
                pieces.push(piece.to_string());
            } else if let Some(last) = pieces.last_mut() {
                last.push_str(", ");
                last.push_str(piece);
            }
        }
        for piece in pieces {
            let mut words = piece.split_whitespace();
            let Some(mana) = words.next().and_then(mana_symbol) else {
                return fail(format!("line {}: Spend: {piece} is not a kind of mana", step.line));
            };
            let rest: Vec<&str> = words.collect();
            let restriction = if rest.is_empty() {
                None
            } else {
                match ManaRestriction::parse(rest.into_iter()) {
                    Some(r) => Some(r),
                    None => return fail(format!("line {}: Spend: {piece} names something that is not a mana restriction", step.line)),
                }
            };
            out.push_back((mana, restriction));
        }
    }
    Ok(out)
}

