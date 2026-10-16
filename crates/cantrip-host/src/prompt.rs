//! The engine's requests as the wire's prompts, in a seat's own handle space.
//!
//! The native requests mirror the C# prompts kind for kind, so most of this is a field-for-field transcription. The
//! mapping, request by request (docs/native-host.md in the report has the same table):
//!
//! | native `RequestKind`   | wire prompt                                                                        |
//! |------------------------|------------------------------------------------------------------------------------|
//! | `Priority`             | `PriorityPromptMessage`                                                            |
//! | `PayCosts`             | `PayCostsPromptMessage` (announcing X/kicks sets `AnnouncesXOrKicks`)              |
//! | `UnlessPays`           | `UnlessPaysPromptMessage`                                                          |
//! | `ChooseCards`          | `ChooseCardsPromptMessage`                                                         |
//! | `TapForPower`          | `ChooseCardsPromptMessage` (purpose `TapToPay`, `Powers`, `MinimumPower`; harmonize's: `TapToReduce`, up to one) |
//! | `ChooseTarget`         | `ChooseTargetPromptMessage`                                                        |
//! | `ChooseModes`          | `ChooseModesPromptMessage`                                                         |
//! | `ChooseSubtype`        | `ChooseModesPromptMessage` over the subtypes (the C# engine asks it that way too)  |
//! | `ChooseColors`         | `ChooseColorsPromptMessage`                                                        |
//! | `ChooseName`           | `NameCardPromptMessage`                                                            |
//! | `May`                  | `MayPromptMessage` (the verb is the purpose's name)                                |
//! | `ChoosePlayer`         | `ChoosePlayerPromptMessage`                                                        |
//! | `ChooseDividedTargets` | `ChooseDividedTargetsPromptMessage`                                                |
//! | `DeclareAttackers`     | `DeclareAttackersPromptMessage`                                                    |
//! | `AssignCombatDamage`   | `AssignCombatDamagePromptMessage`                                                  |
//! | `DeclareBlockers`      | `DeclareBlockersPromptMessage`                                                     |
//! | `DiscardToHandSize`    | `DiscardPromptMessage`                                                             |
//! | `Mulligan`             | `MulliganPromptMessage`                                                            |
//! | `ChooseStartingPlayer` | `ChooseStartingPlayerPromptMessage`                                                |
//! | `OrderTriggers`        | `OrderTriggersPromptMessage`                                                       |
//! | `DistributeCards`      | `DistributeCardsPromptMessage`                                                     |
//! | `SearchLibrary`        | `SearchLibraryPromptMessage`                                                       |
//! | `ChooseReplacement`    | `ChooseReplacementPromptMessage`                                                   |

use std::collections::HashSet;

use cantrip_engine::*;

use crate::handles::HandleSpace;
use crate::protocol::*;
use crate::view::*;

/// The objects a request names, which its player must be able to see for as long as it is outstanding: a library being
/// searched, the cards looked at, an opponent's hand taken from. They are shown to that seat alone and the handles go
/// when the request does.
pub fn request_objects(request: &Request) -> HashSet<ObjectId> {
    let mut out: HashSet<ObjectId> = HashSet::new();
    let mut add = |ids: &[ObjectId]| out.extend(ids.iter().copied());
    match &request.kind {
        RequestKind::Priority { options } | RequestKind::UnlessPays { options, .. } => {
            add_options(&mut add, options);
            if let RequestKind::UnlessPays { source, subject, .. } = &request.kind {
                add(&[*source, *subject]);
            }
        }
        RequestKind::PayCosts { source, options, .. } => {
            add(&[*source]);
            add_options(&mut add, options);
        }
        RequestKind::ChooseCards { source, cards, eligible, .. } => {
            add(&[*source]);
            add(cards);
            add(eligible);
        }
        RequestKind::ChooseTarget { source, candidates, .. } | RequestKind::ChooseDividedTargets { source, candidates, .. } => {
            add(&[*source]);
            add(&candidates.iter().filter_map(|t| t.object()).collect::<Vec<_>>());
        }
        RequestKind::ChooseModes { source, .. }
        | RequestKind::ChooseColors { source, .. }
        | RequestKind::ChooseSubtype { source, .. }
        | RequestKind::May { source, .. }
        | RequestKind::ChoosePlayer { source, .. } => add(&[*source]),
        RequestKind::ChooseName { source, looked_at, .. } => {
            add(&[*source]);
            add(looked_at);
        }
        RequestKind::TapForPower { source, cards, .. } | RequestKind::CollectEvidence { source, cards, .. } => {
            add(&[*source]);
            add(cards);
        }
        RequestKind::DeclareAttackers { eligible, requirements, battles, planeswalkers, .. } => {
            add(battles);
            add(planeswalkers);
            add(eligible);
            add(requirements);
        }
        RequestKind::AssignCombatDamage { divisions } => {
            for d in divisions {
                add(&[d.source]);
                add(&d.creatures);
                if let Target::Object(id) = d.defender {
                    add(&[id]);
                }
            }
        }
        RequestKind::DeclareBlockers { eligible, attackers, allowed } => {
            add(eligible);
            add(attackers);
            add(&allowed.iter().flat_map(|(a, b)| [*a, *b]).collect::<Vec<_>>());
        }
        RequestKind::DiscardToHandSize { hand, .. } => add(hand),
        RequestKind::Mulligan { .. } | RequestKind::ChooseStartingPlayer { .. } => {}
        RequestKind::OrderTriggers { sources, .. } => add(sources),
        RequestKind::DistributeCards { source, cards, destinations, .. } => {
            add(&[*source]);
            add(cards);
            for d in destinations {
                add(&d.eligible);
            }
        }
        RequestKind::SearchLibrary { source, library, eligible, .. } => {
            add(&[*source]);
            add(library);
            add(eligible);
        }
        RequestKind::ChooseReplacement { subject, sources, .. } => {
            if let Some(s) = subject {
                add(&[*s]);
            }
            add(sources);
        }
    }
    out
}

fn add_options(add: &mut impl FnMut(&[ObjectId]), options: &[ActionOption]) {
    for o in options {
        if let Some(s) = o.source {
            add(&[s]);
        }
        add(&o.target_candidates.iter().filter_map(|t| t.object()).collect::<Vec<_>>());
    }
}

