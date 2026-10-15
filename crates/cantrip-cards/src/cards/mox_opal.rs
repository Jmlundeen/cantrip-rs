use cantrip_api::*;
use cantrip_oracle as oracle;

/// Metalcraft (CR 207.2c): "activate only if you control three or more artifacts" (CR 602.5b), counted through the
/// layers. A restriction on activating, not on having: the ability is on the Mox whatever the count.
fn metalcraft(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller).into_iter().filter(|&id| q.chars(id).is(CardType::Artifact)).count() >= 3
}

fn add_chosen(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Metalcraft \u{2014} {T}: Add one mana of any color. Activate only if you control three or more artifacts.",
        cost: Cost { tap: true, ..Cost::NONE },
        mana: Some(add_chosen),
        any_color: true,
        adds_any: 1,
        condition: Some(metalcraft),
        ..ActivatedDef::BLANK
    })],
    ..oracle::MOX_OPAL
};
