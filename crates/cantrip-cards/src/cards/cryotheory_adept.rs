use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "{3}{U}, Exile this card from your graveyard: Tap target creature and put a stun counter on it." (CR 701.26a,
/// 122.1d.) The stun counter keeps it tapped through one untap.
fn freeze(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.tap(creature);
        g.add_counters(creature, CounterKind::Stun, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::PROWESS,
        ability(cost("{3}{U}").and(EXILE_THIS), freeze, "{3}{U}, Exile this card from your graveyard: Tap target creature and put a stun counter on it. Activate only as a sorcery.")
            .targets(&[targets::CREATURE])
            .from(ZoneKind::Graveyard)
            .sorcery_speed(),
    ],
    ..oracle::CRYOTHEORY_ADEPT
};