/// What a prompt may not say about an object that is gone: a source that has left and is no longer in the view is no
/// handle, and the client draws the ability from its text.
fn source_handle(game: &Game, handles: &mut HandleSpace, id: ObjectId) -> ViewHandle {
    let existing = handles.existing(id);
    if !existing.is_none() {
        existing
    } else if id.index() < game.state.objects.len() && live(&game.state, id) {
        handles.handle(id)
    } else {
        ViewHandle::NONE
    }
}

fn target_message(handles: &mut HandleSpace, target: Target) -> Option<TargetMessage> {
    match target {
        Target::Player(p) => Some(TargetMessage::player(seat(p))),
        Target::Object(id) => Some(TargetMessage::object(handles.handle(id))),
        Target::Ability(serial) => Some(TargetMessage::object(handles.handle(ability_key(serial)))),
        Target::None => None,
    }
}

pub(crate) fn action_kind_view(kind: ActionKind) -> ActionKindView {
    match kind {
        ActionKind::PlayLand => ActionKindView::PlayLand,
        ActionKind::CastSpell => ActionKindView::CastSpell,
        ActionKind::ActivateAbility => ActionKindView::ActivateAbility,
        ActionKind::CancelCasting => ActionKindView::CancelCasting,
        ActionKind::PayCost => ActionKindView::PayCost,
        ActionKind::DeclineCost => ActionKindView::DeclineCost,
        ActionKind::AssignMana => ActionKindView::AssignMana,
        ActionKind::ConfirmCasting => ActionKindView::ConfirmCasting,
        ActionKind::IncreaseX => ActionKindView::IncreaseX,
        ActionKind::DecreaseX => ActionKindView::DecreaseX,
        ActionKind::TapToPay => ActionKindView::TapToPay,
        ActionKind::ExileToPay => ActionKindView::ExileToPay,
    }
}

/// A cost as rules text, for a mana ability's row in a menu.
fn cost_text_of(cost: &Cost, name: &str) -> String {
    let mut parts = Vec::new();
    if cost.mana != ManaCost::ZERO {
        parts.push(cost.mana.to_string());
    }
    if cost.tap {
        parts.push("{T}".to_string());
    }
    if cost.life > 0 {
        parts.push(format!("Pay {} life", cost.life));
    }
    if cost.sacrifice_self {
        parts.push(format!("Sacrifice {name}"));
    }
    if cost.permanents.is_some() {
        parts.push("Sacrifice a permanent".to_string());
    }
    if cost.discard_self {
        parts.push("Discard".to_string());
    }
    parts.join(", ")
}

fn option_message(game: &Game, handles: &mut HandleSpace, o: &ActionOption) -> ActionOptionMessage {
    let source = o.source.map(|s| handles.handle(s)).unwrap_or(ViewHandle::NONE);
    let mut m = ActionOptionMessage::new(action_kind_view(o.kind), source);
    m.ability_index = i32::from(o.index);
    // A face-down permanent's one ability turns it face up (CR 701.40b).
    if o.kind == ActionKind::ActivateAbility
        && let Some(s) = o.source
        && game.chars(s).abilities().nth(o.index as usize).is_some_and(|a| a.card == game.cards.face_down)
    {
        m.kind = ActionKindView::TurnFaceUp;
    }
    // A Room's locked door is unlocked by paying its mana cost (CR 709.5e): its own kind, saying which door and what it costs.
    if o.kind == ActionKind::ActivateAbility
        && let Some(s) = o.source
        && let Some(a) = game.chars(s).abilities().nth(o.index as usize)
        && a.card == game.cards.intrinsic
        && matches!(a.index, intrinsic::UNLOCK_LEFT | intrinsic::UNLOCK_RIGHT)
    {
        let room = game.cards.get(game.chars(s).card);
        if let Some((left, right)) = room.split {
            let door = if a.index == intrinsic::UNLOCK_LEFT { left } else { right };
            m.kind = ActionKindView::UnlockDoor;
            m.cost_text = door.cost.to_string();
            m.spell_name = door.name.to_string();
        }
    }
    m.targets_required = i32::from(o.targets_required);
    m.targets_optional = o.targets_optional;
    m.color_choices = colors_view(o.color_choices);
    m.description = describe(game.cards, &game.state, o).to_string();
    if m.kind == ActionKindView::UnlockDoor {
        m.description = format!("Unlock {} {}", m.spell_name, m.cost_text);
    }
    if let Some(mana) = o.mana {
        m.produces = vec![mana_view(mana)];
    }
    // What restricts the mana: of pool mana to assign, or of the mana a mana ability adds (CR 106.6).
    if let Some(restriction) = o.restriction {
        (m.filter, m.effect) = restriction_view(restriction);
    }
    m.pays_nothing = o.pays_nothing;
    let candidates: Vec<TargetMessage> = o.target_candidates.iter().filter_map(|t| target_message(handles, *t)).collect();

    let Some(src) = o.source else {
        m.target_candidates = candidates;
        return m;
    };
    let def = game.cards.get(game.state.obj(src).card);
    match o.kind {
        ActionKind::CastSpell => {
            // Cast as its Adventure or its Omen (CR 715.3, 720.3), it is that spell: its name, and its own mana cost.
            let inset = def.inset_face(o.index);
            // A cast a permanent grants names the face it casts (a Room's door, an Adventure): its name and its cost.
            let granted_face = granted_slot(o.index).and_then(|(_, face)| match face {
                GRANTED_FACE_LEFT => def.split.map(|h| h.0),
                GRANTED_FACE_RIGHT => def.split.map(|h| h.1),
                GRANTED_FACE_ADVENTURE => def.adventure,
                GRANTED_FACE_OMEN => def.omen,
                GRANTED_FACE_BACK => def.back,
                _ => None,
            });
            if let Some(face) = granted_face.or(inset) {
                m.spell_name = face.name.to_string();
            }
            if let Some(face) = inset {
                m.cost_text = face.cost.to_string();
            }
            let cost = if let Some(face) = granted_face.or(inset) {
                face.cost
            } else if o.index == 0 || granted_slot(o.index).is_some() {
                def.cost
            } else {
                def.ways.get(o.index as usize - 1).map_or(def.cost, |w| w.cost.mana)
            };
            m.has_x = cost.x > 0;
            // A cost that depends on the target it is cast with (CR 601.2c, 601.2f): the row says what the cast costs with
            // each candidate, where that is not its mana cost.
            if o.index == 0 && o.targets_required == 1 && !o.target_candidates.is_empty() {
                let player = game.state.obj(src).controller;
                let costs = cast_costs(game, player, src, &o.target_candidates);
                if costs.iter().any(|&c| c != cost) {
                    let each = costs_by_target(game, &o.target_candidates, &costs);
                    m.description = if m.description.is_empty() { format!("costs {each}") } else { format!("{} (costs {each})", m.description) };
                }
            }
            m.target_candidates = candidates;
        }
        ActionKind::ActivateAbility => {
            let ability = game.chars(src).abilities().nth(o.index as usize).map(|a| game.cards.ability(a));
            if let Some(AbilityDef::Activated(d)) = ability {
                m.life_cost = i32::from(d.cost.life);
                if d.is_mana_ability() {
                    for mana in Mana::ALL {
                        for _ in 0..d.adds.get(mana) {
                            m.produces.push(mana_view(mana));
                        }
                    }
                    if !o.color_choices.is_empty() {
                        m.produces_of_any_one_color = i32::from(d.adds_any.max(1));
                    }
                    m.cost_text = cost_text_of(&d.cost, def.name);
                    // A mana ability's sacrifice comes with the activation: the engine takes it as a target.
                    if d.cost.permanents.is_some() && o.targets_required > 0 {
                        m.sacrifice_candidates = candidates.iter().map(|t| t.object).collect();
                        m.targets_required = 0;
                        return m;
                    }
                } else if o.targets_required == 1 && !o.target_candidates.is_empty() {
                    // A cost that depends on the target it is announced with (CR 601.2c, 601.2f, 602.2b): the row says what
                    // the activation costs with each candidate, where that is not the printed cost.
                    let player = game.state.obj(src).controller;
                    let costs = activation_costs(game, player, src, o.index, &o.target_candidates);
                    if costs.iter().any(|&c| c != d.cost.mana) {
                        m.description = format!("{} (costs {})", m.description, costs_by_target(game, &o.target_candidates, &costs));
                    }
                }
            }
            m.target_candidates = candidates;
        }
        ActionKind::TapToPay | ActionKind::ExileToPay => {
            m.alternative = i32::from(o.index);
            m.pays_up_to = if o.index == PAY_CONVOKE { 0 } else { i32::MAX.min(255) };
            m.target_candidates = candidates;
        }
        _ => m.target_candidates = candidates,
    }
    m
}

