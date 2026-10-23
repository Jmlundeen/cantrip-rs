use cantrip_api::*;
use cantrip_mechanics::{choose, damage_target, select, targets};
use cantrip_oracle as oracle;

/// "Volcanic Spite deals 3 damage to target creature, planeswalker, or battle." (CR 120.3) "You may put a card from your hand
/// on the bottom of your library. If you do, draw a card." (CR 401.4, 121.1): the card is chosen as the spell resolves, so an
/// empty hand is asked nothing; the draw follows a card actually put there (CR 608.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    g.end_sentence();
    let hand = g.state().zone(Zone::hand(item.controller)).to_vec();
    let chosen = choose::select(g, item.controller, item.object, Purpose::PutOnBottom, hand, select::cards(select::up_to(1)))?;
    if !g.put_into_library(&chosen, LibraryPlace::Bottom).is_empty() {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE_PLANESWALKER_OR_BATTLE], effect), ..oracle::VOLCANIC_SPITE };
