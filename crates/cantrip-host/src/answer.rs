//! The wire's answers as the engine's, resolving a seat's handles back to objects.
//!
//! | wire answer                                   | native `Answer`                                              |
//! |-----------------------------------------------|--------------------------------------------------------------|
//! | `PassAnswer`                                  | `Pass`                                                       |
//! | `CancelAnnouncementAnswer`                    | `Cancel`                                                     |
//! | `CardsChosenAnswer`                           | `Cards`                                                      |
//! | `TakeActionAnswer`                            | `Act` (a mana ability's `Sacrifice` becomes its target)      |
//! | `AttackersDeclaredAnswer`                     | `Attack` (a player or a battle)                              |
//! | `BlockersDeclaredAnswer`                      | `Block`                                                      |
//! | `CombatDamageAssignedAnswer`                  | `Shares`                                                     |
//! | `CardsDistributedAnswer`                      | `Placements`                                                 |
//! | `TargetsChosenAnswer`                         | `Targets`                                                    |
//! | `MayChosenAnswer`                             | `Yes` / `No` (yes to a mulligan prompt takes the mulligan)   |
//! | `ModesChosenAnswer`                           | `Modes`, or `Subtype` to a creature-type question            |
//! | `ReplacementChosenAnswer`                     | `Index`                                                      |
//! | `TriggersOrderedAnswer`                       | `Order`                                                      |
//! | `CardNamedAnswer`                             | `Name`                                                       |
//! | `PlayerChosenAnswer`                          | `Player`                                                     |
//! | `ColorsChosenAnswer`                          | `Colors`                                                     |
//! | `TargetsDividedAnswer`                        | `Divided`                                                    |

use cantrip_engine::*;

use crate::handles::HandleSpace;
use crate::protocol::*;
use crate::view::*;

/// Why an answer could not even be put to the engine.
pub enum Untranslatable {
    UnknownHandle(String),
    Illegal(String),
}

fn object(handles: &HandleSpace, game: &Game, h: ViewHandle) -> Result<ObjectId, Untranslatable> {
    match handles.object(h) {
        Some(id) if !is_ability_key(id) && id.index() < game.state.objects.len() => Ok(id),
        _ => Err(Untranslatable::UnknownHandle(format!("handle {} names nothing this seat can see", h.0))),
    }
}

fn target(handles: &HandleSpace, game: &Game, t: &TargetMessage) -> Result<Target, Untranslatable> {
    if t.is_player() {
        return player_at(t.player).map(Target::Player).ok_or_else(|| Untranslatable::Illegal("no such seat".into()));
    }
    match handles.object(t.object) {
        Some(id) if is_ability_key(id) => Ok(Target::Ability(id.0 & 0x7fff_ffff)),
        Some(id) if id.index() < game.state.objects.len() => Ok(Target::Object(id)),
        _ => Err(Untranslatable::UnknownHandle(format!("handle {} names nothing this seat can see", t.object.0))),
    }
}

fn cards(handles: &HandleSpace, game: &Game, list: &[ViewHandle]) -> Result<Vec<ObjectId>, Untranslatable> {
    list.iter().map(|&h| object(handles, game, h)).collect()
}

fn kind_of(kind: ActionKindView) -> Result<ActionKind, Untranslatable> {
    Ok(match kind {
        ActionKindView::PlayLand => ActionKind::PlayLand,
        ActionKindView::CastSpell => ActionKind::CastSpell,
        ActionKindView::ActivateAbility => ActionKind::ActivateAbility,
        ActionKindView::CancelCasting => ActionKind::CancelCasting,
        ActionKindView::PayCost => ActionKind::PayCost,
        ActionKindView::DeclineCost => ActionKind::DeclineCost,
        ActionKindView::AssignMana => ActionKind::AssignMana,
        ActionKindView::ConfirmCasting => ActionKind::ConfirmCasting,
        ActionKindView::IncreaseX => ActionKind::IncreaseX,
        ActionKindView::DecreaseX => ActionKind::DecreaseX,
        ActionKindView::TapToPay => ActionKind::TapToPay,
        ActionKindView::ExileToPay => ActionKind::ExileToPay,
        // Turning a face-down permanent face up is its one ability (CR 708.2a), offered as an activation.
        ActionKindView::TurnFaceUp => ActionKind::ActivateAbility,
        // Unlocking a Room's door is the intrinsic ability it was offered as (CR 709.5e).
        ActionKindView::UnlockDoor => ActionKind::ActivateAbility,
        other => return Err(Untranslatable::Illegal(format!("{other:?} is not something this engine offers"))),
    })
}

