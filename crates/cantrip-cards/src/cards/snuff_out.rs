use cantrip_api::*;
use cantrip_mechanics::destroy_target_no_regen;
use cantrip_oracle as oracle;

/// "Target nonblack creature" (CR 115.1).
fn nonblack_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id) && {
        let c = q.chars(id);
        c.is(CardType::Creature) && !c.colors.has(q.words(asker.source).color(Color::Black))
    })
}

/// "If you control a Swamp" (CR 118.9): a permanent with the Swamp land type, whatever its name.
fn control_a_swamp(q: &dyn Query, asker: &Asker) -> bool {
    q.state().battlefield().iter().any(|&id| q.controller(id) == asker.controller && q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Swamp)))
}

/// Destroy target nonblack creature. It can't be regenerated (CR 701.19c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target_no_regen(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: nonblack_creature }], effect }),
    ways: &[WayDef {
        text: "Pay 4 life",
        cost: Cost { life: 4, ..Cost::NONE },
        from: ZoneKind::Hand,
        exile_after: false,
        condition: Some(control_a_swamp),
        instant_speed: false,
    }],
    ..oracle::SNUFF_OUT
};
