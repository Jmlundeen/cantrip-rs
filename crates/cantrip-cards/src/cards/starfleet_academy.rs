use cantrip_api::*;
use cantrip_mechanics::creature_types_among_non_borg;
use cantrip_oracle as oracle;

/// "Choose a color. Add an amount of mana of that color equal to the number of creature types among non-Borg creatures you
/// control." (CR 106.5, 608.2h): both the color and the count are settled as the ability resolves, and with no such
/// creature types it adds nothing.
fn add_by_creature_types(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    let count = creature_types_among_non_borg(g, asker.source, asker.controller) as u32;
    if let Some(color) = color
        && count > 0
    {
        g.add_mana(asker.controller, color.mana(), count);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(
            cost("{5}, {T}"),
            add_by(add_by_creature_types).choose_color(),
            "Federation \u{2014} {5}, {T}: Choose a color. Add an amount of mana of that color equal to the number of creature types among non-Borg creatures you control.",
        ),
    ],
    ..oracle::STARFLEET_ACADEMY
};
