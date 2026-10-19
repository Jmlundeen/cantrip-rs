use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}: Choose a color. Add one mana of that color for each different power among creatures you control."
/// Count distinct current powers, including zero.
fn add_for_each_power(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    let mut powers: Vec<i16> = Vec::new();
    for id in g.permanents_of(asker.controller) {
        if g.chars(id).is(CardType::Creature) {
            let power = g.chars(id).power.unwrap_or(0);
            if !powers.contains(&power) {
                powers.push(power);
            }
        }
    }
    if !powers.is_empty() {
        g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), powers.len() as u8);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP,
        add_by(add_for_each_power).choose_color(),
        "{T}: Choose a color. Add one mana of that color for each different power among creatures you control.",
    )],
    ..oracle::LOOT_THE_NEXUS
};
