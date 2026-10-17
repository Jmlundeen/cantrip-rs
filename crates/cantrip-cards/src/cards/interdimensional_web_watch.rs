use cantrip_api::*;
use cantrip_mechanics::library::exile_top_play_until_end_of_next_turn;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this artifact enters, exile the top two cards of your library. Until the end of your next turn, you may play those
/// cards." The ability's controller plays them, lands under the ordinary land-play rules and spells with all their costs
/// and timing (CR 305.1, 601.2; its ruling); a short library exiles what it has.
fn exile_top_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_until_end_of_next_turn(g, item.controller, 2);
    Ok(())
}

/// "Add two mana in any combination of colors. Spend this mana only to cast spells from exile." Each mana is a color chosen
/// as the ability resolves, repeats allowed (CR 106.1b), and pays only for casting a spell from exile (CR 106.6, 601.2a):
/// any spell cast from exile, not only one this artifact exiled (its ruling).
fn add_two(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    for color in cantrip_mechanics::choose_colors(g, asker.controller, asker.source, ColorSet(0b11111), 2)? {
        g.add_restricted_mana(asker.controller, color.mana(), 1, ManaRestriction::SPELLS_FROM_EXILE);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            exile_top_two,
            "When this artifact enters, exile the top two cards of your library. Until the end of your next turn, you may play those cards.",
        ),
        mana_ability(
            TAP,
            add_by(add_two).yields_any(2).only_for(ManaRestriction::SPELLS_FROM_EXILE),
            "{T}: Add two mana in any combination of colors. Spend this mana only to cast spells from exile.",
        ),
    ],
    ..oracle::INTERDIMENSIONAL_WEB_WATCH
};
