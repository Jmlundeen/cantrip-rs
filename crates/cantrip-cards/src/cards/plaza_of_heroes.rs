use cantrip_api::*;
use cantrip_mechanics::effects::target_gains_hexproof_and_indestructible_until_end_of_turn;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// The colors among the legendary permanents its controller controls (CR 105.2), read through the layers each time the
/// ability is offered. With none there is no color to choose: the ability is still offered and adds no mana (CR 106.5).
fn colors_among_legendary_permanents(q: &dyn Query, asker: &Asker) -> ColorSet {
    let mut colors = ColorSet::NONE;
    for id in q.permanents_of(asker.controller) {
        let c = q.chars(id);
        if c.supertypes.legendary {
            colors = ColorSet(colors.0 | c.colors.0);
        }
    }
    colors
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_any_color(1).only_for(ManaRestriction::LegendarySpells), "{T}: Add one mana of any color. Spend this mana only to cast a legendary spell."),
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color among legendary permanents you control.").colors_from(colors_among_legendary_permanents),
        ability(
            cost("{3}, {T}").and(EXILE_THIS),
            target_gains_hexproof_and_indestructible_until_end_of_turn,
            "{3}, {T}, Exile this land: Target legendary creature gains hexproof and indestructible until end of turn.",
        )
        .targets(&[targets::LEGENDARY_CREATURE]),
    ],
    ..oracle::PLAZA_OF_HEROES
};
