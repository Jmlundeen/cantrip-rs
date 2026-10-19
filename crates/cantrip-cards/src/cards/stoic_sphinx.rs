use cantrip_api::*;
use cantrip_mechanics::statics::have_hexproof;
use cantrip_oracle as oracle;

/// "This creature has hexproof as long as you haven't cast a spell this turn" (CR 702.11, 604.1, 611.3a): only the Sphinx itself,
/// and only while its controller - "you", whoever that is now - has no spell cast this turn in the event history, which the layers
/// ask again whenever characteristics are computed, so the hexproof stops the moment a spell is cast (the Sphinx's own cast,
/// flashed in, included) and comes back as the turn changes.
fn itself_while_its_controller_has_cast_no_spell_this_turn(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && view.history().casts_by(view.state().obj(source).controller).next().is_none()
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::Ability,
        itself_while_its_controller_has_cast_no_spell_this_turn,
        have_hexproof,
        "This creature has hexproof as long as you haven't cast a spell this turn.",
    )],
    ..oracle::STOIC_SPHINX
};
