use cantrip_api::*;
use cantrip_mechanics::{effects, station, statics, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When U.S.S. Defiant enters, destroy up to one target artifact or enchantment." The target is chosen as the ability
/// triggers; an illegal target on resolution is ignored, and zero targets is a legal choice (CR 603.3, 608.2b).
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::destroy_first_target(g, item)
}
/// "5+ | Flying, hexproof" (CR 721.2a-b): while it has five or more charge counters it is a 7/4 artifact creature with
/// flying and hexproof itself (CR 702.9, 702.11); below 5+ it has neither.
fn flying_and_hexproof(q: &dyn LayerQuery, id: ObjectId, chars: &mut Chars) {
    statics::have_flying(q, id, chars);
    statics::have_hexproof(q, id, chars);
}

const FIVE_PLUS: &str = "5+ | Flying, hexproof";

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, destroy, "When U.S.S. Defiant enters, destroy up to one target artifact or enchantment.")
            .targets(&[targets::ARTIFACT_OR_ENCHANTMENT.between(0, 1)]),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 5+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<5>, station::is_a_creature, FIVE_PLUS),
        static_ability(Layer::Ability, statics::itself, flying_and_hexproof, FIVE_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<5>, statics::base_pt::<7, 4>, FIVE_PLUS).continues(),
    ],
    ..oracle::U_S_S_DEFIANT_TOUGH_LITTLE_SHIP
};
