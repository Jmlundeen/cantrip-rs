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
//! | `TapForPower`          | `ChooseCardsPromptMessage` (purpose `TapToPay`, `Powers`, `MinimumPower`)          |
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

fn action_kind_view(kind: ActionKind) -> ActionKindView {
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
    m.targets_required = i32::from(o.targets_required);
    m.targets_optional = o.targets_optional;
    m.color_choices = colors_view(o.color_choices);
    m.description = describe(game.cards, &game.state, o).to_string();
    if let Some(mana) = o.mana {
        m.produces = vec![mana_view(mana)];
    }
    match o.restriction {
        Some(ManaRestriction::NotNonartifactSpells) => m.filter = ManaSpendFilterView::NotNonartifactSpells,
        Some(ManaRestriction::ArtifactSpells) => m.filter = ManaSpendFilterView::ArtifactSpells,
        _ => {}
    }
    let candidates: Vec<TargetMessage> = o.target_candidates.iter().filter_map(|t| target_message(handles, *t)).collect();

    let Some(src) = o.source else {
        m.target_candidates = candidates;
        return m;
    };
    let def = game.cards.get(game.state.obj(src).card);
    match o.kind {
        ActionKind::CastSpell => {
            if o.index == WAY_ADVENTURE {
                m.spell_name = def.adventure.map_or(String::new(), |a| a.name.to_string());
            }
            let cost = if o.index == 0 || o.index == WAY_ADVENTURE {
                def.cost
            } else {
                def.ways.get(o.index as usize - 1).map_or(def.cost, |w| w.cost.mana)
            };
            m.has_x = cost.x > 0;
            m.target_candidates = candidates;
        }
        ActionKind::ActivateAbility => {
            let ability = game.chars(src).abilities().nth(o.index as usize).map(|a| game.cards.ability(a));
            if let Some(AbilityDef::Activated(d)) = ability {
                m.life_cost = i32::from(d.cost.life);
                if d.mana.is_some() {
                    for mana in Mana::ALL {
                        for _ in 0..d.adds.get(mana) {
                            m.produces.push(mana_view(mana));
                        }
                    }
                    if !o.color_choices.is_empty() {
                        m.produces_of_any_one_color = i32::from(d.adds_any.max(1));
                    }
                    m.cost_text = cost_text_of(&d.cost, def.name);
                    if d.cost.tap {
                        m.tap = Some(source);
                    }
                    // A mana ability's sacrifice comes with the activation: the engine takes it as a target.
                    if d.cost.permanents.is_some() && o.targets_required > 0 {
                        m.sacrifice_candidates = candidates.iter().map(|t| t.object).collect();
                        m.targets_required = 0;
                        return m;
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
        Purpose::TapToPay => V::TapToPay,
        Purpose::OrderSimultaneous => V::OrderSimultaneous,
        Purpose::Destroy => V::Destroy,
        Purpose::Vote => V::Vote,
        Purpose::PutIntoGraveyard => V::PutIntoGraveyard,
        Purpose::Play => V::Play,
        Purpose::Splice => V::Splice,
        Purpose::PutOnBottom => V::PutOnBottom,
        Purpose::Attach => V::Attach,
        Purpose::SeparatePiles
        | Purpose::ChoosePile
        | Purpose::CopyAsEnters
        | Purpose::CopyWhich
        | Purpose::Targets
        | Purpose::Color
        | Purpose::Optional
        | Purpose::RevealForMiracle
        | Purpose::CastForMiracleCost
        | Purpose::PromiseGift
        | Purpose::AlternativeCost => V::Keep,
    }
}

fn may_verb(purpose: Purpose) -> String {
    match purpose {
        Purpose::Cast => "CastDuringResolution".into(),
        Purpose::CastForMiracleCost => "CastForMiracleCost".into(),
        Purpose::PromiseGift => "PromiseGift".into(),
        Purpose::CopyAsEnters => "CopyAsEnters".into(),
        other => format!("{other:?}"),
    }
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
        RequestKind::ChooseCards { source, purpose, cards, eligible, count, up_to } => {
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
                ..Default::default()
            })
        }
        RequestKind::TapForPower { source, purpose, cards, powers, minimum } => {
            let _ = purpose;
            PromptPayload::ChooseCardsPromptMessage(ChooseCardsPrompt {
                player,
                owner: player,
                source: source_handle(game, handles, *source),
                purpose: CardChoicePurposeView::TapToPay,
                cards: handles_of(handles, cards),
                eligible: handles_of(handles, cards),
                count: cards.len() as i32,
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
        RequestKind::ChooseTarget { source, purpose, candidates, optional, count, any_number } => {
            let source_handle = source_handle(game, handles, *source);
            PromptPayload::ChooseTargetPromptMessage(ChooseTargetPrompt {
                player,
                ability: source_handle,
                source: source_handle,
                text: String::new(),
                candidates: candidates.iter().filter_map(|t| target_message(handles, *t)).collect(),
                optional: *optional,
                count: i32::from(*count),
                targeted: *purpose == Purpose::Targets,
                minimum: if *optional { 0 } else { i32::from(*count) },
                any_number: *any_number,
            })
        }
        RequestKind::ChooseModes { source, modes, eligible, count, at_most, affordable, additional_cost, repeat } => {
            PromptPayload::ChooseModesPromptMessage(ChooseModesPrompt {
                player,
                source: source_handle(game, handles, *source),
                modes: modes.iter().map(|l| label_text(l).to_string()).collect(),
                eligible: eligible.iter().map(|&m| i32::from(m)).collect(),
                count: i32::from(*count),
                may_repeat: *repeat,
                additional_cost: *additional_cost,
                heading: String::new(),
                at_most: i32::from(*at_most),
                affordable: affordable.iter().map(|&m| i32::from(m)).collect(),
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
        RequestKind::ChooseName { source, looked_at, suggestion, nonland_only } => {
            let mut names: Vec<String> = game
                .cards
                .cards
                .iter()
                .filter(|c| c.name != "Intrinsic" && !c.name.is_empty() && !(*nonland_only && c.is(CardType::Land)))
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
            })
        }
        RequestKind::May { source, purpose } => PromptPayload::MayPromptMessage(MayPrompt {
            player,
            source: source_handle(game, handles, *source),
            verb: may_verb(*purpose),
            amount: 0,
        }),
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
        RequestKind::SearchLibrary { source, library, eligible, count, optional } => {
            let _ = source;
            PromptPayload::SearchLibraryPromptMessage(SearchLibraryPrompt {
                player,
                count: i32::from(*count),
                optional: *optional,
                library: handles_of(handles, library),
                eligible: handles_of(handles, eligible),
                destination: SearchDestinationView::Hand,
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
