use cantrip_api::*;
use cantrip_mechanics::{connive_n, lands, targets};
use cantrip_oracle as oracle;

/// The number of creatures that died this turn (CR 700.4): every move from the battlefield to a graveyard this turn of
/// an object that was a creature as it left, by last known information, so a token that has since ceased to exist
/// still counts.
fn creatures_died_this_turn(q: &dyn Query) -> u8 {
    q.history()
        .select(EventQuery::kind(EventMask::ZONE_CHANGE).from(ZoneKind::Battlefield).to(ZoneKind::Graveyard).this_turn())
        .filter(|r| {
            matches!(r.event, Event::ZoneChange { old, .. }
                if q.state().last_known(old).is_some_and(|l| l.types.has(CardType::Creature)))
        })
        .count()
        .min(255) as u8
}

/// "Target creature you control connives X, where X is the number of creatures that died this turn": X is counted as
/// the ability resolves, and connive 0 is no connive at all (CR 701.50e). A target that is no longer legal is not
/// conniving.
fn connive_x(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = creatures_died_this_turn(g);
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
