use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

/// "Whenever equipped creature deals combat damage to a player": the creature this is attached to dealing combat
/// damage to a player (CR 510.2); each such event is one player and one trigger (CR 603.2c).
fn equipped_deals_combat_damage_to_a_player(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: dealer, target: Target::Player(_), amount, combat: true }
            if *amount > 0 && q.attached_to(source) == Some(*dealer) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// Create a token that's a copy of this Equipment (CR 707.2, 111.4): the Equipment's copiable values, or its last known
/// ones if it has left the battlefield by now (CR 608.2h). The token is created unattached.
fn copy_this(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.create_token_copies(item.object, item.controller, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::DAMAGE, equipped_deals_combat_damage_to_a_player), copy_this, "Whenever equipped creature deals combat damage to a player, create a token that's a copy of this Equipment."),
        equip(mana("{2}"), "Equip {2}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<2, 0>, "Equipped creature gets +2/+0.")],
    ..oracle::BLOODFORGED_BATTLE_AXE
};
