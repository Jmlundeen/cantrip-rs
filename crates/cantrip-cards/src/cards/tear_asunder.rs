use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Target artifact or enchantment", or any nonland permanent when the spell was kicked: judged by the kicker count
/// announced before the target is chosen, and again as it resolves (CR 601.2c, 608.2b).
fn artifact_or_enchantment(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    if !is_live(q, id) || !q.is_on_battlefield(id) {
        return false;
    }
    let chars = q.chars(id);
    if q.obj(asker.source).kicked > 0 {
        !chars.is(CardType::Land)
    } else {
        chars.is(CardType::Artifact) || chars.is(CardType::Enchantment)
    }
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(artifact_or_enchantment)], effects::exile_first_target),
    kicker: kicker(mana("{1}{B}")),
    late_targets: true,
    ..oracle::TEAR_ASUNDER
};
