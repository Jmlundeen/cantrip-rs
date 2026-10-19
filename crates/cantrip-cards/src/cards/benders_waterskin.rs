use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Untap this artifact during each other player's untap step" (CR 502.3, 611.3a): the rule is true of this artifact only.
fn this_artifact(_: &dyn Query, asker: &Asker, object: ObjectId) -> bool {
    object == asker.source
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color.")],
    untap_in_others_untap_steps: &[this_artifact],
    ..oracle::BENDERS_WATERSKIN
};
