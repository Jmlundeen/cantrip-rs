use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Draw two cards. If this spell's additional cost was paid, instead shuffle your graveyard into your library, draw seven
/// cards, and you have no maximum hand size for the rest of the game. Exile Spirit Water Revival."
///
/// The additional cost is the card's waterbend `kicker` (CR 701.67): paid, `item.kicked` is 1. The graveyard shuffled in is
/// the one the spell finds as it resolves, which does not hold the spell itself, still on the stack; a library is shuffled
/// even when there is no graveyard to put in it (CR 701.24a). The exile is the spell's last instruction.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if item.kicked > 0 {
        let graveyard: Vec<ObjectId> = g.state().zone(Zone::graveyard(you)).to_vec();
        if graveyard.is_empty() {
            g.shuffle(you);
        } else {
            g.put_into_library(&graveyard, LibraryPlace::ShuffledIn);
        }
        g.draw(you, 7);
        g.add_effect(item.object, item.object, EffectKind::NoMaximumHandSize, Duration::Indefinite);
    } else {
        g.draw(you, 2);
    }
    g.exile(item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    // "As an additional cost to cast this spell, you may waterbend {6}."
    kicker: Some(Kicker { cost: mana(6, 0, 0, 0, 0, 0), multi: false, waterbend: true }),
    ..oracle::SPIRIT_WATER_REVIVAL
};