pub fn translate(
    payload: &AnswerPayload,
    request: &Request,
    handles: &HandleSpace,
    game: &Game,
) -> Result<Answer, Untranslatable> {
    Ok(match payload {
        AnswerPayload::PassAnswer(_) => Answer::Pass,
        AnswerPayload::CancelAnnouncementAnswer(_) => Answer::Cancel,
        AnswerPayload::CardsChosenAnswer(a) => Answer::Cards(cards(handles, game, &a.cards)?),
        AnswerPayload::TakeActionAnswer(a) => {
            let kind = kind_of(a.kind)?;
            let mut action = Action {
                kind,
                source: if a.source.is_none() { None } else { Some(object(handles, game, a.source)?) },
                index: a.ability_index.clamp(0, 255) as u8,
                targets: a.targets.iter().map(|t| target(handles, game, t)).collect::<Result<_, _>>()?,
                color: a.color.map(color_of),
                mana: a.mana.map(mana_of),
                restriction: restriction_of(a.filter, a.effect),
            };
            if let Some(s) = a.sacrifice {
                action.targets.push(Target::Object(object(handles, game, s)?));
            }
            // A handle names an object and every earlier one it was (`HandleSpace::rebind`): the source is the one the
            // offered option names - Channel's mana ability, offered from the card that granted it, which has moved since.
            if let Some(source) = action.source
                && let RequestKind::PayCosts { options, .. } | RequestKind::Priority { options } = &request.kind
                && !options.iter().any(|o| o.kind == kind && o.index == action.index && o.source == Some(source))
                && let Some(o) = options
                    .iter()
                    .find(|o| o.kind == kind && o.index == action.index && o.source.is_some_and(|s| handles.existing(s) == a.source))
            {
                action.source = o.source;
            }
            // A cancel or a decline names no source, whatever the option it matches carries.
            if matches!(kind, ActionKind::CancelCasting | ActionKind::DeclineCost | ActionKind::PayCost | ActionKind::ConfirmCasting)
                && let RequestKind::PayCosts { options, .. } | RequestKind::UnlessPays { options, .. } | RequestKind::Priority { options } = &request.kind
                && let Some(o) = options.iter().find(|o| o.kind == kind)
            {
                action.source = o.source;
                action.index = o.index;
            }
            Answer::Act(action)
        }
        AnswerPayload::AttackersDeclaredAnswer(a) => {
            let mut out = Vec::new();
            for attack in &a.attacks {
                let at = if attack.target_permanent.is_none() {
                    Target::Player(player_at(attack.target_player).ok_or_else(|| Untranslatable::Illegal("no such seat".into()))?)
                } else {
                    Target::Object(object(handles, game, attack.target_permanent)?)
                };
                out.push((object(handles, game, attack.attacker)?, at));
            }
            Answer::Attack(out)
        }
        AnswerPayload::BlockersDeclaredAnswer(a) => Answer::Block(
            a.blocks
                .iter()
                .map(|b| Ok((object(handles, game, b.blocker)?, object(handles, game, b.attacker)?)))
                .collect::<Result<_, Untranslatable>>()?,
        ),
        AnswerPayload::CombatDamageAssignedAnswer(a) => Answer::Shares(
            a.shares
                .iter()
                .map(|s| {
                    Ok((
                        object(handles, game, s.source)?,
                        if s.creature.is_none() { None } else { Some(object(handles, game, s.creature)?) },
                        s.amount.clamp(0, i32::from(u16::MAX)) as u16,
                    ))
                })
                .collect::<Result<_, Untranslatable>>()?,
        ),
        AnswerPayload::CardsDistributedAnswer(a) => Answer::Placements {
            placements: a.placements.iter().map(|p| cards(handles, game, p)).collect::<Result<_, _>>()?,
            shuffle: a.shuffle,
        },
        AnswerPayload::TargetsChosenAnswer(a) => {
            Answer::Targets(a.targets.iter().map(|t| target(handles, game, t)).collect::<Result<_, _>>()?)
        }
        AnswerPayload::MayChosenAnswer(a) => {
            if a.accept {
                Answer::Yes
            } else {
                Answer::No
            }
        }
        AnswerPayload::ModesChosenAnswer(a) => {
            if let RequestKind::ChooseSubtype { choices, .. } = &request.kind {
                let i = a.modes.first().copied().unwrap_or(-1);
                let subtype = usize::try_from(i).ok().and_then(|i| choices.get(i)).copied();
                Answer::Subtype(subtype.ok_or_else(|| Untranslatable::Illegal("not one of the types offered".into()))?)
            } else {
                Answer::Modes(a.modes.iter().map(|&m| m.clamp(0, 255) as u8).collect())
            }
        }
        AnswerPayload::ReplacementChosenAnswer(a) => Answer::Index(a.index.clamp(0, 255) as u8),
        AnswerPayload::TriggersOrderedAnswer(a) => Answer::Order(a.order.iter().map(|&m| m.clamp(0, 255) as u8).collect()),
        AnswerPayload::CardNamedAnswer(a) => {
            Answer::Name(Some(card(&a.name).ok_or_else(|| Untranslatable::Illegal(format!("no card is named \"{}\"", a.name)))?))
        }
        AnswerPayload::PlayerChosenAnswer(a) => {
            Answer::Player(player_at(a.player).ok_or_else(|| Untranslatable::Illegal("no such seat".into()))?)
        }
        AnswerPayload::ColorsChosenAnswer(a) => Answer::Colors(a.colors.iter().map(|&c| color_of(c)).collect()),
        AnswerPayload::TargetsDividedAnswer(a) => Answer::Divided(
            a.shares
                .iter()
                .map(|s| Ok((target(handles, game, &s.target)?, s.amount.clamp(0, 255) as u16)))
                .collect::<Result<_, Untranslatable>>()?,
        ),
    })
}

