use cantrip_api::*;
use cantrip_mechanics::{mana, pump, targets};
use cantrip_oracle as oracle;

/// Target creature gets +X/+0 and gains trample until end of turn, X being what was paid.
fn boost(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, item.x as i16, 0);
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{X}{R}{G}, {T}: Target creature gets +X/+0 and gains trample until end of turn.",
            cost: Cost { mana: ManaCost { x: 1, ..mana(0, 0, 0, 0, 1, 1) }, tap: true, ..Cost::NONE },
            targets: &[targets::CREATURE],
            effect: boost,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::KESSIG_WOLF_RUN
};
