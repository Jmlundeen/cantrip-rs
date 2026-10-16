use cantrip_api::*;
use cantrip_mechanics::{connive_n, history, lands, targets};
use cantrip_oracle as oracle;

/// "Target creature you control connives X, where X is the number of creatures that died this turn": X is counted as
/// the ability resolves, and connive 0 is no connive at all (CR 701.50e). A target that is no longer legal is not
/// conniving.
fn connive_x(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = history::creatures_died_this_turn(g, None).min(255) as u8;
    if x == 0 {
        return Ok(());
    }
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    connive_n(g, creature, item.object, x)
}

pub static CARD: CardDef = CardDef {
    enters: Some(lands::enters_tapped_unless_you_control::<{ lands::sub(Subtype::Swamp) }>),
    abilities: &[
        mana_ability(TAP, add(Mana::B), "{T}: Add {B}."),
        ability(cost("{B}, {T}"), connive_x, "{B}, {T}: Target creature you control connives X, where X is the number of creatures that died this turn.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
    ],
    ..oracle::SPYMASTERS_VAULT
};
