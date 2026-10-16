use cantrip_api::*;
use cantrip_mechanics::effects::gain_life;
use cantrip_mechanics::replacements::{enter_tapped, opponents_enter_tapped};
use cantrip_oracle as oracle;

/// "Whenever a creature an opponent controls enters" (CR 603.6a): any creature, token or not, under the control of
/// a player other than this enchantment's controller as it entered.
fn opponents_creature_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) != q.controller(source) && q.chars(*new).is(CardType::Creature) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn gain_1(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    gain_life(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    replacements: &[opponents_enter_tapped::<{ enter_tapped::CREATURES }>("enters tapped")],
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, opponents_creature_enters), gain_1, "Whenever a creature an opponent controls enters, you gain 1 life.")],
    ..oracle::AUTHORITY_OF_THE_CONSULS
};
