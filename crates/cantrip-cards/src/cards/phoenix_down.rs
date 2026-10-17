use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature card with mana value 4 or less from your graveyard" (CR 115.1, 202.3).
fn creature_card_with_mana_value_4_or_less_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard_with_mana_value_at_most(q, asker, t, &[CardType::Creature], 4)
}

/// "Return target creature card with mana value 4 or less from your graveyard to the battlefield tapped." A target no longer
/// legal is not returned (CR 608.2b); the ability is not countered as long as one of its targets is legal, and this one has
/// only that.
fn return_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_onto_battlefield(card, item.controller, true)?;
    Ok(())
}

/// "Target Skeleton, Spirit, or Zombie" (CR 115.1, 205.3): a permanent with one of those creature types, as this artifact's
/// text names them (CR 612.2).
fn skeleton_spirit_or_zombie(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let words = q.words(asker.source);
    let types = [words.subtype(Subtype::Skeleton), words.subtype(Subtype::Spirit), words.subtype(Subtype::Zombie)];
    matches!(t, Target::Object(id) if targets::permanent(q, id) && types.iter().any(|&s| q.chars(id).has_subtype(s)))
}

/// "Exile target Skeleton, Spirit, or Zombie."
fn exile_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    g.exile(permanent);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(return_creature, "Return target creature card with mana value 4 or less from your graveyard to the battlefield tapped.")
        .targets(&[target(creature_card_with_mana_value_4_or_less_in_your_graveyard)]),
    mode(exile_target, "Exile target Skeleton, Spirit, or Zombie.").targets(&[target(skeleton_spirit_or_zombie)]),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_ability(
        cost("{1}{W}, {T}").and(EXILE_THIS),
        &MODAL,
        "{1}{W}, {T}, Exile this artifact: Choose one \u{2014} return target creature card with mana value 4 or less from your graveyard to the battlefield tapped; or exile target Skeleton, Spirit, or Zombie.",
    )],
    ..oracle::PHOENIX_DOWN
};
