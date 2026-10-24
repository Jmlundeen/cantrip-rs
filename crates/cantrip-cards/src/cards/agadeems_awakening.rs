use cantrip_api::*;
use cantrip_mechanics::{mana, select, targets};
use cantrip_oracle as oracle;

/// "Target creature card ... from your graveyard ... mana value X or less" (CR 115.1, 202.3): a quality of each target, reading
/// the X announced (none while it is only asked whether some X could have a target: any), as it is judged again on resolution.
/// A card in a graveyard with {X} in its cost has X of 0 (CR 202.3e), which `Query::mana_value` reads.
fn creature_card_in_your_graveyard_with_mana_value_x_or_less(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Creature])
        && t.object().is_some_and(|id| q.announced_x().is_none_or(|x| q.mana_value(id) <= x))
}

/// "Return from your graveyard to the battlefield any number of target creature cards that each have a different mana value X
/// or less." The targets still legal as it resolves (CR 608.2b) enter together under your control (CR 110.2a).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let cards: Vec<(ObjectId, PlayerId)> = targets::legal_objects(g, item).into_iter().map(|c| (c, you)).collect();
    g.put_all_onto_battlefield(&cards)?;
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::B), "{T}: Add {B}.")],
    enters: Some(mana::enters_tapped_unless_pay_three_life),
    ..oracle::AGADEEM_THE_UNDERCRYPT
};

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[TargetSpec { min: 0, max: TargetSpec::ANY_NUMBER, ..target(creature_card_in_your_graveyard_with_mana_value_x_or_less) }
            .together(select::DIFFERENT_MANA_VALUES)
            .labelled("Creature cards from your graveyard to return to the battlefield")],
        effect,
    ),
    back: Some(&BACK),
    ..oracle::AGADEEMS_AWAKENING
};
