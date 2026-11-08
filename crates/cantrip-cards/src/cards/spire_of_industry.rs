use cantrip_api::*;
use cantrip_mechanics::artifacts_you_control;
use cantrip_oracle as oracle;

/// "Activate only if you control an artifact" (CR 602.5b): checked as the ability is activated, before its cost is paid.
/// Spire of Industry itself counts while it is an artifact (it is not), and an opponent's artifacts do not count.
fn control_an_artifact(q: &dyn Query, asker: &Asker) -> bool {
    artifacts_you_control(q, asker.controller) >= 1
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP.and(life(1)), add_any_color(1), "{T}, Pay 1 life: Add one mana of any color. Activate only if you control an artifact.")
            .condition(control_an_artifact),
    ],
    ..oracle::SPIRE_OF_INDUSTRY
};
