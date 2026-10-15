use cantrip_api::*;
use cantrip_mechanics::{destroy_target, mana, triggers::this_enters};
use cantrip_oracle as oracle;

/// "Target artifact or enchantment an opponent controls" (CR 115.1).
fn opponents_artifact_or_enchantment(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id) && q.controller(id) != asker.controller && {
        let chars = q.chars(id);
        chars.is(CardType::Artifact) || chars.is(CardType::Enchantment)
    })
}

/// Destroy it, if it is still a legal target.
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana::tap_for(mana::add_w, Mana::W, "{T}: Add {W}.")],
    enters: Some(mana::enters_tapped_unless_pay_three_life),
    ..oracle::WITCH_BLESSED_MEADOW
};

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this creature enters, destroy target artifact or enchantment an opponent controls.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[TargetSpec { min: 1, max: 1, filter: opponents_artifact_or_enchantment }],
        effect: destroy,
        ..TriggeredDef::BLANK
    })],
    back: Some(&BACK),
    ..oracle::WITCH_ENCHANTER
};
