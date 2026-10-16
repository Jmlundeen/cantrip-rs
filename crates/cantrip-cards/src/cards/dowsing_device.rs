use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "up to one target creature you control".
const UP_TO_ONE_CREATURE_YOU_CONTROL: TargetSpec = TargetSpec { min: 0, max: 1, ..targets::CREATURE_YOU_CONTROL };

fn artifacts_you_control(q: &dyn Query, player: PlayerId) -> usize {
    q.permanents_of(player).into_iter().filter(|&id| q.chars(id).is(CardType::Artifact)).count()
}

/// "Whenever this artifact or another artifact you control enters" (CR 603.6a): one ability for both, its own entry
/// included.
fn artifact_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) == q.controller(source) && q.chars(*new).is(CardType::Artifact) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn haste_until_end_of_turn(g: &mut dyn Game, source: ObjectId, creature: ObjectId) {
    g.add_effect(source, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::EndOfTurn);
}

/// The target gets +1/+0 and haste until end of turn; then the artifacts are counted as it resolves, and the
/// Device transforms if there are four or more (CR 608.2c).
fn pump_then_transform(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, 1, 0);
        haste_until_end_of_turn(g, item.object, creature);
    }
    if artifacts_you_control(g, item.controller) >= 4 {
        g.transform(item.object);
    }
    Ok(())
}

/// "Until end of turn, target creature gains haste and gets +X/+0, where X is the number of artifacts you control."
fn grotto_pump(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        let x = artifacts_you_control(g, item.controller) as i16;
        pump(g, item.object, creature, x, 0);
        haste_until_end_of_turn(g, item.object, creature);
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::R), "{T}: Add {R}."),
        ability(cost("{2}{R}, {T}"), grotto_pump, "{2}{R}, {T}: Until end of turn, target creature gains haste and gets +X/+0, where X is the number of artifacts you control. Activate only as a sorcery.")
            .sorcery_speed()
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::GEODE_GROTTO
};

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, artifact_enters), pump_then_transform, "Whenever this artifact or another artifact you control enters, up to one target creature you control gets +1/+0 and gains haste until end of turn. Then transform this artifact if you control four or more artifacts.")
        .targets(&[UP_TO_ONE_CREATURE_YOU_CONTROL])],
    back: Some(&BACK),
    ..oracle::DOWSING_DEVICE
};