/// What an action costs with each candidate target, for its menu row: "{1} targeting Dwarven Mauler, {4} targeting
/// Ornithopter".
fn costs_by_target(game: &Game, candidates: &[Target], costs: &[ManaCost]) -> String {
    let each: Vec<String> = candidates
        .iter()
        .zip(costs)
        .map(|(t, cost)| match t.object() {
            Some(id) => format!("{cost} targeting {}", game.name(id)),
            None => format!("{cost} targeting a player"),
        })
        .collect();
    each.join(", ")
}

fn options_messages(game: &Game, handles: &mut HandleSpace, options: &[ActionOption]) -> Vec<ActionOptionMessage> {
    options.iter().map(|o| option_message(game, handles, o)).collect()
}

fn remaining_colored(cost: &ManaCost) -> Vec<ColorView> {
    let mut out = Vec::new();
    for color in Color::ALL {
        for _ in 0..cost.symbols[color.mana() as usize] {
            out.push(color_view(color));
        }
    }
    out
}

fn remaining_hybrid(cost: &ManaCost) -> Vec<HybridSymbolView> {
    let mut out = Vec::new();
    for (a, b) in cost.hybrid.iter().flatten() {
        out.push(HybridSymbolView { colors: [a, b].iter().filter_map(|m| m.color()).map(color_view).collect(), life_cost: 0 });
    }
    for (i, &n) in cost.phyrexian.iter().enumerate() {
        for _ in 0..n {
            out.push(HybridSymbolView { colors: vec![color_view(Color::ALL[i])], life_cost: 2 });
        }
    }
    out
}

fn purpose_view(purpose: Purpose, opening: bool) -> CardChoicePurposeView {
    use CardChoicePurposeView as V;
    match purpose {
        Purpose::Discard => {
            if opening {
                V::PutOnBottom
            } else {
                V::Discard
            }
        }
        Purpose::TakeFromHand => V::Discard,
        Purpose::Sacrifice | Purpose::SacrificeCost => V::Sacrifice,
        Purpose::ReturnToHand => V::ReturnToHand,
        Purpose::PutOnLibrary | Purpose::PutOnTop => V::PutOnTop,
        Purpose::PutOntoBattlefield => V::PutOntoBattlefield,
        Purpose::KeepLegend => V::KeepLegend,
        Purpose::Exile => V::Exile,
        Purpose::Untap => V::Untap,
        Purpose::ExileToPay => V::ExileToPay,
        Purpose::Keep => V::Keep,
        Purpose::PutIntoHand => V::PutIntoHand,
        Purpose::Cast => V::Cast,
        Purpose::CastFree => V::CastFree,
        Purpose::CastCopyFree | Purpose::CastCopyFreeWithin(_) => V::CastCopyFree,
        Purpose::TapToPay => V::TapToPay,
        Purpose::OrderSimultaneous => V::OrderSimultaneous,
        Purpose::Destroy => V::Destroy,
        Purpose::Vote => V::Vote,
        Purpose::PutIntoGraveyard => V::PutIntoGraveyard,
        Purpose::Play => V::Play,
        Purpose::Splice => V::Splice,
        Purpose::PutOnBottom => V::PutOnBottom,
        Purpose::Attach => V::Attach,
        Purpose::Encode => V::Encode,
        Purpose::Behold => V::Behold,
        Purpose::Blight => V::Blight,
        Purpose::TapToReduce => V::TapToReduce,
        Purpose::PutCounters(..) => V::PutCounters,
        Purpose::SeparatePiles
        | Purpose::ChoosePile
        | Purpose::CopyAsEnters
        | Purpose::CopyWhich
        | Purpose::Targets
        | Purpose::Color
        | Purpose::Optional
        | Purpose::Ask(_)
        | Purpose::EndTurn
        | Purpose::RevealForMiracle(_)
        | Purpose::CastForMiracleCost(_)
        | Purpose::PromiseGift(_)
        | Purpose::CastTransformedFree
        | Purpose::CastForMadness
        | Purpose::CastFoundOrHand
        | Purpose::CastOrBottom
        | Purpose::AlternativeCost => V::Keep,
    }
}

/// The costs a spell's card may ask a keyword action for: its optional cost, its additional cost, and each of its choices.
fn costs_of(game: &Game, source: ObjectId) -> Vec<Cost> {
    let def = game.cards.get(game.state.obj(source).card);
    def.optional_cost.map(|o| o.cost).into_iter().chain(def.additional_cost).chain(def.additional_choice.iter().map(|c| c.cost)).collect()
}

