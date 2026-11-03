use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "This creature can't be blocked by Walls." (CR 509.1b): a Wall, read through Juggernaut's own text (CR 612), may block
/// anything but it.
fn creature_attacks_combat_if_able_rule(
    q: &dyn Query,
    source: ObjectId,
    blocker: ObjectId,
    attacker: ObjectId,
) -> bool {
    let asker = &Asker::new(source, q.controller(source));
    !(attacker == asker.source
        && q.chars(blocker).has_subtype(q.words(asker.source).subtype(Subtype::Wall)))
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Ability,
            statics::itself,
            statics::attacks_each_combat_if_able,
            "This creature attacks each combat if able.",
        ),
    ],
    block_rules: &[creature_attacks_combat_if_able_rule],
    ..oracle::JUGGERNAUT
};
