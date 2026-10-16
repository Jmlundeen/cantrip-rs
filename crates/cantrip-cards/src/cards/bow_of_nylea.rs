use cantrip_api::*;
use cantrip_mechanics::library::ask_library_order;
use cantrip_mechanics::{damage_target, effects, targets};
use cantrip_oracle as oracle;

/// "Attacking creatures you control have deathtouch" (CR 506.4, 613.1f): re-asked on every pass, so a creature put onto
/// the battlefield attacking has it and one removed from combat, or combat ending, loses it.
fn attacking_creature_you_control(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let state = view.state();
    so_far.is(CardType::Creature)
        && state.obj(affected).controller == state.obj(source).controller
        && state.attackers.iter().any(|a| a.id == affected)
}

fn deathtouch(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Deathtouch);
}

/// "Target creature with flying" (CR 115.1).
fn creature_with_flying(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.chars(id).has(Keyword::Flying))
}

/// One of "up to four target cards from your graveyard" (CR 115.1): a mode's target slots are one target each, so the
/// four are four optional slots, and a card an earlier one chose is not a target again (CR 115.3).
fn card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[]) && !q.chosen_targets().contains(&t)
}

const UP_TO_ONE_CARD: TargetSpec = TargetSpec { min: 0, max: 1, filter: card_in_your_graveyard, label: "" };

/// Put a +1/+1 counter on target creature.
fn counter_on_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// Bow of Nylea deals 2 damage to target creature with flying.
fn two_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

/// Put up to four target cards from your graveyard on the bottom of your library in any order (CR 401.4, 608.2b): the
/// targets still legal are ordered by the controller (asked only when there is more than one), moved together and
/// placed in that order.
fn bottom_in_any_order(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let cards = targets::legal_objects(g, item);
    if cards.is_empty() {
        return Ok(());
    }
    let order = ask_library_order(g, you, item.object, &cards, DestinationKind::LibraryBottom)?;
    g.put_into_library(&order, LibraryPlace::Bottom);
    Ok(())
}

static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef { text: "Put a +1/+1 counter on target creature.", targets: &[targets::CREATURE], effect: counter_on_creature, cost: ManaCost::ZERO },
        ModeDef {
            text: "Bow of Nylea deals 2 damage to target creature with flying.",
            targets: &[TargetSpec { min: 1, max: 1, filter: creature_with_flying, label: "" }],
            effect: two_damage,
            cost: ManaCost::ZERO,
        },
        ModeDef { text: "You gain 3 life.", targets: &[], effect: effects::gain_life::<3>, cost: ManaCost::ZERO },
        ModeDef {
            text: "Put up to four target cards from your graveyard on the bottom of your library in any order.",
            targets: &[UP_TO_ONE_CARD, UP_TO_ONE_CARD, UP_TO_ONE_CARD, UP_TO_ONE_CARD],
            effect: bottom_in_any_order,
            cost: ManaCost::ZERO,
        },
    ],
    min: 1,
    max: 1,
    escalate: None,
    repeat: false, allowed: None,
};

pub static CARD: CardDef = CardDef {
    abilities: &[modal_ability(cost("{1}{G}, {T}"), &MODAL, "{1}{G}, {T}: Choose one \u{2014} put a +1/+1 counter on target creature; Bow of Nylea deals 2 damage to target creature with flying; you gain 3 life; or put up to four target cards from your graveyard on the bottom of your library in any order.")],
    statics: &[StaticDef {
        text: "Attacking creatures you control have deathtouch.",
        layer: Layer::Ability,
        affects: attacking_creature_you_control,
        apply: deathtouch,
        ..StaticDef::BLANK
    }],
    ..oracle::BOW_OF_NYLEA
};
