use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Create two 1/1 red Officer creature tokens." (CR 111.1, 111.4): two tokens, made together. Then a new sentence,
/// "Create a tapped Planet land token named New Planet with '{T}: Add one mana of any color.'" (CR 111.4).
fn officers_and_planet(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::OFFICER, 2)?;
    g.end_sentence();
    tokens::create_by_rule(g, item.controller, &tokens::NEW_PLANET, 1, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{4}{W}").and(TAP).and(SACRIFICE_THIS),
            officers_and_planet,
            "{4}{W}, {T}, Sacrifice this land: Create two 1/1 red Officer creature tokens. Create a tapped Planet land token named New Planet with \"{T}: Add one mana of any color.\"",
        ),
    ],
    ..oracle::SUBSPACE_ANOMALY
};

pub static EXTRA: &[&CardDef] = &[&tokens::OFFICER, &tokens::NEW_PLANET];