fn wire_target(handles: &mut HandleSpace, t: Target) -> TargetMessage {
    match t {
        Target::Player(p) => TargetMessage::player(seat(p)),
        Target::Ability(serial) => TargetMessage::object(handles.handle(ability_key(serial))),
        Target::Object(id) => TargetMessage::object(handles.handle(id)),
        Target::None => TargetMessage::player(0),
    }
}

/// An engine answer as the wire's, in a seat's handles: the inverse of [`translate`], for saying what a scripted seat
/// is meant to answer (`Session::scenario`). Objects the seat has no handle for are given one.
pub fn payload_of(answer: &Answer, request: &Request, handles: &mut HandleSpace, game: &Game) -> AnswerPayload {
    match answer {
        Answer::Pass => AnswerPayload::PassAnswer(PassAnswer {}),
        Answer::Cancel => AnswerPayload::CancelAnnouncementAnswer(CancelAnnouncementAnswer {}),
        Answer::Cards(ids) => AnswerPayload::CardsChosenAnswer(CardsChosenAnswer { cards: ids.iter().map(|&id| handles.handle(id)).collect() }),
        Answer::Act(action) => {
            let source = action.source.map_or(ViewHandle::NONE, |s| handles.handle(s));
            let mut a = TakeActionAnswer::new(crate::prompt::action_kind_view(action.kind), source);
            a.ability_index = i32::from(action.index);
            a.color = action.color.map(color_view);
            a.mana = action.mana.map(mana_view);
            if let Some(restriction) = action.restriction {
                (a.filter, a.effect) = restriction_view(restriction);
            }
            // A mana ability's sacrifice goes to the wire as `Sacrifice`, not a target (`prompt::option_message`).
            let sacrifices = action.kind == ActionKind::ActivateAbility
                && action.source.is_some_and(|s| {
                    matches!(
                        game.chars(s).abilities().nth(action.index as usize).map(|a| game.cards.ability(a)),
                        Some(AbilityDef::Activated(d)) if d.is_mana_ability() && d.cost.permanents.is_some()
                    )
                });
            if sacrifices && let Some(Target::Object(id)) = action.targets.first() {
                a.sacrifice = Some(handles.handle(*id));
            } else {
                a.targets = action.targets.iter().map(|&t| wire_target(handles, t)).collect();
            }
            AnswerPayload::TakeActionAnswer(a)
        }
        Answer::Targets(ts) => AnswerPayload::TargetsChosenAnswer(TargetsChosenAnswer { targets: ts.iter().map(|&t| wire_target(handles, t)).collect() }),
        Answer::Modes(ms) => AnswerPayload::ModesChosenAnswer(ModesChosenAnswer { modes: ms.iter().map(|&m| i32::from(m)).collect() }),
        Answer::Subtype(st) => {
            let index = match &request.kind {
                RequestKind::ChooseSubtype { choices, .. } => choices.iter().position(|c| c == st).map_or(-1, |i| i as i32),
                _ => -1,
            };
            AnswerPayload::ModesChosenAnswer(ModesChosenAnswer { modes: vec![index] })
        }
        Answer::Colors(cs) => AnswerPayload::ColorsChosenAnswer(ColorsChosenAnswer { colors: cs.iter().map(|&c| color_view(c)).collect() }),
        Answer::Yes => AnswerPayload::MayChosenAnswer(MayChosenAnswer { accept: true }),
        Answer::No => AnswerPayload::MayChosenAnswer(MayChosenAnswer { accept: false }),
        Answer::Attack(attacks) => AnswerPayload::AttackersDeclaredAnswer(AttackersDeclaredAnswer {
            attacks: attacks
                .iter()
                .map(|&(attacker, at)| match at {
                    Target::Player(p) => AttackMessage { attacker: handles.handle(attacker), target_player: seat(p), target_permanent: ViewHandle::NONE },
                    other => {
                        let permanent = match other {
                            Target::Object(id) => handles.handle(id),
                            _ => ViewHandle::NONE,
                        };
                        AttackMessage { attacker: handles.handle(attacker), target_player: 0, target_permanent: permanent }
                    }
                })
                .collect(),
        }),
        Answer::Block(blocks) => AnswerPayload::BlockersDeclaredAnswer(BlockersDeclaredAnswer {
            blocks: blocks.iter().map(|&(b, a)| BlockMessage { blocker: handles.handle(b), attacker: handles.handle(a) }).collect(),
        }),
        Answer::Player(p) => AnswerPayload::PlayerChosenAnswer(PlayerChosenAnswer { player: seat(*p) }),
        Answer::Name(id) => AnswerPayload::CardNamedAnswer(CardNamedAnswer { name: id.map_or(String::new(), |c| game.cards.get(c).name.to_string()) }),
        Answer::Order(order) => AnswerPayload::TriggersOrderedAnswer(TriggersOrderedAnswer { order: order.iter().map(|&o| i32::from(o)).collect() }),
        Answer::Index(i) => AnswerPayload::ReplacementChosenAnswer(ReplacementChosenAnswer { index: i32::from(*i) }),
        Answer::Shares(shares) => AnswerPayload::CombatDamageAssignedAnswer(CombatDamageAssignedAnswer {
            shares: shares
                .iter()
                .map(|&(source, creature, amount)| DamageShareMessage {
                    source: handles.handle(source),
                    creature: creature.map_or(ViewHandle::NONE, |c| handles.handle(c)),
                    amount: i32::from(amount),
                })
                .collect(),
        }),
        Answer::Placements { placements, shuffle } => AnswerPayload::CardsDistributedAnswer(CardsDistributedAnswer {
            placements: placements.iter().map(|p| p.iter().map(|&id| handles.handle(id)).collect()).collect(),
            shuffle: *shuffle,
        }),
        Answer::Divided(shares) => AnswerPayload::TargetsDividedAnswer(TargetsDividedAnswer {
            shares: shares.iter().map(|&(t, amount)| TargetShareMessage { target: wire_target(handles, t), amount: i32::from(amount) }).collect(),
        }),
    }
}