/// "Behold" or "Blight" for an additional cost that is that keyword action, else nothing.
fn cost_kind(game: &Game, source: ObjectId) -> &'static str {
    let costs = costs_of(game, source);
    if costs.iter().any(|c| c.behold.is_some()) {
        "Behold"
    } else if costs.iter().any(|c| c.blight > 0) {
        "Blight"
    } else {
        ""
    }
}

/// The quality a spell's behold cost names ("Dragon").
fn beheld_quality(game: &Game, source: ObjectId) -> &'static str {
    costs_of(game, source).iter().find_map(|c| c.behold.map(|(_, _, quality)| quality)).unwrap_or("")
}

/// The verb of a "you may" and its amount (`CastDuringResolution` with an amount of 1 is a cast without paying).
fn may_verb(purpose: Purpose) -> (String, i32) {
    let verb = match purpose {
        Purpose::Cast => "CastDuringResolution",
        Purpose::CastFree => return ("CastDuringResolution".into(), 1),
        Purpose::CastCopyFree | Purpose::CastCopyFreeWithin(_) => "CastCopyWithoutPaying",
        Purpose::CastTransformedFree => "CastTransformed",
        Purpose::CastForMadness => "CastForMadnessCost",
        Purpose::CastFoundOrHand => "CastFoundOrPutIntoHand",
        Purpose::CastOrBottom => "CastWithoutPaying",
        Purpose::CastForMiracleCost(_) => "CastForMiracleCost",
        Purpose::PromiseGift(gift) => match gift {
            Gift::Food => "PromiseGiftFood",
            Gift::Card => "PromiseGiftCard",
            Gift::TappedFish => "PromiseGiftTappedFish",
            Gift::ExtraTurn => "PromiseGiftExtraTurn",
            Gift::Treasure => "PromiseGiftTreasure",
            Gift::Octopus => "PromiseGiftOctopus",
        },
        Purpose::CopyAsEnters => "CopyAsEnters",
        other => return (format!("{other:?}"), 0),
    };
    (verb.into(), 0)
}

/// The modal spell or ability whose modes `modes` are the labels of.
fn modal_of(game: &Game, modes: &[Label]) -> Option<&'static ModalDef> {
    let Some(Label::Mode(ability, _)) = modes.first() else { return None };
    let def = game.cards.get(ability.card);
    if ability.index == u8::MAX {
        def.modal
    } else {
        match def.abilities.get(ability.index as usize) {
            Some(AbilityDef::Activated(a)) => a.modal,
            Some(AbilityDef::Triggered(t)) => t.modal,
            _ => None,
        }
    }
}

/// What a choice of modes says beyond its heading, and whether choosing more of them than the fewest costs something: a
/// modal's extra modes cost their escalate cost or their own cost (spree, CR 702.172a), the faces of a card are not modes
/// at all, and the labels of a pile or a plain choice are chosen without a target.
fn modes_detail(game: &Game, modes: &[Label], faces: bool) -> (String, bool) {
    if faces {
        return ("Only the face you choose is cast, with its own cost and mana value".into(), false);
    }
    match modal_of(game, modes) {
        Some(modal) => {
            if let Some(escalate) = modal.escalate {
                (format!("Each mode past the first costs {}", cost_text_of(&escalate, "")), true)
            } else if modal.modes.iter().any(|m| m.cost.mana_value() > 0) {
                ("Each mode adds the cost shown with it".into(), true)
            } else {
                (String::new(), false)
            }
        }
        None if modes.iter().all(|l| matches!(l, Label::Card(..) | Label::Name(_) | Label::Pile { .. })) => ("Chosen, not targeted".into(), false),
        None => (String::new(), false),
    }
}

/// A miracle cost as words: its symbols, or that it can't be paid (CR 118.6).
fn cost_words(cost: Option<ManaCost>) -> String {
    cost.map_or_else(|| "(unpayable: it has no mana cost)".to_string(), |c| c.to_string())
}

/// What a miracle ability costs, in words, from the card table alone (the card being drawn is no object yet when it is
/// asked about), reduced as its grant says: a split card's for each half it may be cast as, since a granted cost may be
/// read off the half (CR 709.3a), and what of a reduction is left to come off X (CR 118.7a).
fn miracle_cost_words(game: &Game, miracle: MiracleOf) -> String {
    let words = |face: CardId| match miracle_cost_and_x_less(game, miracle, face) {
        Some((cost, x_less)) if x_less > 0 => format!("{cost} (X less {x_less})"),
        cost => cost_words(cost.map(|(c, _)| c)),
    };
    match game.cards.get(miracle.card).split {
        Some((left, right)) if !miracle.is_own() => {
            [left, right].iter().map(|half| format!("{} {}", half.name, words(game.cards.id_of(half)))).collect::<Vec<_>>().join(" or ")
        }
        _ => words(miracle.card),
    }
}

/// One miracle ability of a card, for the choice of which to reveal it with (CR 702.94a, 607.5): its cost and whose it is.
fn miracle_label(game: &Game, miracle: MiracleOf) -> String {
    let cost = miracle_cost_words(game, miracle);
    if miracle.is_own() { format!("Miracle {cost}, its own") } else { format!("Miracle {cost}, from {}", game.cards.get(miracle.granter).name) }
}

/// A miracle ability as the questions about it word it: "miracle {2}", and who gave it, if not the card itself.
fn miracle_words(game: &Game, miracle: MiracleOf) -> (String, String) {
    let cost = format!("miracle {}", miracle_cost_words(game, miracle));
    let from = if miracle.is_own() { String::new() } else { format!("{} gives it {cost}", game.cards.get(miracle.granter).name) };
    (cost, from)
}

