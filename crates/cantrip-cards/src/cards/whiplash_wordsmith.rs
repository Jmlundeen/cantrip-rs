use cantrip_api::*;
use cantrip_mechanics::{history, prepare, statics};
use cantrip_oracle as oracle;

/// "As long as an opponent was dealt noncombat damage this turn" is asked of the current controller's opponents
/// in this turn's event history whenever the layers apply (CR 611.3a).
fn opponent_damaged_this_turn(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected
        && history::opponent_dealt_noncombat_damage_this_turn(view.history(), view.state(), view.controller(source))
}

/// Enters prepared with the same Vicious Verse as Stingerquill Voxmancer (CR 722.3c).
pub static CARD: CardDef = CardDef {
    enters: Some(prepare::enters_prepared),
    prepare: Some(&super::stingerquill_voxmancer::VICIOUS_VERSE),
    statics: &[static_ability(
        Layer::Ability,
        opponent_damaged_this_turn,
        statics::have_flying_and_haste,
        "As long as an opponent was dealt noncombat damage this turn, this creature has flying and haste.",
    )],
    ..oracle::WHIPLASH_WORDSMITH
};
