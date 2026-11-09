use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

/// "Add X mana in any combination of colors" (CR 106.1b) is the shared announced-X combination recipe: X was announced
/// with the activation (CR 107.3a, 605.3a) and is read as the ability resolves, after its cost sacrificed the source.
/// The recipe also states what the ability could produce (CR 106.7) without guessing an X or pricing its cost.
/// "When this artifact is put into a graveyard from the battlefield" (CR 700.4, 603.10a): looked back on.
pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(cost("{X}, {T}").and(SACRIFICE_THIS),
            cantrip_mechanics::mana::announced_x_in_any_combination::<0b11111>(),
            "{X}, {T}, Sacrifice this artifact: Add X mana in any combination of colors.").x_is_mana_amount(),
        triggered(THIS_DIES, |g, item| effects::draw(g, item, 1), "When this artifact is put into a graveyard from the battlefield, draw a card."),
    ],
    enters: Some(cantrip_mechanics::mana::enters_tapped),
    ..oracle::WIZARDS_ROCKETS
};