/// Miracle's two questions (CR 702.94a), worded with the cost: whether to reveal the card being drawn, and, as the linked
/// ability resolves, whether to cast it for the cost of the miracle ability it was revealed with. A cost a permanent gave
/// is not on the card's face, so the question says where it comes from.
fn miracle_prompt(game: &Game, player: i32, handle: ViewHandle, purpose: Purpose) -> MayPrompt {
    let seen = "Every player sees it from now on, even if you then decide not to cast it";
    let (verb, question, detail, yes, no, no_default) = match purpose {
        Purpose::RevealForMiracle(Some(one)) => {
            let (cost, from) = miracle_words(game, one);
            let detail = if from.is_empty() { seen.to_string() } else { format!("{from}  ·  {seen}") };
            ("RevealForMiracle", format!("Reveal {{source}} for its {cost}?"), detail, "Reveal", "Keep it hidden", true)
        }
        Purpose::RevealForMiracle(None) => (
            "RevealForMiracle",
            "Reveal {source} for one of its miracle abilities?".to_string(),
            format!("It has more than one: you choose which next  ·  {seen}"),
            "Reveal",
            "Keep it hidden",
            true,
        ),
        Purpose::CastForMiracleCost(miracle) => {
            let (cost, from) = miracle_words(game, miracle);
            let pay = format!("You pay its {cost} instead of its mana cost");
            let detail = if from.is_empty() { pay } else { format!("{pay}  ·  {from}") };
            ("CastForMiracleCost", format!("Cast {{source}} for its {cost}?"), detail, "Cast", "Don't cast", false)
        }
        _ => unreachable!("only miracle's questions are worded here"),
    };
    MayPrompt { player, source: handle, verb: verb.into(), amount: 0, question, detail, yes: yes.into(), no: no.into(), no_default }
}

/// What a card of the request is cast as, for the face choice: its name and its cost.
fn face_text(game: &Game, face: CardId) -> String {
    let def = game.cards.get(face);
    if def.has_cost { format!("{} {}", def.name, def.cost) } else { def.name.to_string() }
}

fn handles_of(handles: &mut HandleSpace, ids: &[ObjectId]) -> Vec<ViewHandle> {
    ids.iter().map(|&id| handles.handle(id)).collect()
}

