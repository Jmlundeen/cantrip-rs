use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this artifact enters, mill a card. You may play that card this turn." (CR 701.17, 601.3, 305.1): the card the
/// mill put into the graveyard is the object played; with an empty library nothing is milled and nothing is granted.
fn mill_and_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for card in g.mill(item.controller, 1) {
        g.allow_play(card, item.controller, PlayGrant { lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, mill_and_play, "When this artifact enters, mill a card. You may play that card this turn."),
        mana_ability(TAP, add(Mana::R), "{T}: Add {R}."),
        mana_ability(TAP, add_n(Mana::R, 2).only_for(ManaRestriction::InstantOrSorcerySpells), "{T}: Add {R}{R}. Spend this mana only to cast instant and sorcery spells."),
    ],
    ..oracle::TABLET_OF_DISCOVERY
};
