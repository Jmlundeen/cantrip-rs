use cantrip_api::*;
use cantrip_mechanics::{mana, station, targets};
use cantrip_oracle as oracle;

/// "Draw cards equal to the sacrificed creature's power" (CR 121.1, 608.2h): the power it had as it was sacrificed, read from
/// this activation's cost (`Query::cost_objects`, CR 602.2b). A power of 0 or less draws nothing.
fn draw_cards_equal_to_its_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(&sacrificed) = g.cost_objects(item).first() else { return Ok(()) };
    let power = g.chars(sacrificed).power.unwrap_or_default().max(0);
    g.draw(item.controller, power as u32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::B), "{T}: Add {B}."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Planet. Station only as a sorcery.)"),
    ],
    // "This land enters tapped" (CR 614.1d).
    enters: Some(mana::enters_tapped),
    // The 12+ striation (CR 721.2a): an activated ability the land has while it has twelve or more charge counters. Its
    // controller may sacrifice a creature only as a cost, the land itself is not one.
    delayed: &[ability(
        cost("{1}{B}").and(TAP).and(life(2)).and(sacrifice(1, targets::filters::creature)),
        draw_cards_equal_to_its_power,
        "{1}{B}, {T}, Pay 2 life, Sacrifice a creature: Draw cards equal to the sacrificed creature's power. Activate only as a sorcery.",
    )
    .sorcery_speed()],
    statics: &[static_ability(
        Layer::Ability,
        station::charged::<12>,
        station::has::<0>,
        "12+ | {1}{B}, {T}, Pay 2 life, Sacrifice a creature: Draw cards equal to the sacrificed creature's power. Activate only as a sorcery.",
    )],
    ..oracle::SUSUR_SECUNDI_VOID_ALTAR
};
