use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "{X}: Lazav becomes a copy of target creature card in your graveyard with mana value X, except its name is Lazav, the
/// Multifarious, it's legendary in addition to its other types, and it has this ability." The card's copiable values with the
/// exceptions added, so they are copiable values too (CR 707.9a-b, 611.2a-b). Nothing if the target is gone (CR 608.2b).
fn become_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let values = g.copiable_values(card).named("Lazav, the Multifarious").legendary().with_ability(item.ability);
    g.become_copy(item.object, item.object, values, Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, |g, item| effects::surveil(g, item, 1), "When Lazav enters, surveil 1."),
        ability(
                cost("{X}"),
                become_copy,
                "{X}: Lazav becomes a copy of target creature card in your graveyard with mana value X, except its name is Lazav, the Multifarious, it's legendary in addition to its other types, and it has this ability.",
            )
            .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD_WITH_MANA_VALUE_X])
            .targets_after_x(),
    ],
    ..oracle::LAZAV_THE_MULTIFARIOUS
};