/// The prompt a request is, and whether the seat may give the announcement up.
pub fn prompt(game: &Game, request: &Request, handles: &mut HandleSpace) -> Result<(PromptPayload, bool), String> {
    let player = seat(request.player);
    let opening = game.state.agenda.last().is_some_and(|f| f.kind == FrameKind::Opening);
    let payload = match &request.kind {
        RequestKind::Priority { options } => {
            PromptPayload::PriorityPromptMessage(PriorityPrompt { player, options: options_messages(game, handles, options) })
        }
        RequestKind::PayCosts { source, total, remaining, options, announcing, x } => {
            let def = game.cards.get(game.state.obj(*source).card);
            let kicker = def.kicker;
            let spends_no_mana = game.chars(*source).has(Keyword::CantSpendManaToCast);
            PromptPayload::PayCostsPromptMessage(PayCostsPrompt {
                player,
                spell: source_handle(game, handles, *source),
                total: total.to_string(),
                remaining: remaining.to_string(),
                remaining_generic: i32::from(remaining.generic),
                remaining_colored: remaining_colored(remaining),
                remaining_colorless: i32::from(remaining.symbols[Mana::C as usize]),
                remaining_hybrid: remaining_hybrid(remaining),
                options: options_messages(game, handles, options),
                has_x: total.x > 0 || (*announcing && kicker.is_some()),
                x: i32::from(*x),
                kicker: kicker.map_or(String::new(), |k| k.cost.to_string()),
                multikicker: kicker.is_some_and(|k| k.multi),
                kicker_paid: kicker.is_some() && *x > 0,
                announces_x_or_kicks: *announcing,
                most_affordable: -1,
                spends_no_mana,
                ..Default::default()
            })
        }
        RequestKind::UnlessPays { source, subject, total, options } => PromptPayload::UnlessPaysPromptMessage(UnlessPaysPrompt {
            player,
            source: source_handle(game, handles, *source),
            subject: source_handle(game, handles, *subject),
            total: total.to_string(),
            remaining: total.to_string(),
            remaining_generic: i32::from(total.generic),
            remaining_colored: remaining_colored(total),
            remaining_hybrid: remaining_hybrid(total),
            options: options_messages(game, handles, options),
            verb: "pay".into(),
        }),
        RequestKind::ChooseCards { source, purpose, cards, eligible, count, up_to, kind } => {
            let owner = if *purpose == Purpose::TakeFromHand {
                cards.first().map_or(request.player, |c| game.state.obj(*c).owner)
            } else {
                request.player
            };
            PromptPayload::ChooseCardsPromptMessage(ChooseCardsPrompt {
                player,
                owner: seat(owner),
                source: source_handle(game, handles, *source),
                purpose: purpose_view(*purpose, opening),
                cards: handles_of(handles, cards),
                eligible: handles_of(handles, eligible),
                count: i32::from(*count),
                up_to: *up_to,
                // Blight (CR 701.68a) is a creature you control; behold (CR 701.4a) names its quality ("Dragon"): the
                // client says so in its caption. Any other choice is worded as the asking card words it ("nonland"), if it does.
                kind: match purpose {
                    Purpose::Blight => "creature".into(),
                    Purpose::Behold if !kind.is_empty() => (*kind).to_string(),
                    Purpose::Behold => beheld_quality(game, *source).into(),
                    _ => (*kind).to_string(),
                },
                maximum_mana_value: match purpose {
                    Purpose::CastCopyFreeWithin(left) => i32::from(*left),
                    _ => 0,
                },
                // The counters the chosen permanent gets, for the caption ("put 2 loyalty counters on").
                counter: match purpose {
                    Purpose::PutCounters(counter, _) => counter.name().into(),
                    _ => String::new(),
                },
                counter_count: match purpose {
                    Purpose::PutCounters(_, n) => i32::from(*n),
                    _ => 0,
                },
                ..Default::default()
            })
        }
        RequestKind::TapForPower { source, purpose, cards, powers, minimum } => {
            // Harmonize's creature (CR 702.180a) is up to one, and `powers` is what tapping each takes off the cost.
            let reduce = *purpose == Purpose::TapToReduce;
            PromptPayload::ChooseCardsPromptMessage(ChooseCardsPrompt {
                player,
                owner: player,
                source: source_handle(game, handles, *source),
                purpose: if reduce { CardChoicePurposeView::TapToReduce } else { CardChoicePurposeView::TapToPay },
                cards: handles_of(handles, cards),
                eligible: handles_of(handles, cards),
                count: if reduce { 1 } else { cards.len() as i32 },
                kind: if reduce { "creature".into() } else { String::new() },
                up_to: true,
                minimum_power: i32::from(*minimum),
                powers: powers.iter().map(|&p| i32::from(p)).collect(),
                ..Default::default()
            })
        }
        RequestKind::CollectEvidence { source, cards, values, minimum } => {
            PromptPayload::ChooseCardsPromptMessage(ChooseCardsPrompt {
                player,
                owner: player,
                source: source_handle(game, handles, *source),
                purpose: CardChoicePurposeView::ExileToPay,
                cards: handles_of(handles, cards),
                eligible: handles_of(handles, cards),
                count: cards.len() as i32,
                up_to: true,
                minimum_mana_value: i32::from(*minimum),
                mana_values: values.iter().map(|&v| i32::from(v)).collect(),
                ..Default::default()
            })
        }
        RequestKind::ChooseTarget { source, purpose, candidates, optional, least, count, any_number, label, slot_player } => {
            let source_handle = source_handle(game, handles, *source);
            PromptPayload::ChooseTargetPromptMessage(ChooseTargetPrompt {
                player,
                ability: source_handle,
                source: source_handle,
                text: (*label).to_string(),
                candidates: candidates.iter().filter_map(|t| target_message(handles, *t)).collect(),
                optional: *optional,
                count: i32::from(*count),
                targeted: *purpose == Purpose::Targets,
                // Only a printed range has a minimum (the wire's zero is "exactly count, or none where optional").
                minimum: if *least > 0 && least < count { i32::from(*least) } else { 0 },
                any_number: *any_number,
                for_player: slot_player.map_or(0, seat),
            })
        }
        RequestKind::ChooseModes { source, modes, eligible, count, at_most, affordable, additional_cost, repeat } => {
            // The faces a card may be cast as (CR 709.3): not modes, so each is its name and its cost, and the question says so.
            let faces = !modes.is_empty() && modes.iter().all(|l| matches!(l, Label::Face(_)));
            let (detail, extra_modes_cost) = modes_detail(game, modes, faces);
            PromptPayload::ChooseModesPromptMessage(ChooseModesPrompt {
                player,
                source: source_handle(game, handles, *source),
                modes: modes
                    .iter()
                    .map(|l| match l {
                        Label::Face(face) => face_text(game, *face),
                        Label::Miracle(m) => miracle_label(game, *m),
                        other => label_text(other).to_string(),
                    })
                    .collect(),
                eligible: eligible.iter().map(|&m| i32::from(m)).collect(),
                count: i32::from(*count),
                may_repeat: *repeat,
                additional_cost: *additional_cost,
                heading: if faces {
                    "Choose which face to cast".into()
                } else if matches!(modes.first(), Some(Label::Miracle(..))) {
                    "Choose which miracle to reveal it with".into()
                } else {
                    String::new()
                },
                at_most: i32::from(*at_most),
                affordable: affordable.iter().map(|&m| i32::from(m)).collect(),
                cost_kind: if *additional_cost { cost_kind(game, *source).into() } else { String::new() },
                detail,
                extra_modes_cost,
                texts: modes
                    .iter()
                    .filter_map(|l| match l {
                        Label::Face(face) => Some(game.cards.get(*face).text.to_string()),
                        _ => None,
                    })
                    .collect(),
            })
        }
        RequestKind::ChooseSubtype { source, choices } => PromptPayload::ChooseModesPromptMessage(ChooseModesPrompt {
            player,
            source: source_handle(game, handles, *source),
            modes: choices.iter().map(|s| s.name().to_string()).collect(),
            eligible: (0..choices.len() as i32).collect(),
            count: 1,
            at_most: 1,
            heading: "Choose a creature type".into(),
            ..Default::default()
        }),
        RequestKind::ChooseColors { source, choices, count } => PromptPayload::ChooseColorsPromptMessage(ChooseColorsPrompt {
            player,
            source: source_handle(game, handles, *source),
            choices: colors_view(*choices),
            count: i32::from(*count),
            may_repeat: i32::from(*count) > choices.iter().count() as i32,
            verb: String::new(),
        }),
        RequestKind::ChooseName { source, looked_at, suggestion, kind } => {
            let mut names: Vec<String> = game
                .cards
                .cards
                .iter()
                // A split card has two names and a player names one of them (CR 709.4a): its halves are listed, not the whole.
                .filter(|c| c.name != "Intrinsic" && !c.name.is_empty() && c.split.is_none() && kind.allows(c.is(CardType::Land)))
                .map(|c| c.name.to_string())
                .collect();
            names.sort();
            names.dedup();
            let owner = looked_at.first().map_or(0, |c| seat(game.state.obj(*c).owner));
            PromptPayload::NameCardPromptMessage(NameCardPrompt {
                player,
                source: source_handle(game, handles, *source),
                owner,
                hand: handles_of(handles, looked_at),
                suggestion: suggestion.map_or_else(|| names.first().cloned().unwrap_or_default(), |c| game.cards.get(c).name.to_string()),
                names,
                kind: kind.word().to_string(),
            })
        }
        RequestKind::May { source, purpose: Purpose::Ask(question) } => PromptPayload::MayPromptMessage(MayPrompt {
            player,
            source: source_handle(game, handles, *source),
            verb: question.verb.into(),
            amount: i32::from(question.amount),
            question: question.ask.into(),
            detail: question.detail.into(),
            yes: question.yes.into(),
            no: question.no.into(),
            no_default: !question.yes_is_default,
        }),
        RequestKind::May { source, purpose: purpose @ (Purpose::RevealForMiracle(_) | Purpose::CastForMiracleCost(_)) } => {
            PromptPayload::MayPromptMessage(miracle_prompt(game, player, source_handle(game, handles, *source), *purpose))
        }
        RequestKind::May { source, purpose } => {
            let (verb, amount) = may_verb(*purpose);
            PromptPayload::MayPromptMessage(MayPrompt { player, source: source_handle(game, handles, *source), verb, amount, ..MayPrompt::default() })
        }
        RequestKind::ChoosePlayer { source, candidates, verb, amount } => PromptPayload::ChoosePlayerPromptMessage(ChoosePlayerPrompt {
            player,
            source: source_handle(game, handles, *source),
            candidates: candidates.iter().map(|&p| seat(p)).collect(),
            verb: (*verb).to_string(),
            amount: i32::from(*amount),
        }),
        RequestKind::ChooseDividedTargets { source, candidates, total, max_targets, optional } => {
            let source_handle = source_handle(game, handles, *source);
            PromptPayload::ChooseDividedTargetsPromptMessage(ChooseDividedTargetsPrompt {
                player,
                ability: source_handle,
                source: source_handle,
                text: String::new(),
                candidates: candidates.iter().filter_map(|t| target_message(handles, *t)).collect(),
                total: i32::from(*total),
                max_targets: i32::from(*max_targets),
                optional: *optional,
            })
        }
        RequestKind::DeclareAttackers { eligible, players, planeswalkers, battles, requirements } => {
            PromptPayload::DeclareAttackersPromptMessage(DeclareAttackersPrompt {
                player,
                eligible: handles_of(handles, eligible),
                attackable_players: players.iter().map(|&p| seat(p)).collect(),
                attackable_planeswalkers: handles_of(handles, planeswalkers),
                attackable_battles: handles_of(handles, battles),
                requirements: requirements
                    .iter()
                    .map(|&a| AttackRequirementMessage { attacker: handles.handle(a), players: Vec::new() })
                    .collect(),
                forbidden: Vec::new(),
            })
        }
        RequestKind::AssignCombatDamage { divisions } => PromptPayload::AssignCombatDamagePromptMessage(AssignCombatDamagePrompt {
            player,
            divisions: divisions
                .iter()
                .map(|d| DamageDivisionMessage {
                    source: handles.handle(d.source),
                    damage: i32::from(d.damage),
                    creatures: handles_of(handles, &d.creatures),
                    lethal: d.lethal.iter().map(|&l| i32::from(l)).collect(),
                    may_assign_to_defender: d.may_assign_to_defender,
                    defender: d.defender.player().map_or(0, seat),
                    suggested: d.suggested.iter().map(|&l| i32::from(l)).collect(),
                    suggested_to_defender: i32::from(d.suggested_to_defender),
                    defender_permanent: d.defender.object().map_or(ViewHandle::NONE, |id| handles.handle(id)),
                })
                .collect(),
        }),
        RequestKind::DeclareBlockers { eligible, attackers, allowed } => {
            PromptPayload::DeclareBlockersPromptMessage(DeclareBlockersPrompt {
                player,
                eligible: handles_of(handles, eligible),
                attackers: handles_of(handles, attackers),
                allowed: allowed
                    .iter()
                    .map(|&(blocker, attacker)| BlockMessage { blocker: handles.handle(blocker), attacker: handles.handle(attacker) })
                    .collect(),
                minimum_blockers: attackers.iter().map(|&a| i32::from(game.chars(a).minimum_blockers())).collect(),
            })
        }
        RequestKind::DiscardToHandSize { count, hand } => PromptPayload::DiscardPromptMessage(DiscardPrompt {
            player,
            count: i32::from(*count),
            maximum_hand_size: 7,
            hand: handles_of(handles, hand),
        }),
        RequestKind::Mulligan { mulligans } => PromptPayload::MulliganPromptMessage(MulliganPrompt {
            player,
            mulligans: i32::from(*mulligans),
            keeps: 7 - i32::from(*mulligans),
        }),
        RequestKind::ChooseStartingPlayer { candidates } => {
            PromptPayload::ChooseStartingPlayerPromptMessage(ChooseStartingPlayerPrompt {
                player,
                candidates: candidates.iter().map(|&p| seat(p)).collect(),
            })
        }
        RequestKind::OrderTriggers { sources, abilities } => PromptPayload::OrderTriggersPromptMessage(OrderTriggersPrompt {
            player,
            sources: sources.iter().map(|&s| source_handle(game, handles, s)).collect(),
            texts: abilities.iter().map(|&a| game.cards.ability(a).text().to_string()).collect(),
        }),
        RequestKind::DistributeCards { source, cards, destinations, may_shuffle } => {
            PromptPayload::DistributeCardsPromptMessage(DistributeCardsPrompt {
                player,
                source: source_handle(game, handles, *source),
                cards: handles_of(handles, cards),
                destinations: destinations
                    .iter()
                    .map(|d| CardDestinationMessage {
                        kind: match d.kind {
                            DestinationKind::LibraryTop => CardDestinationKindView::LibraryTop,
                            DestinationKind::LibraryBottom => CardDestinationKindView::LibraryBottom,
                            DestinationKind::Graveyard => CardDestinationKindView::Graveyard,
                            DestinationKind::Hand => CardDestinationKindView::Hand,
                            DestinationKind::Exile => CardDestinationKindView::Exile,
                            DestinationKind::BattlefieldFaceDown => CardDestinationKindView::BattlefieldFaceDown,
                        },
                        owner: player,
                        min: i32::from(d.min),
                        max: if d.max == u8::MAX { i32::MAX } else { i32::from(d.max) },
                        eligible: handles_of(handles, &d.eligible),
                        ordered: matches!(d.kind, DestinationKind::LibraryTop | DestinationKind::LibraryBottom),
                        randomized: false,
                        groups: Vec::new(),
                        group_names: Vec::new(),
                    })
                    .collect(),
                may_shuffle: *may_shuffle,
            })
        }
        RequestKind::SearchLibrary { source, library, eligible, count, optional, destination } => {
            let _ = source;
            // Whose library it is: the cards in it are all its owner's.
            let owner = library.first().map_or(request.player, |c| game.state.obj(*c).owner);
            PromptPayload::SearchLibraryPromptMessage(SearchLibraryPrompt {
                player,
                count: i32::from(*count),
                optional: *optional,
                library: handles_of(handles, library),
                eligible: handles_of(handles, eligible),
                destination: match destination {
                    SearchDestination::Hand => SearchDestinationView::Hand,
                    SearchDestination::TopOfLibrary => SearchDestinationView::TopOfLibrary,
                    SearchDestination::Graveyard => SearchDestinationView::Graveyard,
                    SearchDestination::Battlefield => SearchDestinationView::Battlefield,
                    SearchDestination::Exile => SearchDestinationView::Exile,
                    SearchDestination::ExileFaceDown => SearchDestinationView::ExileFaceDown,
                    SearchDestination::Cast => SearchDestinationView::Cast,
                    SearchDestination::Found => SearchDestinationView::Found,
                    SearchDestination::BattlefieldTapped => SearchDestinationView::BattlefieldTapped,
                    SearchDestination::CastFree => SearchDestinationView::CastFree,
                },
                owner: seat(owner),
            })
        }
        RequestKind::ChooseReplacement { subject, sources, hints } => {
            PromptPayload::ChooseReplacementPromptMessage(ChooseReplacementPrompt {
                player,
                subject: subject.map_or(ViewHandle::NONE, |s| source_handle(game, handles, s)),
                event: String::new(),
                sources: sources.iter().map(|&s| source_handle(game, handles, s)).collect(),
                hints: hints.iter().map(|h| (*h).to_string()).collect(),
            })
        }
    };
    Ok((payload, request.may_cancel))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A game dealt these two decks, top of the library first, and the objects of the first seat's opening hand.
    fn game_with(first: &[&str]) -> (Game, Vec<ObjectId>) {
        let deck = |top: &[&str]| top.iter().map(|c| c.to_string()).chain(std::iter::repeat_n("Forest".to_string(), 20)).collect::<Vec<_>>();
        let game = deal(&[deck(first), deck(&[])], 1, false, None).expect("a game");
        let seat = game.state.seats().next().expect("a seat");
        let hand = game.state.zone(Zone::hand(seat)).to_vec();
        (game, hand)
    }

    /// A choice is worded as its card words it, not as the cards that happen to be eligible: a hand with only creatures left is
    /// still a choice of "a nonland card" when that is what the card says, and a card that says nothing says "card".
    #[test]
    fn a_choice_says_the_kind_its_card_gave_and_infers_none() {
        let (game, hand) = game_with(&["Llanowar Elves", "Ornithopter", "Forest"]);
        let creatures: Vec<ObjectId> = hand.iter().copied().filter(|&c| game.chars(c).is(CardType::Creature)).collect();
        assert_eq!(creatures.len(), 2, "the Elves and the Ornithopter");
        let seat = game.state.seats().next().expect("a seat");
        let kind_of = |kind: &'static str| {
            let request = Request {
                id: 1,
                player: seat,
                kind: RequestKind::ChooseCards {
                    source: hand[0],
                    purpose: Purpose::Exile,
                    cards: hand.clone(),
                    eligible: creatures.clone(),
                    count: 1,
                    up_to: false,
                    kind,
                },
                may_cancel: false,
            };
            match prompt(&game, &request, &mut HandleSpace::new()).expect("a prompt").0 {
                PromptPayload::ChooseCardsPromptMessage(p) => p.kind,
                other => panic!("{other:?}"),
            }
        };
        assert_eq!(kind_of("nonland"), "nonland");
        assert_eq!(kind_of("artifact or creature"), "artifact or creature");
        // Only creatures were eligible, and nothing was said: nothing is claimed.
        assert_eq!(kind_of(""), "");
    }

    #[test]
    fn casts_during_resolution_are_asked_in_the_words_of_what_they_cost() {
        assert_eq!(may_verb(Purpose::CastFree), ("CastDuringResolution".to_string(), 1));
        assert_eq!(may_verb(Purpose::Cast), ("CastDuringResolution".to_string(), 0));
        assert_eq!(may_verb(Purpose::CastCopyFree).0, "CastCopyWithoutPaying");
        assert_eq!(may_verb(Purpose::CastForMadness).0, "CastForMadnessCost");
        assert_eq!(may_verb(Purpose::CastTransformedFree).0, "CastTransformed");
        assert_eq!(may_verb(Purpose::CastOrBottom).0, "CastWithoutPaying");
        assert_eq!(may_verb(Purpose::CastFoundOrHand).0, "CastFoundOrPutIntoHand");
    }

    #[test]
    fn the_faces_of_a_card_are_named_with_their_costs_and_say_so() {
        let (game, _) = game_with(&["Bonecrusher Giant"]);
        let giant = card("Bonecrusher Giant").expect("the card");
        assert_eq!(face_text(&game, giant), "Bonecrusher Giant {2}{R}");

        let (detail, costs) = modes_detail(&game, &[Label::Face(giant), Label::Face(giant)], true);
        assert!(detail.contains("Only the face you choose"), "{detail}");
        assert!(!costs);
    }

    /// Miracle's questions name the cost (CR 702.94a), and a choice between miracle abilities names each one's cost and
    /// whose it is: the card's own, or the permanent's that gave it (CR 607.5), whose cost is not on the card's face. A
    /// Room's granted cost is read off each door (CR 709.3a).
    #[test]
    fn miracle_questions_name_the_cost_and_who_gave_it() {
        let (game, _) = game_with(&[]);
        let fanatic = card("Metamorphosis Fanatic").expect("the card");
        let hop = card("Hop to It").expect("the card");
        let room = card("Unholy Annex // Ritual Chamber").expect("the card");
        let lorehold = card("Lorehold, the Historian").expect("the card");
        let aminatou = card("Aminatou, Veil Piercer").expect("the card");
        let own = MiracleOf { card: fanatic, granter: fanatic, index: MiracleOf::OWN };
        let given = MiracleOf { card: hop, granter: lorehold, index: 0 };

        let reveal = miracle_prompt(&game, 1, ViewHandle(7), Purpose::RevealForMiracle(Some(own)));
        assert_eq!((reveal.verb.as_str(), reveal.question.as_str()), ("RevealForMiracle", "Reveal {source} for its miracle {1}{B}?"));
        assert!(reveal.no_default, "revealing is never the default");
        let reveal = miracle_prompt(&game, 1, ViewHandle(7), Purpose::RevealForMiracle(Some(given)));
        assert!(reveal.detail.starts_with("Lorehold, the Historian gives it miracle {2}"), "{}", reveal.detail);
        let several = miracle_prompt(&game, 1, ViewHandle(7), Purpose::RevealForMiracle(None));
        assert_eq!(several.question, "Reveal {source} for one of its miracle abilities?");

        assert_eq!(miracle_label(&game, own), "Miracle {1}{B}, its own");
        assert_eq!(miracle_label(&game, given), "Miracle {2}, from Lorehold, the Historian");
        assert_eq!(miracle_label(&game, MiracleOf { card: hop, granter: aminatou, index: 0 }), "Miracle {W}, from Aminatou, Veil Piercer");
        assert_eq!(
            miracle_label(&game, MiracleOf { card: room, granter: aminatou, index: 0 }),
            "Miracle Unholy Annex {B} or Ritual Chamber {B}{B}, from Aminatou, Veil Piercer"
        );
        let meathook = card("The Meathook Massacre").expect("the card");
        assert_eq!(miracle_label(&game, MiracleOf { card: meathook, granter: aminatou, index: 0 }), "Miracle {X}{B}{B} (X less 4), from Aminatou, Veil Piercer");
        assert_eq!(cost_words(None), "(unpayable: it has no mana cost)");

        let cast = miracle_prompt(&game, 1, ViewHandle(7), Purpose::CastForMiracleCost(given));
        assert_eq!((cast.verb.as_str(), cast.question.as_str()), ("CastForMiracleCost", "Cast {source} for its miracle {2}?"));
        assert!(cast.detail.starts_with("You pay its miracle {2} instead of its mana cost"), "{}", cast.detail);
        assert!(!cast.no_default);
    }
}
